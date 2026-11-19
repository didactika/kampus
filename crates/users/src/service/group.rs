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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! System groups: people of a campus or a category, for enrolment and reports. Members are added by hand,
//! by the group's rule over profiles, or by a module that synchronises the group; each membership keeps
//! its origin, and a rule or a sync removes only the memberships it made.

use std::collections::BTreeSet;

use kampus_access::{Actor, ScopeId, ScopeKind};
use kampus_common::{Error, FieldError, Result};
use kampus_identity::AccountId;
use sqlx::Row;

use crate::permissions::GROUP_MANAGE;
use crate::{GroupId, GroupKind, GroupRule, Member, Origin, SystemGroup, Users, sql};

impl Users {
    /// Creates a system group in a campus or a category; needs `users.system_group.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for another kind of scope or an empty
    /// code or name, `conflict` when the scope has a group with that code.
    pub async fn create_group(
        &self,
        actor: &Actor,
        scope: ScopeId,
        code: &str,
        name: &str,
        kind: GroupKind,
    ) -> Result<GroupId> {
        self.access.require(actor, GROUP_MANAGE, scope).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let path = self.access.path(&mut conn, scope).await?;
        drop(conn);
        let mut problems = Vec::new();
        if !matches!(path[0].kind, ScopeKind::Campus | ScopeKind::Category) {
            problems.push(FieldError::new("scope", "campus_or_category"));
        }
        let (code, name) = (code.trim(), name.trim());
        if code.is_empty() || code.chars().count() > 100 {
            problems.push(FieldError::new("code", "length"));
        }
        if name.is_empty() || name.chars().count() > 200 {
            problems.push(FieldError::new("name", "length"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("users.group_invalid", problems));
        }
        let (kind_text, rule) = match &kind {
            GroupKind::Manual => ("manual", None),
            GroupKind::Rule(rule) => (
                "rule",
                Some(
                    serde_json::to_value(rule)
                        .map_err(|e| Error::internal("users.rule_unwritable").with_source(e))?,
                ),
            ),
        };
        let id = GroupId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, scope_id, code, name, kind, rule) values ($1, $2, $3, $4, $5, $6)",
            self.table("system_group")
        )))
        .bind(id.as_uuid())
        .bind(scope.as_uuid())
        .bind(code)
        .bind(name)
        .bind(kind_text)
        .bind(rule)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(|error| {
            if error.code() == "db.unique_violation" {
                Error::conflict("users.group_exists")
            } else {
                error
            }
        })?;
        Ok(id)
    }

    /// A system group.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group.
    pub async fn group(&self, id: GroupId) -> Result<SystemGroup> {
        let row = sqlx::query(sql(format!(
            "select id, scope_id, code, name, kind, rule from {} where id = $1",
            self.table("system_group")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("users.group_not_found"))?;
        let kind = match row.get::<Option<serde_json::Value>, _>("rule") {
            Some(rule) => GroupKind::Rule(
                serde_json::from_value(rule)
                    .map_err(|e| Error::internal("users.rule_unreadable").with_source(e))?,
            ),
            None => GroupKind::Manual,
        };
        Ok(SystemGroup {
            id,
            scope: ScopeId::from_uuid(row.get("scope_id")),
            code: row.get("code"),
            name: row.get("name"),
            kind,
        })
    }

    /// Adds a person to a group by hand; the person must belong to the group's campus.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `users.system_group.manage` in the group's scope, `validation` for an
    /// account of another campus.
    pub async fn add_member(&self, actor: &Actor, group: GroupId, account: AccountId) -> Result<()> {
        let group = self.managed(actor, group).await?;
        if self.campus(account).await? != self.campus_of_scope(group.scope).await? {
            return Err(Error::validation(
                "users.member_of_another_campus",
                vec![FieldError::new("account", "campus")],
            ));
        }
        self.insert_members(group.id, &[account], Origin::Manual).await
    }

    /// Removes the membership a person added by hand; memberships of a rule or a sync stay.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `users.system_group.manage` in the group's scope.
    pub async fn remove_member(&self, actor: &Actor, group: GroupId, account: AccountId) -> Result<()> {
        let group = self.managed(actor, group).await?;
        let mut tx = self.db.begin().await?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where group_id = $1 and account_id = $2 and origin = 'manual'",
            self.table("system_group_member")
        )))
        .bind(group.id.as_uuid())
        .bind(account.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed > 0 {
            self.members_changed_in(&mut tx, group.id).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The accounts in a system group, whatever their origin, for a module that keeps something in step
    /// with it, such as enrolment.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group.
    pub async fn member_accounts(&self, group: GroupId) -> Result<Vec<AccountId>> {
        self.group(group).await?;
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct account_id from {} where group_id = $1 order by account_id",
            self.table("system_group_member")
        )))
        .bind(group.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(ids.into_iter().map(AccountId::from_uuid).collect())
    }

    /// The members of a group, once per origin.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `users.system_group.manage` in the group's scope.
    pub async fn members(&self, actor: &Actor, group: GroupId) -> Result<Vec<Member>> {
        let group = self.managed(actor, group).await?;
        let rows = sqlx::query(sql(format!(
            "select account_id, origin from {} where group_id = $1 order by account_id, origin",
            self.table("system_group_member")
        )))
        .bind(group.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Member {
                account: AccountId::from_uuid(r.get("account_id")),
                origin: Origin::parse(r.get("origin")),
            })
            .collect())
    }

    /// Applies the group's rule: the people of its campus whose profile matches become members by rule,
    /// and those who no longer match stop being; hand-added members stay. Returns how many match.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for a group without a rule.
    pub async fn apply_rule(&self, actor: &Actor, group: GroupId) -> Result<usize> {
        let group = self.managed(actor, group).await?;
        let GroupKind::Rule(GroupRule { field, equals }) = &group.kind else {
            return Err(Error::validation(
                "users.group_without_rule",
                vec![FieldError::new("group", "rule")],
            ));
        };
        let campus = self.campus_of_scope(group.scope).await?;
        let matching: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select account_id from {} where campus_id = $1 and {} = $2",
            self.table("profile"),
            field.column()
        )))
        .bind(campus.as_uuid())
        .bind(equals)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let accounts: Vec<AccountId> = matching.into_iter().map(AccountId::from_uuid).collect();
        self.replace_members(group.id, Origin::Rule, &accounts).await?;
        Ok(accounts.len())
    }

    /// Makes the memberships of one origin exactly these accounts; the other origins stay. A module that
    /// synchronises a group calls it with [`Origin::Sync`].
    ///
    /// # Errors
    ///
    /// `validation` for the manual origin, which people manage, and `unavailable` when the database fails.
    pub async fn replace_members(
        &self,
        group: GroupId,
        origin: Origin,
        accounts: &[AccountId],
    ) -> Result<()> {
        if origin == Origin::Manual {
            return Err(Error::validation(
                "users.origin_manual",
                vec![FieldError::new("origin", "rule_or_sync")],
            ));
        }
        let wanted: Vec<uuid::Uuid> = accounts
            .iter()
            .map(AccountId::as_uuid)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let table = self.table("system_group_member");
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {table} where group_id = $1 and origin = $2 and not (account_id = any($3))"
        )))
        .bind(group.as_uuid())
        .bind(origin.as_str())
        .bind(&wanted)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {table} (group_id, account_id, origin) select $1, unnest($3::uuid[]), $2 on conflict do nothing"
        )))
        .bind(group.as_uuid())
        .bind(origin.as_str())
        .bind(&wanted)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.members_changed_in(&mut tx, group).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Adds memberships of one origin.
    async fn insert_members(&self, group: GroupId, accounts: &[AccountId], origin: Origin) -> Result<()> {
        let ids: Vec<uuid::Uuid> = accounts.iter().map(AccountId::as_uuid).collect();
        let mut tx = self.db.begin().await?;
        let added = sqlx::query(sql(format!(
            "insert into {} (group_id, account_id, origin) select $1, unnest($3::uuid[]), $2 on conflict do nothing",
            self.table("system_group_member")
        )))
        .bind(group.as_uuid())
        .bind(origin.as_str())
        .bind(&ids)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if added > 0 {
            self.members_changed_in(&mut tx, group).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Publishes that the members of a group changed, in the transaction of the change.
    async fn members_changed_in(&self, conn: &mut sqlx::PgConnection, group: GroupId) -> Result<()> {
        kampus_events::Events::new(self.db.clone())
            .publish_in(
                conn,
                kampus_events::Publish::new(
                    crate::events::MEMBERS_CHANGED,
                    Some(group.as_uuid()),
                    serde_json::json!({ "group": group }),
                ),
            )
            .await
            .map(drop)
    }

    /// A group, once the actor may manage it.
    async fn managed(&self, actor: &Actor, id: GroupId) -> Result<SystemGroup> {
        let group = self.group(id).await?;
        self.access.require(actor, GROUP_MANAGE, group.scope).await?;
        Ok(group)
    }

    /// The campus a scope belongs to.
    async fn campus_of_scope(&self, scope: ScopeId) -> Result<ScopeId> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let path = self.access.path(&mut conn, scope).await?;
        path.iter()
            .find(|s| s.kind == ScopeKind::Campus)
            .map(|s| s.id)
            .ok_or_else(|| {
                Error::validation(
                    "users.scope_without_campus",
                    vec![FieldError::new("scope", "campus")],
                )
            })
    }
}
