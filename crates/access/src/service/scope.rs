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

//! The scope tree: every scope has a kind, a parent of an allowed kind and its campus;
//! the closure table is kept in the transaction that creates a scope. Also principals, which belong to a
//! campus.

use kampus_common::{Error, Result};
use sqlx::{PgConnection, Row};

use crate::permissions::SCOPE_FREEZE;
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

    /// Creates a scope under a parent, when its kind may hang from the parent's.
    ///
    /// # Errors
    ///
    /// A `not_found` error for an unknown parent, a `validation` error for a kind that may not hang there.
    pub async fn create_scope(&self, parent: ScopeId, kind: ScopeKind) -> Result<ScopeId> {
        let mut tx = self.db.begin().await?;
        let id = self.create_scope_in(&mut tx, parent, kind).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Creates a scope under a parent inside the caller's transaction, so another module can create it
    /// together with the entity it belongs to.
    ///
    /// # Errors
    ///
    /// As [`Access::create_scope`].
    pub async fn create_scope_in(
        &self,
        conn: &mut PgConnection,
        parent: ScopeId,
        kind: ScopeKind,
    ) -> Result<ScopeId> {
        let row = sqlx::query(sql(format!(
            "select kind, campus_id from {} where id = $1",
            self.table("scope")
        )))
        .bind(parent.as_uuid())
        .fetch_optional(&mut *conn)
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
        self.insert_scope(conn, id, kind, Some(parent), campus).await?;
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

    /// The platform scope, once it exists.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn platform(&self) -> Result<Option<ScopeId>> {
        let id: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select id from {} where kind = 'platform'",
            self.table("scope")
        )))
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(id.map(ScopeId::from_uuid))
    }

    /// The campuses of the installation, oldest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn campuses(&self) -> Result<Vec<ScopeId>> {
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select id from {} where kind = 'campus' order by id",
            self.table("scope")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(ids.into_iter().map(ScopeId::from_uuid).collect())
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

    /// Freezes or unfreezes a scope; both need `access.scope.freeze` in it. Changing a frozen
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

    /// Freezes or unfreezes a scope inside the caller's transaction, for a module that checked its own
    /// permission, such as archiving a course.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn set_frozen_in(&self, conn: &mut PgConnection, scope: ScopeId, frozen: bool) -> Result<()> {
        let changed = sqlx::query(sql(format!(
            "update {} set frozen = $2, version = version + 1 where id = $1",
            self.table("scope")
        )))
        .bind(scope.as_uuid())
        .bind(frozen)
        .execute(conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            return Err(Error::not_found("access.scope_not_found"));
        }
        Ok(())
    }

    /// Moves a scope and its subtree under another parent of the same campus, inside the caller's
    /// transaction: grants, overrides and descendants move with it, and only the closure rows that cross
    /// the moved subtree change.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope or parent, `validation` (`access.scope_parent_invalid`) for a
    /// parent of a kind it may not hang from, of another campus, or inside its own subtree.
    pub async fn move_scope_in(
        &self,
        conn: &mut PgConnection,
        scope: ScopeId,
        parent: ScopeId,
    ) -> Result<()> {
        let scopes = self.table("scope");
        let closure = self.table("scope_closure");
        let rows = sqlx::query(sql(format!(
            "select id, kind, campus_id from {scopes} where id = any($1) for update"
        )))
        .bind([scope.as_uuid(), parent.as_uuid()].as_slice())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let find = |id: ScopeId| {
            rows.iter()
                .find(|r| r.get::<uuid::Uuid, _>("id") == id.as_uuid())
                .ok_or_else(|| Error::not_found("access.scope_not_found"))
        };
        let (moved, target) = (find(scope)?, find(parent)?);
        let kind: ScopeKind = moved.get::<String, _>("kind").parse()?;
        let parent_kind: ScopeKind = target.get::<String, _>("kind").parse()?;
        let same_campus = moved.get::<Option<uuid::Uuid>, _>("campus_id")
            == target.get::<Option<uuid::Uuid>, _>("campus_id");
        let inside: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {closure} where ancestor_id = $1 and descendant_id = $2)"
        )))
        .bind(scope.as_uuid())
        .bind(parent.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        if !kind.may_hang_from(parent_kind) || !same_campus || inside {
            return Err(Error::validation(
                "access.scope_parent_invalid",
                vec![kampus_common::FieldError::new("parent", "kind")],
            )
            .with_param("kind", kind.as_str())
            .with_param("parent", parent_kind.as_str()));
        }
        sqlx::query(sql(format!(
            "delete from {closure} where descendant_id in (select descendant_id from {closure} where ancestor_id = $1)
             and ancestor_id not in (select descendant_id from {closure} where ancestor_id = $1)"
        )))
        .bind(scope.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {closure} (ancestor_id, descendant_id, depth)
             select a.ancestor_id, d.descendant_id, a.depth + d.depth + 1
             from {closure} a cross join {closure} d where a.descendant_id = $2 and d.ancestor_id = $1"
        )))
        .bind(scope.as_uuid())
        .bind(parent.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {scopes} set parent_id = $2, version = version + 1 where id = $1"
        )))
        .bind(scope.as_uuid())
        .bind(parent.as_uuid())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Deletes a scope inside the caller's transaction, for the module that owns the entity it belongs to:
    /// its descendants, grants and overrides go with it, so the module checks first that nothing else
    /// hangs from it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope, `validation` for the platform or a campus.
    pub async fn delete_scope_in(&self, conn: &mut PgConnection, scope: ScopeId) -> Result<()> {
        let kind: Option<String> = sqlx::query_scalar(sql(format!(
            "select kind from {} where id = $1",
            self.table("scope")
        )))
        .bind(scope.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let kind: ScopeKind = kind
            .ok_or_else(|| Error::not_found("access.scope_not_found"))?
            .parse()?;
        if matches!(kind, ScopeKind::Platform | ScopeKind::Tenant | ScopeKind::Campus) {
            return Err(Error::validation(
                "access.scope_not_deletable",
                vec![kampus_common::FieldError::new("scope", "kind")],
            ));
        }
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("scope"))))
            .bind(scope.as_uuid())
            .execute(conn)
            .await
            .map(drop)
            .map_err(kampus_db::classify)
    }

    /// The campus of a principal.
    ///
    /// # Errors
    ///
    /// `not_found` (`access.principal_not_found`) for an unknown principal.
    pub async fn principal_campus(&self, id: PrincipalId) -> Result<ScopeId> {
        let campus: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select campus_id from {} where id = $1",
            self.table("principal")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("access.principal_not_found"))?;
        Ok(ScopeId::from_uuid(campus))
    }

    /// Creates a principal in a campus; identity creates one for every account, with the account's id.
    ///
    /// # Errors
    ///
    /// A `conflict` error when the campus does not exist or the id is taken.
    pub async fn create_principal(&self, id: PrincipalId, campus: ScopeId) -> Result<()> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.create_principal_in(&mut conn, id, campus).await
    }

    /// Creates a principal inside the caller's transaction, so identity creates it with its account.
    ///
    /// # Errors
    ///
    /// As [`Access::create_principal`].
    pub async fn create_principal_in(
        &self,
        conn: &mut PgConnection,
        id: PrincipalId,
        campus: ScopeId,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id) values ($1, $2)",
            self.table("principal")
        )))
        .bind(id.as_uuid())
        .bind(campus.as_uuid())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Marks a principal active or not; identity calls it when an account is suspended, restored or
    /// deleted, so inactive accounts fail every check.
    ///
    /// # Errors
    ///
    /// A `not_found` error for an unknown principal.
    pub async fn set_principal_active(&self, id: PrincipalId, active: bool) -> Result<()> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.set_principal_active_in(&mut conn, id, active).await
    }

    /// Marks a principal active or not inside the caller's transaction, together with the account change.
    ///
    /// # Errors
    ///
    /// As [`Access::set_principal_active`].
    pub async fn set_principal_active_in(
        &self,
        conn: &mut PgConnection,
        id: PrincipalId,
        active: bool,
    ) -> Result<()> {
        let updated = sqlx::query(sql(format!(
            "update {} set active = $2 where id = $1",
            self.table("principal")
        )))
        .bind(id.as_uuid())
        .bind(active)
        .execute(conn)
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
