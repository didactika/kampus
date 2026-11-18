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

//! Grants (UC-001): who holds which role where. Every change is recorded (RN-015) and never leaves a
//! campus unmanaged (RN-008).

use kampus_common::{Error, FieldError, Result, Timestamp};
use serde_json::json;
use sqlx::{PgConnection, Row};

use crate::permissions::{GRANT_CREATE, GRANT_DELETE};
use crate::service::role::principal;
use crate::{Access, Actor, AllowanceKind, GrantId, PrincipalId, RoleId, ScopeId, sql};

/// Who a new grant comes from.
enum Origin<'a> {
    /// A person, with the permission and the allowance (RN-013).
    Manual(&'a Actor),
    /// A module that owns the grant, such as an enrolment (RN-016).
    Module(&'a str),
}

impl Access {
    /// Grants a role to a principal in a scope, from now until `until` (RN-013).
    ///
    /// # Errors
    ///
    /// `permission_denied` without `access.grant.create` or an allowance for the role, `validation` when
    /// the role cannot be granted in this kind of scope, `conflict` for an existing manual grant.
    pub async fn grant(
        &self,
        actor: &Actor,
        role: RoleId,
        scope: ScopeId,
        to: PrincipalId,
        until: Option<Timestamp>,
    ) -> Result<GrantId> {
        self.insert_grant(Origin::Manual(actor), role, scope, to, until)
            .await
    }

    /// Grants a role on behalf of a module, which owns the grant and is the only one to remove it (RN-016).
    ///
    /// # Errors
    ///
    /// `validation` when the role cannot be granted in this kind of scope.
    pub async fn grant_for_module(
        &self,
        module: &str,
        role: RoleId,
        scope: ScopeId,
        to: PrincipalId,
        until: Option<Timestamp>,
    ) -> Result<GrantId> {
        self.insert_grant(Origin::Module(module), role, scope, to, until)
            .await
    }

    /// Grants a role on behalf of a module inside the caller's transaction, so the grant comes and goes
    /// with the change that causes it.
    ///
    /// # Errors
    ///
    /// As [`Access::grant_for_module`].
    pub async fn grant_for_module_in(
        &self,
        conn: &mut PgConnection,
        module: &str,
        role: RoleId,
        scope: ScopeId,
        to: PrincipalId,
        until: Option<Timestamp>,
    ) -> Result<GrantId> {
        self.insert_grant_in(conn, Origin::Module(module), role, scope, to, until)
            .await
    }

    /// The role of a campus made from a template, the oldest when there are several.
    ///
    /// # Errors
    ///
    /// `not_found` when the campus has no role from that template.
    pub async fn campus_role_in(
        &self,
        conn: &mut PgConnection,
        campus: ScopeId,
        template: &str,
    ) -> Result<RoleId> {
        sqlx::query_scalar(sql(format!(
            "select r.id from {} r join {} t on t.id = r.template_id where r.campus_id = $1 and t.code = $2
             order by r.created_at, r.id limit 1",
            self.table("role"),
            self.table("role_template")
        )))
        .bind(campus.as_uuid())
        .bind(template)
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .map(RoleId::from_uuid)
        .ok_or_else(|| Error::not_found("access.role_not_found").with_param("template", template))
    }

    /// Inserts a grant after its checks.
    async fn insert_grant(
        &self,
        origin: Origin<'_>,
        role: RoleId,
        scope: ScopeId,
        to: PrincipalId,
        until: Option<Timestamp>,
    ) -> Result<GrantId> {
        let mut tx = self.db.begin().await?;
        let id = self
            .insert_grant_in(&mut tx, origin, role, scope, to, until)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Inserts a grant after its checks, inside the caller's transaction.
    async fn insert_grant_in(
        &self,
        tx: &mut PgConnection,
        origin: Origin<'_>,
        role: RoleId,
        scope: ScopeId,
        to: PrincipalId,
        until: Option<Timestamp>,
    ) -> Result<GrantId> {
        let path = self.path(&mut *tx, scope).await?;
        let kind = path[0].kind;
        let (origin_text, module, by) = match origin {
            Origin::Manual(actor) => {
                self.require_with_allowance(&mut *tx, actor, GRANT_CREATE, scope, role, AllowanceKind::Grant)
                    .await?;
                ("manual", None, principal(actor))
            }
            Origin::Module(module) => ("module", Some(module), None),
        };
        let assignable: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where role_id = $1 and kind = $2)",
            self.table("role_scope_kind")
        )))
        .bind(role.as_uuid())
        .bind(kind.as_str())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if !assignable {
            return Err(Error::validation(
                "access.role_not_assignable",
                vec![FieldError::new("scope", "kind")],
            )
            .with_param("kind", kind.as_str()));
        }
        let role_campus = self.role_campus(&mut *tx, role).await?;
        if !path.iter().any(|s| s.id == role_campus) {
            return Err(Error::validation(
                "access.role_of_another_campus",
                vec![FieldError::new("role", "campus")],
            ));
        }
        let id = GrantId::generate();
        let from = self.now();
        sqlx::query(sql(format!(
            "insert into {} (id, role_id, scope_id, principal_id, origin, origin_module, valid_from, valid_until, granted_by)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            self.table("grant")
        )))
        .bind(id.as_uuid())
        .bind(role.as_uuid())
        .bind(scope.as_uuid())
        .bind(to.as_uuid())
        .bind(origin_text)
        .bind(module)
        .bind(from.as_offset_date_time())
        .bind(until.map(|u| u.as_offset_date_time()))
        .bind(by.map(|b| b.as_uuid()))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let after =
            json!({ "role": role, "scope": scope, "principal": to, "origin": origin_text, "module": module });
        self.record(&mut *tx, by, "grant", &id.to_string(), None, Some(after))
            .await?;
        Ok(id)
    }

    /// Removes a manual grant: needs `access.grant.delete`; grants owned by a module are removed only by
    /// it (RN-016); refused when it leaves the campus unmanaged (RN-008).
    ///
    /// # Errors
    ///
    /// `permission_denied`, `conflict` for a module's grant or for RN-008, `not_found` for an unknown grant.
    pub async fn revoke(&self, actor: &Actor, grant: GrantId) -> Result<()> {
        self.remove_grant(Some(actor), None, grant).await
    }

    /// Removes a grant a module owns.
    ///
    /// # Errors
    ///
    /// `conflict` when another module or a person owns it, or for RN-008.
    pub async fn revoke_for_module(&self, module: &str, grant: GrantId) -> Result<()> {
        self.remove_grant(None, Some(module), grant).await
    }

    /// Removes a grant after its checks.
    async fn remove_grant(&self, actor: Option<&Actor>, module: Option<&str>, grant: GrantId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "select scope_id, origin_module, role_id, principal_id from {} where id = $1",
            self.table("grant")
        )))
        .bind(grant.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("access.grant_not_found"))?;
        let scope = ScopeId::from_uuid(row.get("scope_id"));
        let owner: Option<String> = row.get("origin_module");
        match (actor, module) {
            (Some(actor), _) => {
                if owner.is_some() {
                    return Err(Error::conflict("access.grant_owned_by_module")
                        .with_param("module", owner.unwrap_or_default()));
                }
                if !self
                    .evaluate(&mut tx, actor, GRANT_DELETE, scope, false)
                    .await?
                    .allowed
                {
                    return Err(
                        Error::permission_denied("access.denied").with_param("permission", GRANT_DELETE)
                    );
                }
            }
            (None, module) => {
                if owner.as_deref() != module {
                    return Err(Error::conflict("access.grant_not_owned")
                        .with_param("module", module.unwrap_or_default()));
                }
            }
        }
        let campus = self.campus_of(&mut tx, scope).await?;
        let was_managed = self.managed_before(&mut tx, campus).await?;
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("grant"))))
            .bind(grant.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        self.ensure_still_managed(&mut tx, campus, was_managed).await?;
        let before = json!({ "role": RoleId::from_uuid(row.get("role_id")), "scope": scope, "principal": PrincipalId::from_uuid(row.get("principal_id")) });
        self.record(
            &mut tx,
            actor.and_then(principal),
            "grant",
            &grant.to_string(),
            Some(before),
            None,
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Requires a permission and an allowance of one of the actor's roles over the target role.
    pub(crate) async fn require_with_allowance(
        &self,
        conn: &mut PgConnection,
        actor: &Actor,
        permission: &str,
        scope: ScopeId,
        target: RoleId,
        kind: AllowanceKind,
    ) -> Result<()> {
        let decision = self.evaluate(conn, actor, permission, scope, false).await?;
        if !decision.allowed {
            return Err(Error::permission_denied("access.denied")
                .with_param("permission", permission)
                .with_param("rule", decision.rule));
        }
        let roles: Vec<uuid::Uuid> = decision.grants.iter().map(|g| g.role.as_uuid()).collect();
        let allowed: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where role_id = any($1) and target_role_id = $2 and kind = $3)",
            self.table("role_allowance")
        )))
        .bind(&roles)
        .bind(target.as_uuid())
        .bind(kind.as_str())
        .fetch_one(conn)
        .await
        .map_err(kampus_db::classify)?;
        if allowed {
            Ok(())
        } else {
            Err(Error::permission_denied("access.no_allowance").with_param("kind", kind.as_str()))
        }
    }

    /// Refuses a change that leaves a campus, managed before it, without an active principal able to grant
    /// roles in it (RN-008).
    pub(crate) async fn ensure_still_managed(
        &self,
        conn: &mut PgConnection,
        campus: Option<ScopeId>,
        was_managed: bool,
    ) -> Result<()> {
        match campus {
            Some(campus) if was_managed && !self.managed(conn, campus).await? => {
                Err(Error::conflict("access.campus_unmanaged").with_param("campus", campus.to_string()))
            }
            _ => Ok(()),
        }
    }

    /// Whether a campus has an active principal able to grant roles in it.
    pub(crate) async fn managed_before(
        &self,
        conn: &mut PgConnection,
        campus: Option<ScopeId>,
    ) -> Result<bool> {
        match campus {
            Some(campus) => self.managed(conn, campus).await,
            None => Ok(false),
        }
    }

    /// Whether a campus has an active principal able to grant roles in it.
    async fn managed(&self, conn: &mut PgConnection, campus: ScopeId) -> Result<bool> {
        let path = self.path(conn, campus).await?;
        let ids: Vec<uuid::Uuid> = path.iter().map(|s| s.id.as_uuid()).collect();
        let candidates: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct g.principal_id from {} g join {} p on p.id = g.principal_id
             where g.scope_id = any($1) and p.active and g.valid_from <= $2 and (g.valid_until is null or g.valid_until > $2)",
            self.table("grant"),
            self.table("principal")
        )))
        .bind(&ids)
        .bind(self.now().as_offset_date_time())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for candidate in candidates {
            if self
                .evaluate(
                    conn,
                    &Actor::Principal(PrincipalId::from_uuid(candidate)),
                    GRANT_CREATE,
                    campus,
                    true,
                )
                .await?
                .allowed
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
