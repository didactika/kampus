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

//! Roles: made from a template version, with the scope kinds where they can be granted and the
//! roles they may grant, override or view as. A role can be compared with its template and reset to it.

use std::collections::BTreeMap;

use kampus_common::{Error, Result};
use serde::Serialize;
use serde_json::json;
use sqlx::{PgConnection, Row};

use crate::permissions::ROLE_MANAGE;
use crate::{Access, Actor, AllowanceKind, PrincipalId, RoleId, ScopeId, ScopeKind, Value, sql};

/// A permission whose value differs between a role and its template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleDifference {
    /// The permission.
    pub permission: String,
    /// The role's value, if any.
    pub role: Option<Value>,
    /// The template's value, if any.
    pub template: Option<Value>,
}

/// A default role: its template, the kinds where it can be granted, and the roles it may grant, override and view as.
type DefaultRole = (
    &'static str,
    &'static [ScopeKind],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
);

/// The default roles of a campus.
const DEFAULT_ROLES: &[DefaultRole] = &[
    (
        "campus_administrator",
        &[ScopeKind::Campus],
        &[
            "campus_administrator",
            "manager",
            "course_creator",
            "teacher",
            "non_editing_teacher",
            "section_leader",
            "learner",
            "mentor",
        ],
        &[
            "manager",
            "course_creator",
            "teacher",
            "non_editing_teacher",
            "section_leader",
            "learner",
            "mentor",
            "authenticated_user",
            "guest",
        ],
        &["manager", "teacher", "non_editing_teacher", "learner", "guest"],
    ),
    (
        "manager",
        &[ScopeKind::Campus, ScopeKind::Category],
        &[
            "course_creator",
            "teacher",
            "non_editing_teacher",
            "section_leader",
            "learner",
            "mentor",
        ],
        &[
            "teacher",
            "non_editing_teacher",
            "section_leader",
            "learner",
            "guest",
        ],
        &["teacher", "non_editing_teacher", "learner", "guest"],
    ),
    ("course_creator", &[ScopeKind::Category], &[], &[], &[]),
    (
        "teacher",
        &[ScopeKind::Course, ScopeKind::Section],
        &["non_editing_teacher", "section_leader", "learner"],
        &["learner"],
        &["learner", "non_editing_teacher"],
    ),
    (
        "non_editing_teacher",
        &[ScopeKind::Course, ScopeKind::Section],
        &[],
        &[],
        &[],
    ),
    ("section_leader", &[ScopeKind::Section], &["learner"], &[], &[]),
    ("learner", &[ScopeKind::Course, ScopeKind::Section], &[], &[], &[]),
    ("mentor", &[ScopeKind::User], &[], &[], &[]),
    ("authenticated_user", &[ScopeKind::Campus], &[], &[], &[]),
    ("guest", &[ScopeKind::Course], &[], &[], &[]),
];

impl Access {
    /// Creates the default roles of a new campus from the latest version of their templates, with their
    /// allowances. It is part of creating a campus, not an action of a person.
    ///
    /// # Errors
    ///
    /// A `not_found` error when a template is not synchronised yet.
    pub async fn install_campus_roles(&self, campus: ScopeId) -> Result<BTreeMap<&'static str, RoleId>> {
        let mut tx = self.db.begin().await?;
        let mut roles = BTreeMap::new();
        for (code, kinds, _, _, _) in DEFAULT_ROLES {
            roles.insert(*code, self.insert_role(&mut tx, campus, code, code, kinds).await?);
        }
        for (code, _, grant, overrides, view_as) in DEFAULT_ROLES {
            for (kind, targets) in [
                (AllowanceKind::Grant, *grant),
                (AllowanceKind::Override, *overrides),
                (AllowanceKind::ViewAs, *view_as),
            ] {
                for target in targets {
                    self.insert_allowance(&mut tx, roles[code], roles[target], kind)
                        .await?;
                }
            }
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(roles)
    }

    /// Creates a role of a campus from the latest version of a template.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.role.manage`, `not_found` for an unknown template, `conflict`
    /// for a short name already used in the campus.
    pub async fn create_role(
        &self,
        actor: &Actor,
        campus: ScopeId,
        short_name: &str,
        template: &str,
        kinds: &[ScopeKind],
    ) -> Result<RoleId> {
        self.require(actor, ROLE_MANAGE, campus).await?;
        let mut tx = self.db.begin().await?;
        let role = self
            .insert_role(&mut tx, campus, &short_name.to_lowercase(), template, kinds)
            .await?;
        self.record(
            &mut tx,
            principal(actor),
            "role",
            &role.to_string(),
            None,
            Some(json!({ "template": template, "short_name": short_name })),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(role)
    }

    /// Inserts a role with the values of the latest version of its template.
    async fn insert_role(
        &self,
        conn: &mut PgConnection,
        campus: ScopeId,
        short_name: &str,
        template: &str,
        kinds: &[ScopeKind],
    ) -> Result<RoleId> {
        let template_id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select id from {} where code = $1 order by version desc limit 1",
            self.table("role_template")
        )))
        .bind(template)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("access.template_not_found").with_param("template", template))?;
        let id = RoleId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, short_name, template_id) values ($1, $2, $3, $4)",
            self.table("role")
        )))
        .bind(id.as_uuid())
        .bind(campus.as_uuid())
        .bind(short_name)
        .bind(template_id)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {} (role_id, permission_code, value) select $1, permission_code, value from {} where template_id = $2",
            self.table("role_permission"),
            self.table("role_template_value")
        )))
        .bind(id.as_uuid())
        .bind(template_id)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for kind in kinds {
            sqlx::query(sql(format!(
                "insert into {} (role_id, kind) values ($1, $2)",
                self.table("role_scope_kind")
            )))
            .bind(id.as_uuid())
            .bind(kind.as_str())
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(id)
    }

    /// Inserts an allowance between two roles.
    async fn insert_allowance(
        &self,
        conn: &mut PgConnection,
        role: RoleId,
        target: RoleId,
        kind: AllowanceKind,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (role_id, target_role_id, kind) values ($1, $2, $3) on conflict do nothing",
            self.table("role_allowance")
        )))
        .bind(role.as_uuid())
        .bind(target.as_uuid())
        .bind(kind.as_str())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Gives a role a value for a permission, or removes its value with `None`; refused when it would
    /// leave the campus without anyone able to manage access. Recorded.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.role.manage`, `conflict` when it would leave the campus without anyone to manage it.
    pub async fn set_role_value(
        &self,
        actor: &Actor,
        role: RoleId,
        permission: &str,
        value: Option<Value>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let campus = self.role_campus(&mut tx, role).await?;
        if !self
            .evaluate(&mut tx, actor, ROLE_MANAGE, campus, false)
            .await?
            .allowed
        {
            return Err(Error::permission_denied("access.denied").with_param("permission", ROLE_MANAGE));
        }
        let before = self.role_value(&mut tx, role, permission).await?;
        let was_managed = self.managed_before(&mut tx, Some(campus)).await?;
        let table = self.table("role_permission");
        match value {
            Some(value) => sqlx::query(sql(format!(
                "insert into {table} (role_id, permission_code, value) values ($1, $2, $3)
                 on conflict (role_id, permission_code) do update set value = excluded.value"
            )))
            .bind(role.as_uuid())
            .bind(permission)
            .bind(value.as_str()),
            None => sqlx::query(sql(format!(
                "delete from {table} where role_id = $1 and permission_code = $2"
            )))
            .bind(role.as_uuid())
            .bind(permission),
        }
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.ensure_still_managed(&mut tx, Some(campus), was_managed)
            .await?;
        let key = format!("{role}/{permission}");
        self.record(
            &mut tx,
            principal(actor),
            "role_permission",
            &key,
            before.map(|v| json!(v)),
            value.map(|v| json!(v)),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Lets one role grant, override or view as another, or stops it.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.role.manage`.
    pub async fn set_allowance(
        &self,
        actor: &Actor,
        role: RoleId,
        target: RoleId,
        kind: AllowanceKind,
        allowed: bool,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let campus = self.role_campus(&mut tx, role).await?;
        if !self
            .evaluate(&mut tx, actor, ROLE_MANAGE, campus, false)
            .await?
            .allowed
        {
            return Err(Error::permission_denied("access.denied").with_param("permission", ROLE_MANAGE));
        }
        if allowed {
            self.insert_allowance(&mut tx, role, target, kind).await?;
        } else {
            sqlx::query(sql(format!(
                "delete from {} where role_id = $1 and target_role_id = $2 and kind = $3",
                self.table("role_allowance")
            )))
            .bind(role.as_uuid())
            .bind(target.as_uuid())
            .bind(kind.as_str())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        self.record(
            &mut tx,
            principal(actor),
            "role_allowance",
            &format!("{role}/{target}/{}", kind.as_str()),
            None,
            Some(json!(allowed)),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The permissions whose value differs between a role and the latest version of its template.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown role.
    pub async fn template_differences(&self, role: RoleId) -> Result<Vec<RoleDifference>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.differences(&mut conn, role).await
    }

    /// Makes a role require a second factor from the people who hold it, or not; needs
    /// `access.role.manage` in the role's campus. The change is recorded.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `not_found` for an unknown role.
    pub async fn set_role_second_factor(&self, actor: &Actor, role: RoleId, required: bool) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let campus = self.role_campus(&mut tx, role).await?;
        if !self
            .evaluate(&mut tx, actor, ROLE_MANAGE, campus, false)
            .await?
            .allowed
        {
            return Err(Error::permission_denied("access.denied").with_param("permission", ROLE_MANAGE));
        }
        let before: bool = sqlx::query_scalar(sql(format!(
            "update {} r set second_factor_required = $2, version = r.version + 1 from {} o
             where r.id = $1 and o.id = r.id returning o.second_factor_required",
            self.table("role"),
            self.table("role")
        )))
        .bind(role.as_uuid())
        .bind(required)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.record(
            &mut tx,
            principal(actor),
            "role_second_factor",
            &role.to_string(),
            Some(json!({ "required": before })),
            Some(json!({ "required": required })),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Whether a principal holds, now, a role that requires a second factor.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn second_factor_required(&self, principal: PrincipalId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} g join {} r on r.id = g.role_id
             where g.principal_id = $1 and r.second_factor_required
             and g.valid_from <= $2 and (g.valid_until is null or g.valid_until > $2))",
            self.table("grant"),
            self.table("role")
        )))
        .bind(principal.as_uuid())
        .bind(self.now().as_offset_date_time())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    /// Resets a role to the latest version of its template: without `confirm` it only shows the
    /// differences; with it, it applies them.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.role.manage`, `conflict` when it would leave the campus without anyone to manage it.
    pub async fn reset_to_template(
        &self,
        actor: &Actor,
        role: RoleId,
        confirm: bool,
    ) -> Result<Vec<RoleDifference>> {
        let mut tx = self.db.begin().await?;
        let campus = self.role_campus(&mut tx, role).await?;
        if !self
            .evaluate(&mut tx, actor, ROLE_MANAGE, campus, false)
            .await?
            .allowed
        {
            return Err(Error::permission_denied("access.denied").with_param("permission", ROLE_MANAGE));
        }
        let differences = self.differences(&mut tx, role).await?;
        if !confirm {
            return Ok(differences);
        }
        let was_managed = self.managed_before(&mut tx, Some(campus)).await?;
        let table = self.table("role_permission");
        for difference in &differences {
            match difference.template {
                Some(value) => sqlx::query(sql(format!(
                    "insert into {table} (role_id, permission_code, value) values ($1, $2, $3)
                     on conflict (role_id, permission_code) do update set value = excluded.value"
                )))
                .bind(role.as_uuid())
                .bind(&difference.permission)
                .bind(value.as_str()),
                None => sqlx::query(sql(format!(
                    "delete from {table} where role_id = $1 and permission_code = $2"
                )))
                .bind(role.as_uuid())
                .bind(&difference.permission),
            }
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        self.ensure_still_managed(&mut tx, Some(campus), was_managed)
            .await?;
        self.record(
            &mut tx,
            principal(actor),
            "role",
            &role.to_string(),
            Some(json!(differences)),
            Some(json!("reset to template")),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(differences)
    }

    /// The differences between a role and its template, on a connection.
    async fn differences(&self, conn: &mut PgConnection, role: RoleId) -> Result<Vec<RoleDifference>> {
        let rows = sqlx::query(sql(format!(
            "with latest as (
                 select t2.id from {role} r join {template} t on t.id = r.template_id
                 join {template} t2 on t2.code = t.code where r.id = $1 order by t2.version desc limit 1
             )
             select coalesce(p.permission_code, v.permission_code) as permission, p.value as role_value, v.value as template_value
             from (select permission_code, value from {permission} where role_id = $1) p
             full join (select permission_code, value from {value} where template_id = (select id from latest)) v
               on v.permission_code = p.permission_code
             where p.value is distinct from v.value
             order by 1",
            role = self.table("role"),
            template = self.table("role_template"),
            permission = self.table("role_permission"),
            value = self.table("role_template_value"),
        )))
        .bind(role.as_uuid())
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        rows.iter()
            .map(|row| {
                Ok(RoleDifference {
                    permission: row.get("permission"),
                    role: row
                        .get::<Option<String>, _>("role_value")
                        .map(|v| v.parse())
                        .transpose()?,
                    template: row
                        .get::<Option<String>, _>("template_value")
                        .map(|v| v.parse())
                        .transpose()?,
                })
            })
            .collect()
    }

    /// The scope that owns a role (its campus).
    pub(crate) async fn role_campus(&self, conn: &mut PgConnection, role: RoleId) -> Result<ScopeId> {
        sqlx::query_scalar(sql(format!(
            "select campus_id from {} where id = $1",
            self.table("role")
        )))
        .bind(role.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .map(ScopeId::from_uuid)
        .ok_or_else(|| Error::not_found("access.role_not_found"))
    }

    /// The role's own value for a permission.
    async fn role_value(
        &self,
        conn: &mut PgConnection,
        role: RoleId,
        permission: &str,
    ) -> Result<Option<Value>> {
        let value: Option<String> = sqlx::query_scalar(sql(format!(
            "select value from {} where role_id = $1 and permission_code = $2",
            self.table("role_permission")
        )))
        .bind(role.as_uuid())
        .bind(permission)
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?;
        value.map(|v| v.parse()).transpose()
    }
}

/// The principal behind an actor, for the history.
pub(crate) const fn principal(actor: &Actor) -> Option<crate::PrincipalId> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(*p),
        Actor::Visitor(_) => None,
    }
}
