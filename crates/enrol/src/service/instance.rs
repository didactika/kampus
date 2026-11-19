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

//! Method instances: a method configured in a course, with the role its enrolments give, its period,
//! capacity, approval, keys and expiry rule. Whoever configures one must be able to grant its role.

use kampus_access::{Actor, RoleId};
use kampus_common::{Error, FieldError, Result};
use kampus_courses::{CourseId, GroupId};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::{check_version, instant};
use crate::permissions::INSTANCE_MANAGE;
use crate::{Enrol, ExpiryAction, Instance, InstanceFields, InstanceId, Method, sql};

/// The longest name of an instance, in characters.
const MAX_NAME: usize = 255;

/// The shortest self-enrolment key, in characters.
const MIN_KEY: usize = 6;

/// The columns of an instance, as read from `i`.
pub(crate) fn columns(enrol: &Enrol) -> String {
    format!(
        "i.id, i.course_id, i.method_code, i.name, i.role_id, i.enabled, i.open_from, i.open_until,
         i.default_days, i.capacity, i.requires_approval, i.expiry_action, i.version,
         exists (select 1 from {keys} k where k.instance_id = i.id) as has_key,
         (select l.system_group_id from {links} l where l.instance_id = i.id) as source",
        keys = enrol.table("self_key"),
        links = enrol.table("linked_group")
    )
}

/// An instance from its row.
pub(crate) fn instance(row: &PgRow) -> Result<Instance> {
    Ok(Instance {
        id: InstanceId::from_uuid(row.get("id")),
        course: CourseId::from_uuid(row.get("course_id")),
        method: row.get::<String, _>("method_code").parse()?,
        name: row.get("name"),
        role: RoleId::from_uuid(row.get("role_id")),
        enabled: row.get("enabled"),
        open_from: instant(row, "open_from"),
        open_until: instant(row, "open_until"),
        default_days: row.get("default_days"),
        capacity: row.get("capacity"),
        requires_approval: row.get("requires_approval"),
        expiry_action: ExpiryAction::read(row.get::<String, _>("expiry_action").as_str()),
        has_key: row.get("has_key"),
        source: row
            .get::<Option<uuid::Uuid>, _>("source")
            .map(kampus_users::GroupId::from_uuid),
        version: row.get("version"),
    })
}

/// Checks the fields of an instance, every problem at once.
fn checked(fields: &InstanceFields) -> Result<InstanceFields> {
    let mut problems = Vec::new();
    let name = fields.name.trim().to_owned();
    if name.is_empty() {
        problems.push(FieldError::new("name", "required"));
    } else if name.chars().count() > MAX_NAME {
        problems.push(FieldError::new("name", "length"));
    }
    if let (Some(from), Some(until)) = (fields.open_from, fields.open_until)
        && until <= from
    {
        problems.push(FieldError::new("open_until", "after_open_from"));
    }
    if fields.capacity.is_some_and(|c| c < 1) {
        problems.push(FieldError::new("capacity", "positive"));
    }
    if fields.default_days.is_some_and(|d| d < 1) {
        problems.push(FieldError::new("default_days", "positive"));
    }
    if problems.is_empty() {
        Ok(InstanceFields {
            name,
            ..fields.clone()
        })
    } else {
        Err(Error::validation("enrol.instance_invalid", problems))
    }
}

/// A name in use answered as this module's conflict.
fn taken(error: Error) -> Error {
    if error.code() == "db.unique_violation" {
        Error::conflict("enrol.instance_name_taken")
    } else {
        error
    }
}

impl Enrol {
    /// Adds a method instance to a course; needs `enrol.instance.manage` there and being able to grant
    /// its role. A self instance may take a key; a system group instance takes the group it follows.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission or the allowance, `validation` for invalid fields, a
    /// short key, or a system group instance without its group, `conflict` for a name in use.
    pub async fn create_instance(
        &self,
        actor: &Actor,
        course: CourseId,
        method: Method,
        fields: InstanceFields,
        key: Option<&str>,
        source: Option<kampus_users::GroupId>,
    ) -> Result<Instance> {
        let fields = checked(&fields)?;
        let scope = self.courses.course(course).await?.scope;
        self.access
            .require_may_grant(actor, INSTANCE_MANAGE, scope, fields.role)
            .await?;
        let mut problems = Vec::new();
        if method == Method::SystemGroup && source.is_none() {
            problems.push(FieldError::new("source", "required"));
        }
        if method != Method::SystemGroup && source.is_some() {
            problems.push(FieldError::new("source", "system_group_only"));
        }
        if key.is_some() && method != Method::SelfEnrol {
            problems.push(FieldError::new("key", "self_only"));
        }
        if key.is_some_and(|k| k.chars().count() < MIN_KEY) {
            problems.push(FieldError::new("key", "length"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("enrol.instance_invalid", problems));
        }
        if let Some(group) = source {
            self.users.group(group).await?;
        }
        let hash = match key {
            Some(k) => Some(self.hasher.hash(k).await?),
            None => None,
        };
        let mut tx = self.db.begin().await?;
        let id = InstanceId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, course_id, method_code, name, role_id, enabled, open_from, open_until,
             default_days, capacity, requires_approval, expiry_action)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            self.table("instance")
        )))
        .bind(id.as_uuid())
        .bind(course.as_uuid())
        .bind(method.as_str())
        .bind(&fields.name)
        .bind(fields.role.as_uuid())
        .bind(fields.enabled)
        .bind(fields.open_from.map(|t| t.as_offset_date_time()))
        .bind(fields.open_until.map(|t| t.as_offset_date_time()))
        .bind(fields.default_days)
        .bind(fields.capacity)
        .bind(fields.requires_approval)
        .bind(fields.expiry_action.as_str())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken)?;
        if let Some(hash) = hash {
            self.insert_key_in(&mut tx, id, None, &hash).await?;
        }
        if let Some(group) = source {
            sqlx::query(sql(format!(
                "insert into {} (instance_id, system_group_id) values ($1, $2)",
                self.table("linked_group")
            )))
            .bind(id.as_uuid())
            .bind(group.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        let created = self.read_instance_in(&mut tx, id, false).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// An instance, as stored.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown instance.
    pub async fn instance(&self, id: InstanceId) -> Result<Instance> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.read_instance_in(&mut conn, id, false).await
    }

    /// The instances of a course, oldest first; needs `enrol.instance.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn instances(&self, actor: &Actor, course: CourseId) -> Result<Vec<Instance>> {
        let scope = self.courses.course(course).await?.scope;
        self.access.require(actor, INSTANCE_MANAGE, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select {} from {} i where i.course_id = $1 order by i.created_at, i.id",
            columns(self),
            self.table("instance")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.iter().map(instance).collect()
    }

    /// Changes an instance; disabling it suspends the participation it gives without deleting anything.
    /// Needs `enrol.instance.manage` and being able to grant its role.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission or the allowance, `validation` for invalid fields,
    /// `precondition_failed` when it changed since it was read, `conflict` for a name in use.
    pub async fn update_instance(
        &self,
        actor: &Actor,
        id: InstanceId,
        fields: InstanceFields,
        expected_version: Option<i32>,
    ) -> Result<Instance> {
        let fields = checked(&fields)?;
        let stored = self.instance(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access
            .require_may_grant(actor, INSTANCE_MANAGE, scope, fields.role)
            .await?;
        let mut tx = self.db.begin().await?;
        let locked = self.read_instance_in(&mut tx, id, true).await?;
        check_version(expected_version, locked.version, "enrol.instance_changed")?;
        sqlx::query(sql(format!(
            "update {} set name = $2, role_id = $3, enabled = $4, open_from = $5, open_until = $6,
             default_days = $7, capacity = $8, requires_approval = $9, expiry_action = $10,
             version = version + 1 where id = $1",
            self.table("instance")
        )))
        .bind(id.as_uuid())
        .bind(&fields.name)
        .bind(fields.role.as_uuid())
        .bind(fields.enabled)
        .bind(fields.open_from.map(|t| t.as_offset_date_time()))
        .bind(fields.open_until.map(|t| t.as_offset_date_time()))
        .bind(fields.default_days)
        .bind(fields.capacity)
        .bind(fields.requires_approval)
        .bind(fields.expiry_action.as_str())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken)?;
        if fields.role != locked.role {
            self.regrant_instance_in(&mut tx, &locked, fields.role).await?;
        }
        let updated = self.read_instance_in(&mut tx, id, false).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(updated)
    }

    /// Sets or removes a self-enrolment key of an instance, for the whole course or for one of its
    /// groups: who gives a group's key joins that group.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `enrol.instance.manage`, `validation` for an instance that is not a self
    /// one or a key that is too short.
    pub async fn set_key(
        &self,
        actor: &Actor,
        id: InstanceId,
        group: Option<GroupId>,
        key: Option<&str>,
    ) -> Result<()> {
        let stored = self.instance(id).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, INSTANCE_MANAGE, course.scope).await?;
        let mut problems = Vec::new();
        if stored.method != Method::SelfEnrol {
            problems.push(FieldError::new("key", "self_only"));
        }
        if key.is_some_and(|k| k.chars().count() < MIN_KEY) {
            problems.push(FieldError::new("key", "length"));
        }
        if let Some(group) = group
            && self.courses.group(group).await?.course != course.id
        {
            problems.push(FieldError::new("group", "course"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("enrol.key_invalid", problems));
        }
        let hash = match key {
            Some(k) => Some(self.hasher.hash(k).await?),
            None => None,
        };
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where instance_id = $1 and group_id is not distinct from $2",
            self.table("self_key")
        )))
        .bind(id.as_uuid())
        .bind(group.map(|g| g.as_uuid()))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(hash) = hash {
            self.insert_key_in(&mut tx, id, group, &hash).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Deletes an instance with its enrolments, their grants and the group memberships it owns; the
    /// records and progress of the people stay.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `enrol.instance.manage`, `precondition_failed` when it changed since it
    /// was read.
    pub async fn delete_instance(
        &self,
        actor: &Actor,
        id: InstanceId,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let stored = self.instance(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, INSTANCE_MANAGE, scope).await?;
        check_version(expected_version, stored.version, "enrol.instance_changed")?;
        let owned = self.owned_memberships(id, None).await?;
        let mut tx = self.db.begin().await?;
        self.read_instance_in(&mut tx, id, true).await?;
        let enrolments: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select id from {} where instance_id = $1",
            self.table("enrolment")
        )))
        .bind(id.as_uuid())
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for enrolment in enrolments {
            self.revoke_grants_in(&mut tx, crate::EnrolmentId::from_uuid(enrolment))
                .await?;
        }
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("instance")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.release_memberships(&owned).await
    }

    /// Reads an instance, locked for a change when `lock`.
    pub(crate) async fn read_instance_in(
        &self,
        conn: &mut PgConnection,
        id: InstanceId,
        lock: bool,
    ) -> Result<Instance> {
        let lock = if lock { " for update of i" } else { "" };
        let row = sqlx::query(sql(format!(
            "select {} from {} i where i.id = $1{lock}",
            columns(self),
            self.table("instance")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("enrol.instance_not_found"))?;
        instance(&row)
    }

    /// Stores the hash of a key.
    async fn insert_key_in(
        &self,
        conn: &mut PgConnection,
        id: InstanceId,
        group: Option<GroupId>,
        hash: &str,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, instance_id, group_id, hash) values ($1, $2, $3, $4)",
            self.table("self_key")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(id.as_uuid())
        .bind(group.map(|g| g.as_uuid()))
        .bind(hash)
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }
}
