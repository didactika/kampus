// ------------------------------------------------------------------------------
//
//     _  __    _    __  __ ____  _   _ ____
//    | |/ /   / \  |  \/  |  _ \| | | / ___|
//    | ' /   / _ \ | |\/| | |_) | | | \___ \
//    | . \  / ___ \| |  | |  __/| |_| |___) |
//    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
//
//    Knowledge Academic Modular Platform for Universal Study
//
// ------------------------------------------------------------------------------
//
// This file is part of Kampus - https://github.com/didactika/kampus
//
// Kampus is free software: you can redistribute it and/or modify it under the terms of the
// GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with Kampus.
// If not, see <https://www.gnu.org/licenses/>.
//
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Enrolments: created through an instance with its capacity and opening period checked in the same
//! transaction, with the instance locked, so concurrent requests never exceed it. An enrolment owns the
//! grant of its role, and only its own grants and group memberships go with it.

use kampus_access::{Actor, GrantId, PrincipalId, RoleId, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_courses::{CourseId, GroupId};
use kampus_db::references::ExternalReference;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::{check_version, instant};
use crate::permissions::{ENROLMENT_MANAGE, ENROLMENT_READ};
use crate::{
    Enrol, Enrolment, EnrolmentChange, EnrolmentId, Instance, InstanceId, Method, Period, Status, sql,
};

/// The origin of the group memberships enrolment owns in courses.
pub(crate) const ORIGIN: &str = "enrol";

/// The columns of an enrolment `e` with the course of its instance `i`.
const COLUMNS: &str = "e.id, e.instance_id, i.course_id, e.account_id, e.status, e.starts_at, e.ends_at, \
    e.created_by, e.reason, e.version";

/// An enrolment from its row.
fn enrolment(row: &PgRow) -> Enrolment {
    Enrolment {
        id: EnrolmentId::from_uuid(row.get("id")),
        instance: InstanceId::from_uuid(row.get("instance_id")),
        course: CourseId::from_uuid(row.get("course_id")),
        account: PrincipalId::from_uuid(row.get("account_id")),
        status: Status::read(row.get::<String, _>("status").as_str()),
        starts_at: instant(row, "starts_at")
            .unwrap_or_else(|| Timestamp::from_offset_date_time(time::OffsetDateTime::UNIX_EPOCH)),
        ends_at: instant(row, "ends_at"),
        created_by: row
            .get::<Option<uuid::Uuid>, _>("created_by")
            .map(PrincipalId::from_uuid),
        reason: row.get("reason"),
        version: row.get("version"),
    }
}

/// The principal of an actor, if it acts as one.
pub(crate) const fn principal(actor: &Actor) -> Option<PrincipalId> {
    match actor {
        Actor::Principal(p) => Some(*p),
        Actor::Visitor(_) | Actor::ViewAs(..) => None,
    }
}

/// A group membership an instance owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Owned {
    /// The group.
    pub(crate) group: GroupId,
    /// The account.
    pub(crate) account: PrincipalId,
}

impl Enrol {
    /// Enrols people through a manual instance, each one with the same rules as alone; the answer says
    /// what happened to each. Needs `enrol.enrolment.manage` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an instance that is not manual; the
    /// error of each person is in its place of the answer.
    pub async fn enrol(
        &self,
        actor: &Actor,
        instance: InstanceId,
        accounts: &[PrincipalId],
        period: Period,
    ) -> Result<Vec<(PrincipalId, Result<EnrolmentId>)>> {
        let stored = self.instance(instance).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, ENROLMENT_MANAGE, scope).await?;
        if stored.method != Method::Manual {
            return Err(Error::validation(
                "enrol.method_not_manual",
                vec![FieldError::new("instance", "manual")],
            ));
        }
        let mut report = Vec::with_capacity(accounts.len());
        for account in accounts {
            let result = self
                .create_enrolment(&stored, *account, period, principal(actor))
                .await;
            report.push((*account, result));
        }
        Ok(report)
    }

    /// An enrolment.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown enrolment.
    pub async fn enrolment(&self, id: EnrolmentId) -> Result<Enrolment> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.read_enrolment_in(&mut conn, id, false).await
    }

    /// The enrolments of a course, every one or only the active ones; needs `enrol.enrolment.read`.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn participants(
        &self,
        actor: &Actor,
        course: CourseId,
        active_only: bool,
    ) -> Result<Vec<Enrolment>> {
        let scope = self.courses.course(course).await?.scope;
        self.access.require(actor, ENROLMENT_READ, scope).await?;
        let filter = if active_only {
            format!(" and {}", super::participation::ACTIVE)
        } else {
            String::new()
        };
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} e join {} i on i.id = e.instance_id where i.course_id = $1{filter}
             order by e.created_at, e.id",
            self.table("enrolment"),
            self.table("instance")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(enrolment).collect())
    }

    /// Changes the status, period or reason of an enrolment: suspending it takes its role away until it
    /// is active again, and a new end moves the end of its role. Needs `enrol.enrolment.manage`.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an end before the start,
    /// `precondition_failed` when it changed since it was read.
    pub async fn update_enrolment(
        &self,
        actor: &Actor,
        id: EnrolmentId,
        change: EnrolmentChange,
        expected_version: Option<i32>,
    ) -> Result<Enrolment> {
        let stored = self.enrolment(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, ENROLMENT_MANAGE, scope).await?;
        let starts_at = change.starts_at.unwrap_or(stored.starts_at);
        if change.ends_at.is_some_and(|e| e <= starts_at) {
            return Err(Error::validation(
                "enrol.enrolment_invalid",
                vec![FieldError::new("ends_at", "after_starts_at")],
            ));
        }
        let mut tx = self.db.begin().await?;
        let locked = self.read_enrolment_in(&mut tx, id, true).await?;
        check_version(expected_version, locked.version, "enrol.enrolment_changed")?;
        let instance = self.read_instance_in(&mut tx, locked.instance, false).await?;
        sqlx::query(sql(format!(
            "update {} set status = $2, starts_at = $3, ends_at = $4, reason = $5, expired_at = null,
             version = version + 1 where id = $1",
            self.table("enrolment")
        )))
        .bind(id.as_uuid())
        .bind(change.status.as_str())
        .bind(starts_at.as_offset_date_time())
        .bind(change.ends_at.map(|e| e.as_offset_date_time()))
        .bind(
            change
                .reason
                .map(|r| r.trim().to_owned())
                .filter(|r| !r.is_empty()),
        )
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.revoke_grants_in(&mut tx, id).await?;
        if change.status == Status::Active {
            self.grant_in(&mut tx, id, &instance, locked.account, starts_at, change.ends_at)
                .await?;
        }
        let updated = self.read_enrolment_in(&mut tx, id, false).await?;
        self.publish_in(
            &mut tx,
            crate::events::ENROLMENT_UPDATED,
            id,
            updated.course,
            updated.account,
            serde_json::json!({ "status": updated.status }),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(updated)
    }

    /// Removes an enrolment with its grants and the group memberships it brought; records and progress
    /// stay. One that a system group gives is removed by leaving the group. Needs
    /// `enrol.enrolment.manage`.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `conflict` (`enrol.enrolment_linked`) for an enrolment
    /// a system group gives, `precondition_failed` when it changed since it was read.
    pub async fn remove_enrolment(
        &self,
        actor: &Actor,
        id: EnrolmentId,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let stored = self.enrolment(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, ENROLMENT_MANAGE, scope).await?;
        check_version(expected_version, stored.version, "enrol.enrolment_changed")?;
        if self.instance(stored.instance).await?.method == Method::SystemGroup {
            return Err(Error::conflict("enrol.enrolment_linked"));
        }
        self.delete_enrolment(&stored).await
    }

    /// Creates an enrolment through an instance after its checks, with its grant.
    pub(crate) async fn create_enrolment(
        &self,
        instance: &Instance,
        account: PrincipalId,
        period: Period,
        created_by: Option<PrincipalId>,
    ) -> Result<EnrolmentId> {
        self.create_enrolment_with(instance, account, period, created_by, None)
            .await
    }

    /// Creates an enrolment, with the identifier a system gives it when one is given, in one transaction.
    pub(crate) async fn create_enrolment_with(
        &self,
        instance: &Instance,
        account: PrincipalId,
        period: Period,
        created_by: Option<PrincipalId>,
        reference: Option<(ScopeId, &ExternalReference)>,
    ) -> Result<EnrolmentId> {
        let mut tx = self.db.begin().await?;
        let instance = self.read_instance_in(&mut tx, instance.id, true).await?;
        let course = self.courses.course(instance.course).await?;
        if !self.in_campus(account, course.campus).await? {
            return Err(Error::validation(
                "enrol.account_invalid",
                vec![FieldError::new("account", "campus")],
            ));
        }
        let checks = sqlx::query(sql(format!(
            "select ($2::timestamptz is null or $2 <= now()) and ($3::timestamptz is null or now() < $3) as open,
             (select count(*) from {} where instance_id = $1) as taken",
            self.table("enrolment")
        )))
        .bind(instance.id.as_uuid())
        .bind(instance.open_from.map(|t| t.as_offset_date_time()))
        .bind(instance.open_until.map(|t| t.as_offset_date_time()))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if !instance.enabled || !checks.get::<bool, _>("open") {
            return Err(Error::conflict("enrol.instance_closed"));
        }
        if instance
            .capacity
            .is_some_and(|c| checks.get::<i64, _>("taken") >= i64::from(c))
        {
            return Err(Error::conflict("enrol.instance_full"));
        }
        let starts_at = period.starts_at.unwrap_or_else(|| self.now());
        let ends_at = match (period.ends_at, instance.default_days) {
            (Some(end), _) => Some(end),
            (None, Some(days)) => Some(Timestamp::from_offset_date_time(
                starts_at.as_offset_date_time() + time::Duration::days(i64::from(days)),
            )),
            (None, None) => None,
        };
        if ends_at.is_some_and(|e| e <= starts_at) {
            return Err(Error::validation(
                "enrol.enrolment_invalid",
                vec![FieldError::new("ends_at", "after_starts_at")],
            ));
        }
        let id = EnrolmentId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, instance_id, account_id, starts_at, ends_at, created_by) values ($1, $2, $3, $4, $5, $6)",
            self.table("enrolment")
        )))
        .bind(id.as_uuid())
        .bind(instance.id.as_uuid())
        .bind(account.as_uuid())
        .bind(starts_at.as_offset_date_time())
        .bind(ends_at.map(|e| e.as_offset_date_time()))
        .bind(created_by.map(|c| c.as_uuid()))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.unique_violation" {
                Error::conflict("enrol.already_enrolled")
            } else {
                e
            }
        })?;
        self.grant_in(&mut tx, id, &instance, account, starts_at, ends_at)
            .await?;
        self.publish_in(
            &mut tx,
            crate::events::ENROLMENT_CREATED,
            id,
            instance.course,
            account,
            serde_json::json!({ "instance": instance.id }),
        )
        .await?;
        if let Some((campus, reference)) = reference {
            self.enrolment_references()
                .set_in(&mut tx, campus.as_uuid(), id.as_uuid(), reference)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Deletes an enrolment with its grants and the group memberships its instance owns for the person.
    pub(crate) async fn delete_enrolment(&self, stored: &Enrolment) -> Result<()> {
        let owned = self
            .owned_memberships(stored.instance, Some(stored.account))
            .await?;
        let mut tx = self.db.begin().await?;
        self.revoke_grants_in(&mut tx, stored.id).await?;
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("enrolment")
        )))
        .bind(stored.id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "delete from {} where instance_id = $1 and account_id = $2",
            self.table("enrolment_group_member")
        )))
        .bind(stored.instance.as_uuid())
        .bind(stored.account.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            crate::events::ENROLMENT_REMOVED,
            stored.id,
            stored.course,
            stored.account,
            serde_json::json!({}),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.release_memberships(&owned).await
    }

    /// Grants the instance's role in the course to the account, until the end of the enrolment, owned
    /// by the enrolment.
    pub(crate) async fn grant_in(
        &self,
        conn: &mut PgConnection,
        id: EnrolmentId,
        instance: &Instance,
        account: PrincipalId,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> Result<()> {
        let scope = self.courses.course(instance.course).await?.scope;
        let grant = self
            .access
            .grant_for_module_between_in(
                conn,
                crate::module::NAME,
                instance.role,
                scope,
                account,
                Some(from),
                until,
            )
            .await?;
        sqlx::query(sql(format!(
            "insert into {} (enrolment_id, grant_id) values ($1, $2)",
            self.table("enrolment_grant")
        )))
        .bind(id.as_uuid())
        .bind(grant.as_uuid())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Removes the grants an enrolment owns.
    pub(crate) async fn revoke_grants_in(&self, conn: &mut PgConnection, id: EnrolmentId) -> Result<()> {
        let grants: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "delete from {} where enrolment_id = $1 returning grant_id",
            self.table("enrolment_grant")
        )))
        .bind(id.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for grant in grants {
            self.access
                .revoke_for_module_in(conn, crate::module::NAME, GrantId::from_uuid(grant))
                .await?;
        }
        Ok(())
    }

    /// Gives the active enrolments of an instance the grant of another role.
    pub(crate) async fn regrant_instance_in(
        &self,
        conn: &mut PgConnection,
        instance: &Instance,
        role: RoleId,
    ) -> Result<()> {
        let rows = sqlx::query(sql(format!(
            "select id, account_id, starts_at, ends_at from {} where instance_id = $1 and status = 'active'",
            self.table("enrolment")
        )))
        .bind(instance.id.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let changed = Instance {
            role,
            ..instance.clone()
        };
        for row in rows {
            let id = EnrolmentId::from_uuid(row.get("id"));
            self.revoke_grants_in(conn, id).await?;
            self.grant_in(
                conn,
                id,
                &changed,
                PrincipalId::from_uuid(row.get("account_id")),
                instant(&row, "starts_at").unwrap_or_else(|| self.now()),
                instant(&row, "ends_at"),
            )
            .await?;
        }
        Ok(())
    }

    /// The group memberships an instance owns, for one account or all.
    pub(crate) async fn owned_memberships(
        &self,
        instance: InstanceId,
        account: Option<PrincipalId>,
    ) -> Result<Vec<Owned>> {
        let rows = sqlx::query(sql(format!(
            "select group_id, account_id from {} where instance_id = $1 and ($2::uuid is null or account_id = $2)",
            self.table("enrolment_group_member")
        )))
        .bind(instance.as_uuid())
        .bind(account.map(|a| a.as_uuid()))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Owned {
                group: GroupId::from_uuid(r.get("group_id")),
                account: PrincipalId::from_uuid(r.get("account_id")),
            })
            .collect())
    }

    /// Removes from courses the memberships no instance owns any more.
    pub(crate) async fn release_memberships(&self, owned: &[Owned]) -> Result<()> {
        for membership in owned {
            let still: bool = sqlx::query_scalar(sql(format!(
                "select exists (select 1 from {} where group_id = $1 and account_id = $2)",
                self.table("enrolment_group_member")
            )))
            .bind(membership.group.as_uuid())
            .bind(membership.account.as_uuid())
            .fetch_one(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            if !still {
                self.courses
                    .remove_module_member(ORIGIN, membership.group, membership.account)
                    .await?;
            }
        }
        Ok(())
    }

    /// Puts an enrolled person in a group of the course, owned by the instance.
    pub(crate) async fn join_group(
        &self,
        instance: InstanceId,
        group: GroupId,
        account: PrincipalId,
    ) -> Result<()> {
        self.courses.add_module_member(ORIGIN, group, account).await?;
        sqlx::query(sql(format!(
            "insert into {} (instance_id, group_id, account_id) values ($1, $2, $3) on conflict do nothing",
            self.table("enrolment_group_member")
        )))
        .bind(instance.as_uuid())
        .bind(group.as_uuid())
        .bind(account.as_uuid())
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Whether an account belongs to a campus.
    async fn in_campus(&self, account: PrincipalId, campus: kampus_access::ScopeId) -> Result<bool> {
        match self.identity.campus_of(account).await {
            Ok(of) => Ok(of == campus),
            Err(e) if e.kind() == kampus_common::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Reads an enrolment, locked for a change when `lock`.
    pub(crate) async fn read_enrolment_in(
        &self,
        conn: &mut PgConnection,
        id: EnrolmentId,
        lock: bool,
    ) -> Result<Enrolment> {
        let lock = if lock { " for update of e" } else { "" };
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} e join {} i on i.id = e.instance_id where e.id = $1{lock}",
            self.table("enrolment"),
            self.table("instance")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("enrol.enrolment_not_found"))?;
        Ok(enrolment(&row))
    }
}

impl Enrol {
    /// Publishes an event about an enrolment in the transaction of its change.
    pub(crate) async fn publish_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        enrolment: EnrolmentId,
        course: CourseId,
        account: PrincipalId,
        extra: serde_json::Value,
    ) -> Result<()> {
        let mut payload = serde_json::json!({ "enrolment": enrolment, "course": course, "account": account });
        if let (Some(target), Some(more)) = (payload.as_object_mut(), extra.as_object()) {
            target.extend(more.clone());
        }
        let scope = self.courses.course(course).await?.scope;
        let mut event = kampus_events::Publish::new(code, Some(enrolment.as_uuid()), payload);
        event.scope = Some(scope.as_uuid());
        event.summary = Some(format!("{code} {enrolment}"));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }
}
