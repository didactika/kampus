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

//! The permission check (UC-003) and its explanation (RN-019).
//!
//! 1. An undeclared or retired permission fails (RN-021); an inactive principal fails (RN-010).
//! 2. A write in a frozen scope or below fails (RN-011); "view as" never writes and stays in its scope (RN-018).
//! 3. The active grants of the actor on the scope and its ancestors count (RN-003).
//! 4. For each granted role, the value is the first override from the scope up to its campus, otherwise
//!    the role's own value (RN-004); guests and visitors never get a write or a risky permission (RN-009).
//! 5. Any `forbid` fails (RN-005); otherwise one `allow` passes (RN-006).

use std::collections::HashMap;

use kampus_common::{Error, Result};
use serde::Serialize;
use sqlx::{PgConnection, Row};

use crate::model::PermissionKind;
use crate::permissions::GUEST_TEMPLATE;
use crate::service::scope::PathScope;
use crate::{Access, Actor, GrantId, RoleId, ScopeId, ScopeKind, Value, sql};

/// How one granted role was evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrantEvaluation {
    /// The grant, absent for the implicit roles of visitors and "view as".
    pub grant: Option<GrantId>,
    /// The role.
    pub role: RoleId,
    /// Where the role is held.
    pub scope: ScopeId,
    /// The value found, if any.
    pub value: Option<Value>,
    /// The scope of the override that gave the value; absent when it is the role's own value.
    pub set_at: Option<ScopeId>,
}

/// The result of a check, with its explanation (RN-019).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Decision {
    /// Whether the actor may.
    pub allowed: bool,
    /// The rule that decided (`RN-005`…).
    pub rule: &'static str,
    /// The permission checked.
    pub permission: String,
    /// The roles considered and their values.
    pub grants: Vec<GrantEvaluation>,
}

impl Decision {
    /// A decision that refuses without looking at any role.
    fn refuse(permission: &str, rule: &'static str) -> Self {
        Self {
            allowed: false,
            rule,
            permission: permission.to_owned(),
            grants: Vec::new(),
        }
    }
}

/// The declared facts of a permission.
struct Declared {
    /// Read or write.
    kind: PermissionKind,
    /// Whether it has any risk.
    risky: bool,
}

/// The values of a permission for the roles of a check.
struct RoleValues {
    /// For each role, the depth, scope and value of the nearest override.
    overrides: HashMap<uuid::Uuid, (i32, uuid::Uuid, Value)>,
    /// For each role, its own value.
    own: HashMap<uuid::Uuid, Value>,
    /// The roles made from the guest template.
    guests: Vec<uuid::Uuid>,
}

/// A role held by the actor for this check.
struct Held {
    /// The grant, if it is one.
    grant: Option<GrantId>,
    /// The role.
    role: RoleId,
    /// Where it is held.
    scope: ScopeId,
}

impl Access {
    /// Checks whether an actor may do something in a scope, and explains why.
    ///
    /// # Errors
    ///
    /// A `not_found` error for an unknown scope, an `unavailable` error when the database fails.
    pub async fn check(&self, actor: &Actor, permission: &str, scope: ScopeId) -> Result<Decision> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.evaluate(&mut conn, actor, permission, scope, false).await
    }

    /// Fails with `permission_denied` unless the actor may.
    ///
    /// # Errors
    ///
    /// `permission_denied` naming the permission and the deciding rule, or the errors of [`Access::check`].
    pub async fn require(&self, actor: &Actor, permission: &str, scope: ScopeId) -> Result<()> {
        let decision = self.check(actor, permission, scope).await?;
        if decision.allowed {
            Ok(())
        } else {
            Err(Error::permission_denied("access.denied")
                .with_param("permission", permission)
                .with_param("rule", decision.rule))
        }
    }

    /// Evaluates a check on a connection, so it can run inside the transaction of a change (RN-008).
    /// `ignore_frozen` lets unfreezing be checked as if the scope were open.
    pub(crate) async fn evaluate(
        &self,
        conn: &mut PgConnection,
        actor: &Actor,
        permission: &str,
        scope: ScopeId,
        ignore_frozen: bool,
    ) -> Result<Decision> {
        let Some(declared) = self.declared(conn, permission).await? else {
            tracing::warn!(permission, "check of an undeclared or retired permission");
            return Ok(Decision::refuse(permission, "RN-021"));
        };
        let path = self.path(conn, scope).await?;
        let Some(held) = self.held(conn, actor, &path, declared.kind).await? else {
            let rule = if matches!(actor, Actor::ViewAs(..)) {
                "RN-018"
            } else {
                "RN-010"
            };
            return Ok(Decision::refuse(permission, rule));
        };
        if !ignore_frozen && declared.kind == PermissionKind::Write && path.iter().any(|s| s.frozen) {
            return Ok(Decision::refuse(permission, "RN-011"));
        }

        let values = self.role_values(conn, permission, &held, &path).await?;
        let (overrides, own, guests) = (values.overrides, values.own, values.guests);

        let visitor = matches!(actor, Actor::Visitor(_));
        let limited = declared.kind == PermissionKind::Write || declared.risky;
        let grants: Vec<GrantEvaluation> = held
            .iter()
            .map(|h| {
                let key = h.role.as_uuid();
                let (value, set_at) = match overrides.get(&key) {
                    Some((_, scope, value)) => (Some(*value), Some(ScopeId::from_uuid(*scope))),
                    None => (own.get(&key).copied(), None),
                };
                let value = if limited && (visitor || guests.contains(&key)) {
                    None
                } else {
                    value
                };
                GrantEvaluation {
                    grant: h.grant,
                    role: h.role,
                    scope: h.scope,
                    value,
                    set_at,
                }
            })
            .collect();

        let (allowed, rule) = if grants.iter().any(|g| g.value == Some(Value::Forbid)) {
            (false, "RN-005")
        } else {
            (grants.iter().any(|g| g.value == Some(Value::Allow)), "RN-006")
        };
        let rule = if !allowed
            && limited
            && (visitor || grants.iter().any(|g| guests.contains(&g.role.as_uuid())))
            && rule == "RN-006"
        {
            "RN-009"
        } else {
            rule
        };
        Ok(Decision {
            allowed,
            rule,
            permission: permission.to_owned(),
            grants,
        })
    }

    /// The values of a permission for the held roles: the nearest override up to the campus, the roles'
    /// own values, and which roles are guest roles.
    async fn role_values(
        &self,
        conn: &mut PgConnection,
        permission: &str,
        held: &[Held],
        path: &[PathScope],
    ) -> Result<RoleValues> {
        let roles: Vec<uuid::Uuid> = held.iter().map(|h| h.role.as_uuid()).collect();
        let campus_depth = path
            .iter()
            .find(|s| s.kind == ScopeKind::Campus)
            .map_or(i32::MAX, |s| s.depth);
        let reachable: Vec<uuid::Uuid> = path
            .iter()
            .filter(|s| s.depth <= campus_depth)
            .map(|s| s.id.as_uuid())
            .collect();
        let depth_of: HashMap<uuid::Uuid, i32> = path.iter().map(|s| (s.id.as_uuid(), s.depth)).collect();

        let mut overrides: HashMap<uuid::Uuid, (i32, uuid::Uuid, Value)> = HashMap::new();
        let rows = sqlx::query(sql(format!(
            "select scope_id, role_id, value from {} where permission_code = $1 and role_id = any($2) and scope_id = any($3)",
            self.table("permission_override")
        )))
        .bind(permission)
        .bind(&roles)
        .bind(&reachable)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for row in rows {
            let (scope, role): (uuid::Uuid, uuid::Uuid) = (row.get("scope_id"), row.get("role_id"));
            let depth = depth_of.get(&scope).copied().unwrap_or(i32::MAX);
            let value: Value = row.get::<String, _>("value").parse()?;
            if overrides.get(&role).is_none_or(|(best, _, _)| depth < *best) {
                overrides.insert(role, (depth, scope, value));
            }
        }
        let own: HashMap<uuid::Uuid, Value> = sqlx::query(sql(format!(
            "select role_id, value from {} where permission_code = $1 and role_id = any($2)",
            self.table("role_permission")
        )))
        .bind(permission)
        .bind(&roles)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .iter()
        .map(|row| Ok((row.get("role_id"), row.get::<String, _>("value").parse()?)))
        .collect::<Result<_>>()?;
        let guests: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select r.id from {} r join {} t on t.id = r.template_id where t.code = $1 and r.id = any($2)",
            self.table("role"),
            self.table("role_template")
        )))
        .bind(GUEST_TEMPLATE)
        .bind(&roles)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(RoleValues {
            overrides,
            own,
            guests,
        })
    }

    /// The declared facts of a permission, or `None` when it is undeclared or retired.
    async fn declared(&self, conn: &mut PgConnection, permission: &str) -> Result<Option<Declared>> {
        let row = sqlx::query(sql(format!(
            "select kind, cardinality(risks) > 0 as risky from {} where code = $1 and retired_at is null",
            self.table("permission")
        )))
        .bind(permission)
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.map(|row| Declared {
            kind: if row.get::<String, _>("kind") == "write" {
                PermissionKind::Write
            } else {
                PermissionKind::Read
            },
            risky: row.get("risky"),
        }))
    }

    /// The roles the actor holds on the path, or `None` when the actor may not act at all here.
    async fn held(
        &self,
        conn: &mut PgConnection,
        actor: &Actor,
        path: &[PathScope],
        kind: PermissionKind,
    ) -> Result<Option<Vec<Held>>> {
        let ids: Vec<uuid::Uuid> = path.iter().map(|s| s.id.as_uuid()).collect();
        match *actor {
            Actor::Principal(principal) => {
                if !self.principal_active(conn, principal).await? {
                    return Ok(None);
                }
                let rows = sqlx::query(sql(format!(
                    "select id, role_id, scope_id from {} where principal_id = $1 and scope_id = any($2)
                     and valid_from <= $3 and (valid_until is null or valid_until > $3)",
                    self.table("grant")
                )))
                .bind(principal.as_uuid())
                .bind(&ids)
                .bind(self.now().as_offset_date_time())
                .fetch_all(conn)
                .await
                .map_err(kampus_db::classify)?;
                Ok(Some(
                    rows.iter()
                        .map(|row| Held {
                            grant: Some(GrantId::from_uuid(row.get("id"))),
                            role: RoleId::from_uuid(row.get("role_id")),
                            scope: ScopeId::from_uuid(row.get("scope_id")),
                        })
                        .collect(),
                ))
            }
            Actor::Visitor(campus) => {
                if !ids.contains(&campus.as_uuid()) {
                    return Ok(Some(Vec::new()));
                }
                let roles: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
                    "select r.id from {} r join {} t on t.id = r.template_id where t.code = $1 and r.campus_id = $2",
                    self.table("role"),
                    self.table("role_template")
                )))
                .bind(GUEST_TEMPLATE)
                .bind(campus.as_uuid())
                .fetch_all(conn)
                .await
                .map_err(kampus_db::classify)?;
                Ok(Some(
                    roles
                        .into_iter()
                        .map(|role| Held {
                            grant: None,
                            role: RoleId::from_uuid(role),
                            scope: campus,
                        })
                        .collect(),
                ))
            }
            Actor::ViewAs(principal, session) => {
                if kind == PermissionKind::Write || !self.principal_active(conn, principal).await? {
                    return Ok(None);
                }
                let row = sqlx::query(sql(format!(
                    "select scope_id, role_id from {} where id = $1 and principal_id = $2 and ended_at is null",
                    self.table("view_as_session")
                )))
                .bind(session.as_uuid())
                .bind(principal.as_uuid())
                .fetch_optional(conn)
                .await
                .map_err(kampus_db::classify)?;
                let Some(row) = row else { return Ok(None) };
                let scope: uuid::Uuid = row.get("scope_id");
                if !ids.contains(&scope) {
                    return Ok(None);
                }
                Ok(Some(vec![Held {
                    grant: None,
                    role: RoleId::from_uuid(row.get("role_id")),
                    scope: ScopeId::from_uuid(scope),
                }]))
            }
        }
    }

    /// Whether a principal exists and is active (RN-010).
    async fn principal_active(&self, conn: &mut PgConnection, principal: crate::PrincipalId) -> Result<bool> {
        let active: Option<bool> = sqlx::query_scalar(sql(format!(
            "select active from {} where id = $1",
            self.table("principal")
        )))
        .bind(principal.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(active == Some(true))
    }

    /// The scopes, among those under the actor's grants, where a permission is allowed: what a list
    /// filters by in its query, before paging (RN-020).
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the database fails.
    pub async fn allowed_scopes(&self, actor: &Actor, permission: &str) -> Result<Vec<ScopeId>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let roots: Vec<uuid::Uuid> = match *actor {
            Actor::Principal(principal) => sqlx::query_scalar(sql(format!(
                "select distinct scope_id from {} where principal_id = $1 and valid_from <= $2 and (valid_until is null or valid_until > $2)",
                self.table("grant")
            )))
            .bind(principal.as_uuid())
            .bind(self.now().as_offset_date_time())
            .fetch_all(&mut *conn)
            .await
            .map_err(kampus_db::classify)?,
            Actor::Visitor(campus) => vec![campus.as_uuid()],
            Actor::ViewAs(..) => Vec::new(),
        };
        let candidates: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct descendant_id from {} where ancestor_id = any($1)",
            self.table("scope_closure")
        )))
        .bind(&roots)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut allowed = Vec::new();
        for candidate in candidates {
            let scope = ScopeId::from_uuid(candidate);
            if self
                .evaluate(&mut conn, actor, permission, scope, false)
                .await?
                .allowed
            {
                allowed.push(scope);
            }
        }
        allowed.sort();
        Ok(allowed)
    }
}
