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

//! The scope tree (RN-001, RN-002): every scope has a kind, a parent of an allowed kind and its campus;
//! the closure table is kept in the transaction that creates a scope. Also principals, which belong to a
//! campus (DEC-079).

use kampus_common::{Error, Result};
use sqlx::{PgConnection, Row};

use crate::declarations::SCOPE_FREEZE;
use crate::{Access, Actor, PrincipalId, ScopeId, ScopeKind, sql};

/// One scope on the path from a scope up to the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathScope {
    /// The scope.
    pub id: ScopeId,
    /// Its kind.
    pub kind: ScopeKind,
    /// Whether it is frozen.
    pub frozen: bool,
    /// How many levels above the checked scope it is (0 for the scope itself).
    pub depth: i32,
}

impl Access {
    /// Creates the root of the tree: the platform scope.
    ///
    /// # Errors
    ///
    /// A `conflict` error when the platform exists already.
    pub async fn create_platform(&self) -> Result<ScopeId> {
        let mut tx = self.db.begin().await?;
        let exists: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where kind = 'platform')",
            self.table("scope")
        )))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if exists {
            return Err(Error::conflict("access.platform_exists"));
        }
        let id = ScopeId::generate();
        self.insert_scope(&mut tx, id, ScopeKind::Platform, None, None)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Creates a scope under a parent, when its kind may hang from the parent's (RN-001).
    ///
    /// # Errors
    ///
    /// A `not_found` error for an unknown parent, a `validation` error for a kind that may not hang there.
    pub async fn create_scope(&self, parent: ScopeId, kind: ScopeKind) -> Result<ScopeId> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "select kind, campus_id from {} where id = $1",
            self.table("scope")
        )))
        .bind(parent.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("access.scope_not_found"))?;
        let parent_kind: ScopeKind = row.get::<String, _>("kind").parse()?;
        if !kind.may_hang_from(parent_kind) {
            return Err(Error::validation(
                "access.scope_parent_invalid",
                vec![kampus_common::FieldError::new("parent", "kind")],
            )
            .with_param("kind", kind.as_str())
            .with_param("parent", parent_kind.as_str()));
        }
        let id = ScopeId::generate();
        let campus = if kind == ScopeKind::Campus {
            Some(id)
        } else {
            row.get::<Option<uuid::Uuid>, _>("campus_id")
                .map(ScopeId::from_uuid)
        };
        self.insert_scope(&mut tx, id, kind, Some(parent), campus).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Inserts a scope and its rows of the closure table.
    async fn insert_scope(
        &self,
        conn: &mut PgConnection,
        id: ScopeId,
        kind: ScopeKind,
        parent: Option<ScopeId>,
        campus: Option<ScopeId>,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, kind, parent_id, campus_id) values ($1, $2, $3, $4)",
            self.table("scope")
        )))
        .bind(id.as_uuid())
        .bind(kind.as_str())
        .bind(parent.map(|p| p.as_uuid()))
        .bind(campus.map(|c| c.as_uuid()))
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let closure = self.table("scope_closure");
        sqlx::query(sql(format!(
            "insert into {closure} (ancestor_id, descendant_id, depth)
             select $1, $1, 0
             union all
             select ancestor_id, $1, depth + 1 from {closure} where descendant_id = $2"
        )))
        .bind(id.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// The scopes from `scope` up to the root, nearest first.
    ///
    /// # Errors
    ///
    /// A `not_found` error for an unknown scope.
    pub async fn path(&self, conn: &mut PgConnection, scope: ScopeId) -> Result<Vec<PathScope>> {
        let rows = sqlx::query(sql(format!(
            "select s.id, s.kind, s.frozen, c.depth from {} c join {} s on s.id = c.ancestor_id where c.descendant_id = $1 order by c.depth",
            self.table("scope_closure"),
            self.table("scope")
        )))
        .bind(scope.as_uuid())
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        if rows.is_empty() {
            return Err(Error::not_found("access.scope_not_found"));
        }
        rows.iter()
            .map(|row| {
                Ok(PathScope {
                    id: ScopeId::from_uuid(row.get("id")),
                    kind: row.get::<String, _>("kind").parse()?,
                    frozen: row.get("frozen"),
                    depth: row.get("depth"),
                })
            })
            .collect()
    }

    /// The campus a scope belongs to, if it is at or below a campus.
    pub(crate) async fn campus_of(&self, conn: &mut PgConnection, scope: ScopeId) -> Result<Option<ScopeId>> {
        let campus: Option<Option<uuid::Uuid>> = sqlx::query_scalar(sql(format!(
            "select campus_id from {} where id = $1",
            self.table("scope")
        )))
        .bind(scope.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?;
        campus
            .map(|c| c.map(ScopeId::from_uuid))
            .ok_or_else(|| Error::not_found("access.scope_not_found"))
    }

    /// Freezes or unfreezes a scope; both need `access.scope.freeze` in it (RN-011). Changing a frozen
    /// scope is refused, so unfreezing checks the permission as if the scope were open.
    ///
    /// # Errors
    ///
    /// A `permission_denied` error without the permission.
    pub async fn set_frozen(&self, actor: &Actor, scope: ScopeId, frozen: bool) -> Result<()> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let decision = self.evaluate(&mut conn, actor, SCOPE_FREEZE, scope, true).await?;
        if !decision.allowed {
            return Err(Error::permission_denied("access.denied").with_param("permission", SCOPE_FREEZE));
        }
        sqlx::query(sql(format!(
            "update {} set frozen = $2, version = version + 1 where id = $1",
            self.table("scope")
        )))
        .bind(scope.as_uuid())
        .bind(frozen)
        .execute(&mut *conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Creates a principal in a campus; identity creates one for every account, with the account's id.
    ///
    /// # Errors
    ///
    /// A `conflict` error when the campus does not exist or the id is taken.
    pub async fn create_principal(&self, id: PrincipalId, campus: ScopeId) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id) values ($1, $2)",
            self.table("principal")
        )))
        .bind(id.as_uuid())
        .bind(campus.as_uuid())
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Marks a principal active or not; identity calls it when an account is suspended, restored or
    /// deleted, so inactive accounts fail every check (RN-010).
    ///
    /// # Errors
    ///
    /// A `not_found` error for an unknown principal.
    pub async fn set_principal_active(&self, id: PrincipalId, active: bool) -> Result<()> {
        let updated = sqlx::query(sql(format!(
            "update {} set active = $2 where id = $1",
            self.table("principal")
        )))
        .bind(id.as_uuid())
        .bind(active)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if updated == 0 {
            Err(Error::not_found("access.principal_not_found"))
        } else {
            Ok(())
        }
    }
}
