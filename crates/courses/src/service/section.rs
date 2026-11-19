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

//! Sections: parts of a course with a stable identity, nested to any depth and ordered within their
//! parent. Moving a section moves its subtree, its permissions and its content; deleting one keeps it
//! recoverable.

use kampus_access::{Actor, ScopeId, ScopeKind};
use kampus_common::{Error, FieldError, Result};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::{check_version, optional, valid, visibility_of, window_of};
use crate::permissions::{HIDDEN_VIEW, SECTION_MANAGE};
use crate::support::sort_key;
use crate::{Course, CourseId, Courses, Place, Section, SectionFields, SectionId, sql};

/// The columns of a section, as read.
const COLUMNS: &str = "id, scope_id, course_id, parent_id, title, summary, sort_key, visibility, visible_from, visible_until, version";

/// The longest title of a section, in characters.
const MAX_TITLE: usize = 255;

/// A section from its row.
fn section(row: &PgRow) -> Section {
    Section {
        id: SectionId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        course: CourseId::from_uuid(row.get("course_id")),
        parent: row
            .get::<Option<uuid::Uuid>, _>("parent_id")
            .map(SectionId::from_uuid),
        title: row.get("title"),
        summary: row.get("summary"),
        sort_key: row.get("sort_key"),
        visibility: visibility_of(row),
        version: row.get("version"),
    }
}

/// Checks the fields of a section.
fn checked(fields: SectionFields) -> Result<SectionFields> {
    let title = optional(fields.title);
    let mut problems = Vec::new();
    if title.as_ref().is_some_and(|t| t.chars().count() > MAX_TITLE) {
        problems.push(FieldError::new("title", "length"));
    }
    valid("courses.section_invalid", problems)?;
    Ok(SectionFields {
        title,
        summary: optional(fields.summary),
        visibility: fields.visibility.validate()?,
    })
}

impl SectionFields {
    /// The fields of the first section of a new course: untitled, shown.
    #[must_use]
    pub const fn first() -> Self {
        Self {
            title: None,
            summary: None,
            visibility: crate::Visibility::Shown,
        }
    }
}

impl Courses {
    /// Adds a section to a course, at the top or inside another section; needs `courses.section.manage`
    /// where it goes.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields or a parent of
    /// another course, `not_found` for an unknown course or parent.
    pub async fn add_section(
        &self,
        actor: &Actor,
        course: CourseId,
        parent: Option<SectionId>,
        fields: SectionFields,
        place: Place<SectionId>,
    ) -> Result<Section> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let course = self.read_course_in(&mut tx, course, false).await?;
        let parent = match parent {
            Some(p) => Some(self.parent_section_in(&mut tx, &course, p).await?),
            None => None,
        };
        let scope = parent.as_ref().map_or(course.scope, |p| p.scope);
        self.access.require(actor, SECTION_MANAGE, scope).await?;
        let added = self
            .insert_section_in(&mut tx, &course, parent.as_ref(), Some(place), &fields)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(added)
    }

    /// A section that is not deleted.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown or deleted section.
    pub async fn section(&self, id: SectionId) -> Result<Section> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.read_section_in(&mut conn, id, false).await
    }

    /// The sections of a course the actor sees, depth first: each section followed by those nested in
    /// it, in their order.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course.
    pub async fn sections(&self, actor: &Actor, course: CourseId) -> Result<Vec<Section>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where course_id = $1 and deleted_at is null order by sort_key",
            self.table("section")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let all: Vec<Section> = rows.iter().map(section).collect();
        let course = self.course(course).await?;
        let now = self.now();
        let course_shown = self.shown(course.scope).await?;
        let mut ordered = Vec::with_capacity(all.len());
        let mut stack: Vec<(&Section, bool)> = all
            .iter()
            .filter(|s| s.parent.is_none())
            .rev()
            .map(|s| (s, course_shown))
            .collect();
        while let Some((next, parent_shown)) = stack.pop() {
            let shown = parent_shown && next.visibility.shown_at(now);
            if !shown && !self.access.check(actor, HIDDEN_VIEW, next.scope).await?.allowed {
                continue;
            }
            ordered.push(next.clone());
            stack.extend(
                all.iter()
                    .filter(|s| s.parent == Some(next.id))
                    .rev()
                    .map(|s| (s, shown)),
            );
        }
        Ok(ordered)
    }

    /// Changes the fields of a section; needs `courses.section.manage` in it.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields,
    /// `precondition_failed` when it changed since it was read.
    pub async fn update_section(
        &self,
        actor: &Actor,
        id: SectionId,
        fields: SectionFields,
        expected_version: Option<i32>,
    ) -> Result<Section> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let stored = self.read_section_in(&mut tx, id, true).await?;
        self.access.require(actor, SECTION_MANAGE, stored.scope).await?;
        check_version(expected_version, stored.version, "courses.section_changed")?;
        let (from, until) = window_of(fields.visibility);
        let row = sqlx::query(sql(format!(
            "update {} set title = $2, summary = $3, visibility = $4, visible_from = $5, visible_until = $6,
             version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("section")
        )))
        .bind(id.as_uuid())
        .bind(&fields.title)
        .bind(&fields.summary)
        .bind(fields.visibility.mode())
        .bind(from)
        .bind(until)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(section(&row))
    }

    /// Moves a section, with its subtree, permissions and content, under another parent of the same
    /// course or to another place among its siblings; only its own key changes. Needs
    /// `courses.section.manage` in the section and where it goes.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for a parent inside the section or of
    /// another course, `precondition_failed` when it changed since it was read.
    pub async fn move_section(
        &self,
        actor: &Actor,
        id: SectionId,
        parent: Option<SectionId>,
        place: Place<SectionId>,
        expected_version: Option<i32>,
    ) -> Result<Section> {
        let mut tx = self.db.begin().await?;
        let stored = self.read_section_in(&mut tx, id, true).await?;
        self.access.require(actor, SECTION_MANAGE, stored.scope).await?;
        check_version(expected_version, stored.version, "courses.section_changed")?;
        let course = self.read_course_in(&mut tx, stored.course, false).await?;
        let target = match parent {
            Some(p) => self.parent_section_in(&mut tx, &course, p).await?.scope,
            None => course.scope,
        };
        self.access.require(actor, SECTION_MANAGE, target).await?;
        if parent != stored.parent {
            self.access.move_scope_in(&mut tx, stored.scope, target).await?;
        }
        let siblings = self
            .section_siblings_in(&mut tx, course.id, parent, Some(id))
            .await?;
        let key = sort_key::place(&siblings, place)?;
        let row = sqlx::query(sql(format!(
            "update {} set parent_id = $2, sort_key = $3, version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("section")
        )))
        .bind(id.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .bind(&key)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(section(&row))
    }

    /// Deletes a section with the sections nested in it, keeping them recoverable with
    /// [`Courses::restore_section`]; needs `courses.section.manage` in it.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `precondition_failed` when it changed since it was
    /// read.
    pub async fn delete_section(
        &self,
        actor: &Actor,
        id: SectionId,
        expected_version: Option<i32>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.read_section_in(&mut tx, id, true).await?;
        self.access.require(actor, SECTION_MANAGE, stored.scope).await?;
        check_version(expected_version, stored.version, "courses.section_changed")?;
        let sections = self.table("section");
        sqlx::query(sql(format!(
            "with recursive subtree as (
                 select id from {sections} where id = $1
                 union all select s.id from {sections} s join subtree t on s.parent_id = t.id
             )
             update {sections} set deleted_at = now(), version = version + 1
             where id in (select id from subtree) and deleted_at is null"
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Restores a deleted section with the sections deleted with it; its parent must not be deleted.
    /// Needs `courses.section.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found` for a section that is not deleted, `conflict` (`courses.section_parent_deleted`) while
    /// its parent is deleted, `permission_denied` without the permission.
    pub async fn restore_section(&self, actor: &Actor, id: SectionId) -> Result<Section> {
        let sections = self.table("section");
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "select s.scope_id, s.deleted_at, p.deleted_at as parent_deleted_at from {sections} s
             left join {sections} p on p.id = s.parent_id where s.id = $1 and s.deleted_at is not null for update of s"
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("courses.section_not_found"))?;
        let scope = ScopeId::from_uuid(row.get("scope_id"));
        self.access.require(actor, SECTION_MANAGE, scope).await?;
        if row
            .get::<Option<time::OffsetDateTime>, _>("parent_deleted_at")
            .is_some()
        {
            return Err(Error::conflict("courses.section_parent_deleted"));
        }
        let deleted_at: time::OffsetDateTime = row.get("deleted_at");
        sqlx::query(sql(format!(
            "with recursive subtree as (
                 select id from {sections} where id = $1
                 union all select s.id from {sections} s join subtree t on s.parent_id = t.id
             )
             update {sections} set deleted_at = null, version = version + 1
             where id in (select id from subtree) and deleted_at = $2"
        )))
        .bind(id.as_uuid())
        .bind(deleted_at)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let restored = self.read_section_in(&mut tx, id, false).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(restored)
    }

    /// Inserts a section with its scope; the last of its siblings unless placed.
    pub(crate) async fn insert_section_in(
        &self,
        conn: &mut PgConnection,
        course: &Course,
        parent: Option<&Section>,
        place: Option<Place<SectionId>>,
        fields: &SectionFields,
    ) -> Result<Section> {
        let parent_id = parent.map(|p| p.id);
        let siblings = self.section_siblings_in(conn, course.id, parent_id, None).await?;
        let key = sort_key::place(&siblings, place.unwrap_or_default())?;
        let scope = self
            .access
            .create_scope_in(conn, parent.map_or(course.scope, |p| p.scope), ScopeKind::Section)
            .await?;
        let (from, until) = window_of(fields.visibility);
        let row = sqlx::query(sql(format!(
            "insert into {} (id, scope_id, course_id, parent_id, title, summary, sort_key, visibility,
             visible_from, visible_until) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) returning {COLUMNS}",
            self.table("section")
        )))
        .bind(SectionId::generate().as_uuid())
        .bind(scope.as_uuid())
        .bind(course.id.as_uuid())
        .bind(parent_id.map(|p| p.as_uuid()))
        .bind(&fields.title)
        .bind(&fields.summary)
        .bind(&key)
        .bind(fields.visibility.mode())
        .bind(from)
        .bind(until)
        .fetch_one(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(section(&row))
    }

    /// Reads a section that is not deleted, locked for a change when `lock`.
    pub(crate) async fn read_section_in(
        &self,
        conn: &mut PgConnection,
        id: SectionId,
        lock: bool,
    ) -> Result<Section> {
        let lock = if lock { " for update" } else { "" };
        sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1 and deleted_at is null{lock}",
            self.table("section")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .map(|row| section(&row))
        .ok_or_else(|| Error::not_found("courses.section_not_found"))
    }

    /// A parent section, which must belong to the course.
    async fn parent_section_in(
        &self,
        conn: &mut PgConnection,
        course: &Course,
        id: SectionId,
    ) -> Result<Section> {
        let parent = self.read_section_in(conn, id, false).await?;
        if parent.course != course.id {
            return Err(Error::validation(
                "courses.section_parent_invalid",
                vec![FieldError::new("parent", "course")],
            ));
        }
        Ok(parent)
    }

    /// The sections under a parent, in order, without the one moving; deleted ones keep their keys.
    async fn section_siblings_in(
        &self,
        conn: &mut PgConnection,
        course: CourseId,
        parent: Option<SectionId>,
        moving: Option<SectionId>,
    ) -> Result<Vec<(SectionId, String)>> {
        let rows = sqlx::query(sql(format!(
            "select id, sort_key from {} where course_id = $1 and parent_id is not distinct from $2
             and id is distinct from $3 order by sort_key",
            self.table("section")
        )))
        .bind(course.as_uuid())
        .bind(parent.map(|p| p.as_uuid()))
        .bind(moving.map(|m| m.as_uuid()))
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (SectionId::from_uuid(r.get("id")), r.get("sort_key")))
            .collect())
    }
}
