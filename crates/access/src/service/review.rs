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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What the administration reviews of access: the declared permissions, the roles of a campus with their
//! values, the grants and overrides set in a scope, and the explanation of a check. Each read needs
//! `access.review` in the scope, within what the token of the request carries.

use std::collections::BTreeMap;

use kampus_common::{Error, Result, Timestamp};
use serde::Serialize;
use sqlx::Row;

use crate::permissions::REVIEW;
use crate::{
    Access, Acting, Decision, GrantId, PermissionKind, PrincipalId, Risk, RoleId, ScopeId, ScopeKind, Value,
    sql,
};

/// A declared permission, as the matrix of roles lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct PermissionInfo {
    /// The code, `<module>.<area>.<action>`.
    pub code: String,
    /// The module that declares it.
    pub module: String,
    /// Whether it reads or changes something.
    pub kind: PermissionKind,
    /// What it can expose.
    pub risks: Vec<Risk>,
    /// The lowest kind of scope where it means something, and so where it may be overridden.
    pub lowest_kind: ScopeKind,
}

/// A role of a campus with the value it gives each permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct RoleView {
    /// The role.
    pub id: RoleId,
    /// Its short name, unique in its campus.
    pub short_name: String,
    /// The template it was made from.
    pub template: String,
    /// The kinds of scope where it may be granted.
    pub kinds: Vec<ScopeKind>,
    /// Whether whoever holds it must sign in with a second factor.
    pub second_factor_required: bool,
    /// Its version.
    pub version: i32,
    /// The value it gives each permission it names; a permission it does not name is not allowed.
    pub values: BTreeMap<String, Value>,
}

/// A role granted to a principal in a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct GrantView {
    /// The grant.
    pub id: GrantId,
    /// The role.
    pub role: RoleId,
    /// The scope where it is held, and so in every scope below.
    pub scope: ScopeId,
    /// The kind of that scope.
    pub scope_kind: ScopeKind,
    /// Who holds it.
    pub principal: PrincipalId,
    /// How it came: by hand, or by a module (an enrolment, a rule of a group).
    pub origin: String,
    /// The module that gave it, when one did.
    pub origin_module: Option<String>,
    /// From when it counts.
    pub valid_from: Timestamp,
    /// Until when it counts, when it ends.
    pub valid_until: Option<Timestamp>,
}

/// The value a scope gives a permission for a role, in place of the value of the role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct OverrideView {
    /// The role.
    pub role: RoleId,
    /// The permission.
    pub permission: String,
    /// The value in the scope and below.
    pub value: Value,
    /// Its version.
    pub version: i32,
}

/// A word stored for an enum, read back as the enum.
fn word<T: serde::de::DeserializeOwned>(text: String) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(text))
        .map_err(|_| Error::internal("access.stored_value_invalid"))
}

impl Access {
    /// Fails unless the request may review access in a scope.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.review` there or beyond what the token carries.
    pub async fn require_review(&self, acting: &Acting, scope: ScopeId) -> Result<()> {
        let actor = self.acting(acting, REVIEW, scope).await?;
        self.require(actor, REVIEW, scope).await
    }

    /// Every declared permission, by code.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.review` in the scope.
    pub async fn permissions(&self, acting: &Acting, scope: ScopeId) -> Result<Vec<PermissionInfo>> {
        self.require_review(acting, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select code, module, kind, risks, lowest_kind from {} where retired_at is null order by code",
            self.table("permission")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.into_iter()
            .map(|r| {
                Ok(PermissionInfo {
                    code: r.get("code"),
                    module: r.get("module"),
                    kind: word(r.get("kind"))?,
                    risks: r
                        .get::<Vec<String>, _>("risks")
                        .into_iter()
                        .map(word)
                        .collect::<Result<_>>()?,
                    lowest_kind: word(r.get("lowest_kind"))?,
                })
            })
            .collect()
    }

    /// The roles of the campus of a scope, by short name, with their values.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.review` in the scope, `not_found` for an unknown scope.
    pub async fn roles(&self, acting: &Acting, scope: ScopeId) -> Result<Vec<RoleView>> {
        self.require_review(acting, scope).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let Some(campus) = self.campus_of(&mut conn, scope).await? else {
            return Ok(Vec::new());
        };
        self.roles_where(&mut conn, "r.campus_id = $1", campus.as_uuid())
            .await
    }

    /// A role with its values.
    ///
    /// # Errors
    ///
    /// `not_found` (`access.role_not_found`) for an unknown role, `permission_denied` without
    /// `access.review` in its campus.
    pub async fn role(&self, acting: &Acting, role: RoleId) -> Result<RoleView> {
        let campus = self.campus_of_role(role).await?;
        self.require_review(acting, campus).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.roles_where(&mut conn, "r.id = $1", role.as_uuid())
            .await?
            .pop()
            .ok_or_else(|| Error::not_found("access.role_not_found"))
    }

    /// The campus of a role.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown role.
    pub async fn campus_of_role(&self, role: RoleId) -> Result<ScopeId> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.role_campus(&mut conn, role).await
    }

    /// A scope and the scopes above it, nearest first.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn path_of(&self, scope: ScopeId) -> Result<Vec<crate::PathScope>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.path(&mut conn, scope).await
    }

    /// The scope where a grant is held.
    ///
    /// # Errors
    ///
    /// `not_found` (`access.grant_not_found`) for an unknown grant.
    pub async fn grant_scope(&self, grant: GrantId) -> Result<ScopeId> {
        sqlx::query_scalar::<_, uuid::Uuid>(sql(format!(
            "select scope_id from {} where id = $1",
            self.table("grant")
        )))
        .bind(grant.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .map(ScopeId::from_uuid)
        .ok_or_else(|| Error::not_found("access.grant_not_found"))
    }

    /// The roles a condition selects, with their kinds and values.
    async fn roles_where(
        &self,
        conn: &mut sqlx::PgConnection,
        condition: &str,
        argument: uuid::Uuid,
    ) -> Result<Vec<RoleView>> {
        let rows = sqlx::query(sql(format!(
            "select r.id, r.short_name, t.code as template, r.second_factor_required, r.version,
                    coalesce((select array_agg(k.kind order by k.kind) from {kinds} k where k.role_id = r.id), '{{}}') as kinds
             from {roles} r join {templates} t on t.id = r.template_id where {condition} order by r.short_name",
            roles = self.table("role"),
            templates = self.table("role_template"),
            kinds = self.table("role_scope_kind"),
        )))
        .bind(argument)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.get("id")).collect();
        let values = sqlx::query(sql(format!(
            "select role_id, permission_code, value from {} where role_id = any($1)",
            self.table("role_permission")
        )))
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut by_role: BTreeMap<uuid::Uuid, BTreeMap<String, Value>> = BTreeMap::new();
        for row in values {
            by_role
                .entry(row.get("role_id"))
                .or_default()
                .insert(row.get("permission_code"), row.get::<String, _>("value").parse()?);
        }
        rows.into_iter()
            .map(|r| {
                let id: uuid::Uuid = r.get("id");
                Ok(RoleView {
                    id: RoleId::from_uuid(id),
                    short_name: r.get("short_name"),
                    template: r.get("template"),
                    kinds: r
                        .get::<Vec<String>, _>("kinds")
                        .into_iter()
                        .map(word)
                        .collect::<Result<_>>()?,
                    second_factor_required: r.get("second_factor_required"),
                    version: r.get("version"),
                    values: by_role.remove(&id).unwrap_or_default(),
                })
            })
            .collect()
    }

    /// The grants held in a scope itself, newest first.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.review` in the scope.
    pub async fn grants_in(&self, acting: &Acting, scope: ScopeId) -> Result<Vec<GrantView>> {
        self.require_review(acting, scope).await?;
        self.grants_where("g.scope_id = $1", scope.as_uuid()).await
    }

    /// The grants a principal holds, in every scope, newest first.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown principal, `permission_denied` without `access.review` in its campus.
    pub async fn grants_of(&self, acting: &Acting, principal: PrincipalId) -> Result<Vec<GrantView>> {
        self.require_review(acting, self.principal_campus(principal).await?)
            .await?;
        self.grants_where("g.principal_id = $1", principal.as_uuid())
            .await
    }

    /// The grants a condition selects, with the kind of their scope.
    async fn grants_where(&self, condition: &str, argument: uuid::Uuid) -> Result<Vec<GrantView>> {
        let rows = sqlx::query(sql(format!(
            "select g.id, g.role_id, g.scope_id, s.kind as scope_kind, g.principal_id, g.origin, g.origin_module,
                    g.valid_from, g.valid_until
             from {} g join {} s on s.id = g.scope_id where {condition} order by g.created_at desc, g.id desc",
            self.table("grant"),
            self.table("scope")
        )))
        .bind(argument)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.iter()
            .map(|r| {
                Ok(GrantView {
                    id: GrantId::from_uuid(r.get("id")),
                    role: RoleId::from_uuid(r.get("role_id")),
                    scope: ScopeId::from_uuid(r.get("scope_id")),
                    scope_kind: word(r.get("scope_kind"))?,
                    principal: PrincipalId::from_uuid(r.get("principal_id")),
                    origin: r.get("origin"),
                    origin_module: r.get("origin_module"),
                    valid_from: Timestamp::from_offset_date_time(r.get("valid_from")),
                    valid_until: r
                        .get::<Option<time::OffsetDateTime>, _>("valid_until")
                        .map(Timestamp::from_offset_date_time),
                })
            })
            .collect()
    }

    /// The overrides set in a scope itself.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.review` in the scope.
    pub async fn overrides_in(&self, acting: &Acting, scope: ScopeId) -> Result<Vec<OverrideView>> {
        self.require_review(acting, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select role_id, permission_code, value, version from {} where scope_id = $1 order by permission_code, role_id",
            self.table("permission_override")
        )))
        .bind(scope.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.iter()
            .map(|r| {
                Ok(OverrideView {
                    role: RoleId::from_uuid(r.get("role_id")),
                    permission: r.get("permission_code"),
                    value: r.get::<String, _>("value").parse()?,
                    version: r.get("version"),
                })
            })
            .collect()
    }

    /// Why a principal may or may not use a permission in a scope: the decision of the check, with every
    /// role it considered.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.review` in the scope, `not_found` for an unknown scope.
    pub async fn explain(
        &self,
        acting: &Acting,
        principal: PrincipalId,
        permission: &str,
        scope: ScopeId,
    ) -> Result<Decision> {
        self.require_review(acting, scope).await?;
        self.check(&crate::Actor::Principal(principal), permission, scope)
            .await
    }
}
