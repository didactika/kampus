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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Categories: the tree of the catalogue under each campus. Order is local to each parent, and a
//! category is deleted only when it is empty.

use kampus_access::{Actor, ScopeId, ScopeKind};
use kampus_common::{Error, FieldError, Result};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::{check_version, optional, required, taken, valid, visibility_of, window_of};
use crate::permissions::CATEGORY_MANAGE;
use crate::support::sort_key;
use crate::{Category, CategoryFields, CategoryId, Courses, Place, RecordKind, sql};
use kampus_db::references::ExternalReference;

/// The columns of a category, as read.
const COLUMNS: &str = "id, scope_id, campus_id, parent_id, code, name, description, sort_key, visibility, \
    visible_from, visible_until, version";

/// A category from its row.
fn category(row: &PgRow) -> Category {
    Category {
        id: CategoryId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        campus: ScopeId::from_uuid(row.get("campus_id")),
        parent: row
            .get::<Option<uuid::Uuid>, _>("parent_id")
            .map(CategoryId::from_uuid),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        sort_key: row.get("sort_key"),
        visibility: visibility_of(row),
        version: row.get("version"),
    }
}

/// Checks the fields of a category, every problem at once.
fn checked(fields: CategoryFields) -> Result<CategoryFields> {
    let mut problems = Vec::new();
    let name = required("name", &fields.name, &mut problems);
    valid("courses.category_invalid", problems)?;
    Ok(CategoryFields {
        name,
        code: optional(fields.code),
        description: optional(fields.description),
        visibility: fields.visibility.validate()?,
    })
}

impl Courses {
    /// Creates a category in a campus, at the top or under another category; needs
    /// `courses.category.manage` where it goes.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields or a parent of
    /// another campus, `conflict` when the campus has a category with that code.
    pub async fn create_category(
        &self,
        actor: &Actor,
        campus: ScopeId,
        parent: Option<CategoryId>,
        fields: CategoryFields,
        place: Place<CategoryId>,
    ) -> Result<Category> {
        self.create_category_with(actor, (campus, parent), fields, place, None)
            .await
    }

    /// Creates a category, with the identifier a system gives it when one is given, in one transaction.
    pub(crate) async fn create_category_with(
        &self,
        actor: &Actor,
        (campus, parent): (ScopeId, Option<CategoryId>),
        fields: CategoryFields,
        place: Place<CategoryId>,
        reference: Option<&ExternalReference>,
    ) -> Result<Category> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let parent_scope = self.parent_scope_in(&mut tx, campus, parent).await?;
        self.access
            .require_in(&mut tx, actor, CATEGORY_MANAGE, parent_scope)
            .await?;
        let siblings = self.category_siblings_in(&mut tx, campus, parent, None).await?;
        let key = sort_key::place(&siblings, place)?;
        let scope = self
            .access
            .create_scope_in(&mut tx, parent_scope, ScopeKind::Category)
            .await?;
        let (from, until) = window_of(fields.visibility);
        let row = sqlx::query(sql(format!(
            "insert into {} (id, scope_id, campus_id, parent_id, code, name, description, sort_key, visibility,
             visible_from, visible_until) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) returning {COLUMNS}",
            self.table("category")
        )))
        .bind(CategoryId::generate().as_uuid())
        .bind(scope.as_uuid())
        .bind(campus.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .bind(&fields.code)
        .bind(&fields.name)
        .bind(&fields.description)
        .bind(&key)
        .bind(fields.visibility.mode())
        .bind(from)
        .bind(until)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken("courses.category_code_taken"))?;
        let created = category(&row);
        if let Some(reference) = reference {
            self.references(RecordKind::Category)
                .set_in(&mut tx, campus.as_uuid(), created.id.as_uuid(), reference)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// A category, as stored.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown category.
    pub async fn category(&self, id: CategoryId) -> Result<Category> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.read_category_in(&mut conn, id, false).await
    }

    /// The categories of a campus under a parent, in their order; hidden ones only for people with
    /// `courses.hidden.view`.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown parent.
    pub async fn categories(
        &self,
        actor: &Actor,
        campus: ScopeId,
        parent: Option<CategoryId>,
    ) -> Result<Vec<Category>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where campus_id = $1 and parent_id is not distinct from $2 order by sort_key",
            self.table("category")
        )))
        .bind(campus.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let parent_scope = match parent {
            Some(p) => self.category(p).await?.scope,
            None => campus,
        };
        let all = rows.iter().map(category).collect();
        self.visible_children(actor, parent_scope, all, |c: &Category| (c.scope, c.visibility))
            .await
    }

    /// Changes the fields of a category; `expected_version` is the version read (`If-Match`).
    ///
    /// # Errors
    ///
    /// `permission_denied` without `courses.category.manage`, `validation` for invalid fields,
    /// `precondition_failed` when it changed since it was read, `conflict` for a code in use.
    pub async fn update_category(
        &self,
        actor: &Actor,
        id: CategoryId,
        fields: CategoryFields,
        expected_version: Option<i32>,
    ) -> Result<Category> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let stored = self.read_category_in(&mut tx, id, true).await?;
        self.access
            .require_in(&mut tx, actor, CATEGORY_MANAGE, stored.scope)
            .await?;
        check_version(expected_version, stored.version, "courses.category_changed")?;
        let (from, until) = window_of(fields.visibility);
        let row = sqlx::query(sql(format!(
            "update {} set code = $2, name = $3, description = $4, visibility = $5, visible_from = $6,
             visible_until = $7, version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("category")
        )))
        .bind(id.as_uuid())
        .bind(&fields.code)
        .bind(&fields.name)
        .bind(&fields.description)
        .bind(fields.visibility.mode())
        .bind(from)
        .bind(until)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken("courses.category_code_taken"))?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(category(&row))
    }

    /// Moves a category, with everything in it, under another parent of its campus or to another place
    /// among its siblings; only its own key changes. Needs `courses.category.manage` in the category and
    /// where it goes.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for a parent inside the category or of
    /// another campus, `precondition_failed` when it changed since it was read.
    pub async fn move_category(
        &self,
        actor: &Actor,
        id: CategoryId,
        parent: Option<CategoryId>,
        place: Place<CategoryId>,
        expected_version: Option<i32>,
    ) -> Result<Category> {
        let mut tx = self.db.begin().await?;
        let stored = self.read_category_in(&mut tx, id, true).await?;
        self.access
            .require_in(&mut tx, actor, CATEGORY_MANAGE, stored.scope)
            .await?;
        check_version(expected_version, stored.version, "courses.category_changed")?;
        let parent_scope = self.parent_scope_in(&mut tx, stored.campus, parent).await?;
        self.access
            .require_in(&mut tx, actor, CATEGORY_MANAGE, parent_scope)
            .await?;
        if parent != stored.parent {
            self.access
                .move_scope_in(&mut tx, stored.scope, parent_scope)
                .await?;
        }
        let siblings = self
            .category_siblings_in(&mut tx, stored.campus, parent, Some(id))
            .await?;
        let key = sort_key::place(&siblings, place)?;
        let row = sqlx::query(sql(format!(
            "update {} set parent_id = $2, sort_key = $3, version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("category")
        )))
        .bind(id.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .bind(&key)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(category(&row))
    }

    /// Deletes an empty category with its scope; one with categories or courses in it is refused, so
    /// nothing is lost by accident.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `courses.category.manage`, `conflict` (`courses.category_not_empty`)
    /// when something is in it, `precondition_failed` when it changed since it was read.
    pub async fn delete_category(
        &self,
        actor: &Actor,
        id: CategoryId,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.read_category_in(&mut tx, id, true).await?;
        self.access
            .require_in(&mut tx, actor, CATEGORY_MANAGE, stored.scope)
            .await?;
        check_version(expected_version, stored.version, "courses.category_changed")?;
        let used: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where parent_id = $1) or exists (select 1 from {} where category_id = $1)",
            self.table("category"),
            self.table("course")
        )))
        .bind(id.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if used {
            return Err(Error::conflict("courses.category_not_empty"));
        }
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("category")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.access.delete_scope_in(&mut tx, stored.scope).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Reads a category, locked for a change when `lock`.
    pub(crate) async fn read_category_in(
        &self,
        conn: &mut PgConnection,
        id: CategoryId,
        lock: bool,
    ) -> Result<Category> {
        let lock = if lock { " for update" } else { "" };
        sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1{lock}",
            self.table("category")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .map(|row| category(&row))
        .ok_or_else(|| Error::not_found("courses.category_not_found"))
    }

    /// The scope a category goes in: its parent's, or the campus.
    async fn parent_scope_in(
        &self,
        conn: &mut PgConnection,
        campus: ScopeId,
        parent: Option<CategoryId>,
    ) -> Result<ScopeId> {
        let Some(parent) = parent else {
            return Ok(campus);
        };
        let parent = self.read_category_in(conn, parent, false).await?;
        if parent.campus != campus {
            return Err(Error::validation(
                "courses.category_parent_invalid",
                vec![FieldError::new("parent", "campus")],
            ));
        }
        Ok(parent.scope)
    }

    /// The siblings of a category under a parent, in order, without the one moving.
    async fn category_siblings_in(
        &self,
        conn: &mut PgConnection,
        campus: ScopeId,
        parent: Option<CategoryId>,
        moving: Option<CategoryId>,
    ) -> Result<Vec<(CategoryId, String)>> {
        let rows = sqlx::query(sql(format!(
            "select id, sort_key from {} where campus_id = $1 and parent_id is not distinct from $2
             and id is distinct from $3 order by sort_key",
            self.table("category")
        )))
        .bind(campus.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .bind(moving.map(|m| m.as_uuid()))
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (CategoryId::from_uuid(r.get("id")), r.get("sort_key")))
            .collect())
    }
}
