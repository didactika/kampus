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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Categories: the nodes of the tree of records of a course. Each course has a root created with its first
//! change, each category has a total record of its own, and a category is deleted only when it is empty.

use kampus_access::Actor;
use kampus_common::{Error, Result};
use kampus_courses::support::sort_key;
use kampus_courses::{Course, CourseId, Place};
use sqlx::{PgConnection, Row};

use super::{CATEGORY_COLUMNS, RECORD_COLUMNS, category, changed, principal, problem, record};
use crate::permissions::{MANAGE, VIEW_ALL, VIEW_HIDDEN, VIEW_OWN};
use crate::{Category, CategoryFields, CategoryId, RecordId, Records, Tree, events, sql};

/// The most characters of a name.
const NAME_LIMIT: usize = 200;

/// The name of a category, checked with the rest of its fields.
fn checked(fields: &CategoryFields) -> Result<String> {
    let mut problems = Vec::new();
    let name = fields.name.trim();
    if name.is_empty() || name.chars().count() > NAME_LIMIT {
        problems.push(problem("name", "length"));
    }
    if fields.drop_lowest < 0 {
        problems.push(problem("drop_lowest", "min"));
    }
    if fields.keep_highest < 0 {
        problems.push(problem("keep_highest", "min"));
    }
    if fields.drop_lowest > 0 && fields.keep_highest > 0 {
        problems.push(problem("keep_highest", "exclusive"));
    }
    if problems.is_empty() {
        Ok(name.to_owned())
    } else {
        Err(Error::validation("records.category_invalid", problems))
    }
}

impl Records {
    /// Fails unless the actor may look at the records of a course: its own grades or everyone's.
    pub(crate) async fn require_view(&self, actor: &Actor, course: &Course) -> Result<()> {
        if !self.courses.sees(actor, course.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        if self.access.check(actor, VIEW_OWN, course.scope).await?.allowed {
            return Ok(());
        }
        self.access.require(actor, VIEW_ALL, course.scope).await
    }

    /// The root category of a course, and the total of the course, created when missing.
    pub(crate) async fn ensure_root_in(&self, conn: &mut PgConnection, course: &Course) -> Result<Category> {
        sqlx::query(sql(format!(
            "insert into {} (id, course_id, root_course_id, name, sort_key) values ($1, $2, $2, $3, 'V')
             on conflict (root_course_id) do nothing",
            self.table("category")
        )))
        .bind(CategoryId::generate().as_uuid())
        .bind(course.id.as_uuid())
        .bind(&course.full_name)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let row = sqlx::query(sql(format!(
            "select {CATEGORY_COLUMNS} from {} where root_course_id = $1",
            self.table("category")
        )))
        .bind(course.id.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let root = category(&row);
        self.ensure_total_in(conn, &root, None).await?;
        Ok(root)
    }

    /// The total record of a category, created when missing; `parent` is the category it sits in, none for
    /// the total of the course.
    async fn ensure_total_in(
        &self,
        conn: &mut PgConnection,
        category: &Category,
        parent: Option<CategoryId>,
    ) -> Result<RecordId> {
        let key = match parent {
            Some(parent) => {
                let siblings = self.record_siblings_in(conn, parent).await?;
                sort_key::place(&siblings, Place::Last)?
            }
            None => "V".to_owned(),
        };
        let kind = if parent.is_some() {
            "category_total"
        } else {
            "course_total"
        };
        let inserted: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "insert into {} (id, course_id, kind, title, category_id, total_of_id, \"min\", \"max\", weight,
             multiplier, offset_value, sort_key) values ($1, $2, $3, $4, $5, $6, 0, 100, 1, 1, 0, $7)
             on conflict (total_of_id) do nothing returning id",
            self.table("record")
        )))
        .bind(RecordId::generate().as_uuid())
        .bind(category.course.as_uuid())
        .bind(kind)
        .bind(&category.name)
        .bind(parent.map(|p| p.as_uuid()))
        .bind(category.id.as_uuid())
        .bind(key)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(id) = inserted {
            return Ok(RecordId::from_uuid(id));
        }
        let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select id from {} where total_of_id = $1",
            self.table("record")
        )))
        .bind(category.id.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(RecordId::from_uuid(id))
    }

    /// The records of a category with their places.
    pub(crate) async fn record_siblings_in(
        &self,
        conn: &mut PgConnection,
        category: CategoryId,
    ) -> Result<Vec<(RecordId, String)>> {
        let rows = sqlx::query(sql(format!(
            "select id, sort_key from {} where category_id = $1 order by sort_key",
            self.table("record")
        )))
        .bind(category.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (RecordId::from_uuid(r.get("id")), r.get("sort_key")))
            .collect())
    }

    /// Adds a category to a course, under another or under the root; needs `records.manage` in the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`records.category_invalid`).
    pub async fn define_category(
        &self,
        actor: &Actor,
        course: CourseId,
        parent: Option<CategoryId>,
        fields: CategoryFields,
        place: Place<CategoryId>,
    ) -> Result<Category> {
        let course = self.courses.course(course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        let name = checked(&fields)?;
        let mut tx = self.db.begin().await?;
        let root = self.ensure_root_in(&mut tx, &course).await?;
        let parent_id = parent.unwrap_or(root.id);
        let above = self.category_in(&mut tx, parent_id, true).await?;
        if above.course != course.id {
            return Err(Error::not_found("records.category_not_found"));
        }
        let rows = sqlx::query(sql(format!(
            "select id, sort_key from {} where parent_id = $1 order by sort_key",
            self.table("category")
        )))
        .bind(above.id.as_uuid())
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let siblings: Vec<(CategoryId, String)> = rows
            .iter()
            .map(|r| (CategoryId::from_uuid(r.get("id")), r.get("sort_key")))
            .collect();
        let key = sort_key::place(&siblings, place)?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, course_id, parent_id, name, method, drop_lowest, keep_highest, exclude_empty,
             sort_key) values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning {CATEGORY_COLUMNS}",
            self.table("category")
        )))
        .bind(CategoryId::generate().as_uuid())
        .bind(course.id.as_uuid())
        .bind(above.id.as_uuid())
        .bind(&name)
        .bind(fields.method.code())
        .bind(fields.drop_lowest)
        .bind(fields.keep_highest)
        .bind(fields.exclude_empty)
        .bind(key)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let created = category(&row);
        let total = self.ensure_total_in(&mut tx, &created, Some(above.id)).await?;
        self.publish_in(
            &mut tx,
            events::RECORD_CREATED,
            &course,
            total,
            None,
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// Replaces the name and the aggregation of a category, on the version it was read at; needs
    /// `records.manage` in the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation`, and `precondition_failed`
    /// (`records.category_changed`) when it changed since it was read.
    pub async fn update_category(
        &self,
        actor: &Actor,
        id: CategoryId,
        fields: CategoryFields,
        expected_version: Option<i32>,
    ) -> Result<Category> {
        let mut tx = self.db.begin().await?;
        let stored = self.category_in(&mut tx, id, true).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        if expected_version.is_some_and(|v| v != stored.version) {
            return Err(changed("records.category_changed", stored.version));
        }
        let name = checked(&fields)?;
        let row = sqlx::query(sql(format!(
            "update {} set name = $2, method = $3, drop_lowest = $4, keep_highest = $5, exclude_empty = $6,
             version = version + 1 where id = $1 returning {CATEGORY_COLUMNS}",
            self.table("category")
        )))
        .bind(id.as_uuid())
        .bind(&name)
        .bind(fields.method.code())
        .bind(fields.drop_lowest)
        .bind(fields.keep_highest)
        .bind(fields.exclude_empty)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if stored.parent.is_some() {
            sqlx::query(sql(format!(
                "update {} set title = $2, version = version + 1 where total_of_id = $1",
                self.table("record")
            )))
            .bind(id.as_uuid())
            .bind(&name)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        let total = self.total_of_in(&mut tx, id).await?;
        self.publish_in(
            &mut tx,
            events::RECORD_UPDATED,
            &course,
            total,
            None,
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(category(&row))
    }

    /// The id of the total record of a category.
    async fn total_of_in(&self, conn: &mut PgConnection, id: CategoryId) -> Result<RecordId> {
        let total: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "select id from {} where total_of_id = $1",
            self.table("record")
        )))
        .bind(id.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(RecordId::from_uuid(total))
    }

    /// Deletes an empty category with its total; needs `records.manage` in the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `conflict` (`records.category_is_root`) for the root of the course,
    /// (`records.category_not_empty`) for one that holds categories or records.
    pub async fn delete_category(&self, actor: &Actor, id: CategoryId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.category_in(&mut tx, id, true).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        if stored.parent.is_none() {
            return Err(Error::conflict("records.category_is_root"));
        }
        let used: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {c} where parent_id = $1)
                 or exists (select 1 from {r} where category_id = $1)",
            c = self.table("category"),
            r = self.table("record")
        )))
        .bind(id.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if used {
            return Err(Error::conflict("records.category_not_empty"));
        }
        let total = self.total_of_in(&mut tx, id).await?;
        self.publish_in(
            &mut tx,
            events::RECORD_DELETED,
            &course,
            total,
            None,
            principal(actor),
        )
        .await?;
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("category")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The categories and grade records of a course as the actor may see them: hidden records only for
    /// whoever holds `records.view_hidden`. Reading creates nothing, so a course nobody has configured has
    /// an empty tree.
    ///
    /// # Errors
    ///
    /// `not_found` for a course the actor does not see, `permission_denied` without
    /// `records.view_own` or `records.view_all`.
    pub async fn tree(&self, actor: &Actor, course: CourseId) -> Result<Tree> {
        let course = self.courses.course(course).await?;
        self.require_view(actor, &course).await?;
        let hidden = self.access.check(actor, VIEW_HIDDEN, course.scope).await?.allowed;
        let categories = sqlx::query(sql(format!(
            "select {CATEGORY_COLUMNS} from {} where course_id = $1 order by parent_id nulls first, sort_key",
            self.table("category")
        )))
        .bind(course.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let records = sqlx::query(sql(format!(
            "select {RECORD_COLUMNS} from {} where course_id = $1
             and ($2 or hidden_until is null or hidden_until <= now())
             order by category_id nulls first, sort_key",
            self.table("record")
        )))
        .bind(course.id.as_uuid())
        .bind(hidden)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(Tree {
            categories: categories.iter().map(category).collect(),
            records: records.iter().map(record).collect(),
        })
    }
}
