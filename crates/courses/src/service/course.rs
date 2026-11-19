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

//! Courses: created in a category with their first section, edited by their teachers, moved between
//! categories, and taken through their lifecycle, draft, published, archived and deletion scheduled.

use kampus_access::{Actor, PrincipalId, ScopeId, ScopeKind};
use kampus_common::{Error, FieldError, Result};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::{check_version, instant, optional, required, taken, valid, visibility_of, window_of};
use crate::permissions::{COURSE_ARCHIVE, COURSE_CREATE, COURSE_DELETE, COURSE_UPDATE};
use crate::support::sort_key;
use crate::{
    CategoryId, Course, CourseFields, CourseId, CourseState, Courses, DEFAULT_LAYOUT, DELETION_GRACE, Place,
    sql,
};

/// The columns of a course, as read.
const COLUMNS: &str = "id, scope_id, campus_id, category_id, short_name, full_name, code, summary, layout_code, \
    language_code, starts_at, ends_at, state, deletion_due_at, visibility, visible_from, visible_until, sort_key, \
    version";

/// The role its creator holds in a new course.
const OWNER_ROLE: &str = "teacher";

/// A course from its row.
fn course(row: &PgRow) -> Result<Course> {
    Ok(Course {
        id: CourseId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        campus: ScopeId::from_uuid(row.get("campus_id")),
        category: CategoryId::from_uuid(row.get("category_id")),
        short_name: row.get("short_name"),
        full_name: row.get("full_name"),
        code: row.get("code"),
        summary: row.get("summary"),
        layout: row.get("layout_code"),
        language_code: row.get("language_code"),
        starts_at: instant(row, "starts_at"),
        ends_at: instant(row, "ends_at"),
        state: row.get::<String, _>("state").parse()?,
        deletion_due_at: instant(row, "deletion_due_at"),
        visibility: visibility_of(row),
        sort_key: row.get("sort_key"),
        version: row.get("version"),
    })
}

/// Checks the fields of a course, every problem at once.
fn checked(fields: CourseFields) -> Result<CourseFields> {
    let mut problems = Vec::new();
    let short_name = required("short_name", &fields.short_name, &mut problems);
    let full_name = required("full_name", &fields.full_name, &mut problems);
    if let (Some(starts), Some(ends)) = (fields.starts_at, fields.ends_at)
        && ends <= starts
    {
        problems.push(FieldError::new("ends_at", "after_starts_at"));
    }
    let layout = optional(fields.layout);
    if layout
        .as_deref()
        .is_some_and(|l| !l.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
    {
        problems.push(FieldError::new("layout", "code"));
    }
    valid("courses.course_invalid", problems)?;
    Ok(CourseFields {
        short_name,
        full_name,
        code: optional(fields.code),
        summary: optional(fields.summary),
        layout,
        language_code: optional(fields.language_code),
        starts_at: fields.starts_at,
        ends_at: fields.ends_at,
        visibility: fields.visibility.validate()?,
    })
}

/// The principal of an actor, if it acts as one.
const fn principal(actor: &Actor) -> Option<PrincipalId> {
    match actor {
        Actor::Principal(p) => Some(*p),
        Actor::Visitor(_) | Actor::ViewAs(..) => None,
    }
}

impl Courses {
    /// Creates a course in a category, as a draft, with its first section; its creator becomes an owner
    /// and a teacher of it. Needs `courses.course.create` in the category.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields, `conflict` when the
    /// campus has a course with that short name or code.
    pub async fn create_course(
        &self,
        actor: &Actor,
        category: CategoryId,
        fields: CourseFields,
        place: Place<CourseId>,
    ) -> Result<Course> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let parent = self.read_category_in(&mut tx, category, false).await?;
        self.access.require(actor, COURSE_CREATE, parent.scope).await?;
        let siblings = self.course_siblings_in(&mut tx, category, None).await?;
        let key = sort_key::place(&siblings, place)?;
        let scope = self
            .access
            .create_scope_in(&mut tx, parent.scope, ScopeKind::Course)
            .await?;
        let (from, until) = window_of(fields.visibility);
        let row = sqlx::query(sql(format!(
            "insert into {} (id, scope_id, campus_id, category_id, short_name, full_name, code, summary,
             layout_code, language_code, starts_at, ends_at, visibility, visible_from, visible_until, sort_key)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) returning {COLUMNS}",
            self.table("course")
        )))
        .bind(CourseId::generate().as_uuid())
        .bind(scope.as_uuid())
        .bind(parent.campus.as_uuid())
        .bind(category.as_uuid())
        .bind(&fields.short_name)
        .bind(&fields.full_name)
        .bind(&fields.code)
        .bind(&fields.summary)
        .bind(fields.layout.as_deref().unwrap_or(DEFAULT_LAYOUT))
        .bind(&fields.language_code)
        .bind(fields.starts_at.map(|s| s.as_offset_date_time()))
        .bind(fields.ends_at.map(|e| e.as_offset_date_time()))
        .bind(fields.visibility.mode())
        .bind(from)
        .bind(until)
        .bind(&key)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken("courses.course_name_taken"))?;
        let created = course(&row)?;
        if let Some(owner) = principal(actor) {
            sqlx::query(sql(format!(
                "insert into {} (course_id, account_id) values ($1, $2)",
                self.table("course_owner")
            )))
            .bind(created.id.as_uuid())
            .bind(owner.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            let role = self
                .access
                .campus_role_in(&mut tx, parent.campus, OWNER_ROLE)
                .await?;
            self.access
                .grant_for_module_in(&mut tx, crate::module::NAME, role, scope, owner, None)
                .await?;
        }
        self.insert_section_in(&mut tx, &created, None, None, &crate::SectionFields::first())
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// A course, as stored.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course.
    pub async fn course(&self, id: CourseId) -> Result<Course> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.read_course_in(&mut conn, id, false).await
    }

    /// The courses of a category, in their order; hidden ones only for people with
    /// `courses.hidden.view`.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown category.
    pub async fn courses(&self, actor: &Actor, category: CategoryId) -> Result<Vec<Course>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where category_id = $1 order by sort_key",
            self.table("course")
        )))
        .bind(category.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let parent = self.category(category).await?;
        let all = rows.iter().map(course).collect::<Result<Vec<_>>>()?;
        self.visible_children(actor, parent.scope, all, |c: &Course| (c.scope, c.visibility))
            .await
    }

    /// The owners of a course.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn owners(&self, course: CourseId) -> Result<Vec<PrincipalId>> {
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select account_id from {} where course_id = $1 order by since, account_id",
            self.table("course_owner")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(ids.into_iter().map(PrincipalId::from_uuid).collect())
    }

    /// Changes the fields of a course; switching layout keeps every section. Needs
    /// `courses.course.update`; `expected_version` is the version read (`If-Match`).
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for invalid fields, `precondition_failed`
    /// when it changed since it was read, `conflict` for a short name or code in use.
    pub async fn update_course(
        &self,
        actor: &Actor,
        id: CourseId,
        fields: CourseFields,
        expected_version: Option<i32>,
    ) -> Result<Course> {
        let fields = checked(fields)?;
        let mut tx = self.db.begin().await?;
        let stored = self.read_course_in(&mut tx, id, true).await?;
        self.access.require(actor, COURSE_UPDATE, stored.scope).await?;
        check_version(expected_version, stored.version, "courses.course_changed")?;
        let (from, until) = window_of(fields.visibility);
        let row = sqlx::query(sql(format!(
            "update {} set short_name = $2, full_name = $3, code = $4, summary = $5, layout_code = $6,
             language_code = $7, starts_at = $8, ends_at = $9, visibility = $10, visible_from = $11,
             visible_until = $12, version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("course")
        )))
        .bind(id.as_uuid())
        .bind(&fields.short_name)
        .bind(&fields.full_name)
        .bind(&fields.code)
        .bind(&fields.summary)
        .bind(fields.layout.as_deref().unwrap_or(&stored.layout))
        .bind(&fields.language_code)
        .bind(fields.starts_at.map(|s| s.as_offset_date_time()))
        .bind(fields.ends_at.map(|e| e.as_offset_date_time()))
        .bind(fields.visibility.mode())
        .bind(from)
        .bind(until)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken("courses.course_name_taken"))?;
        tx.commit().await.map_err(kampus_db::classify)?;
        course(&row)
    }

    /// Moves a course to another category of its campus, or to another place in its own; needs
    /// `courses.course.update` in the course and `courses.course.create` where it goes.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permissions, `validation` for a category of another campus,
    /// `precondition_failed` when it changed since it was read.
    pub async fn move_course(
        &self,
        actor: &Actor,
        id: CourseId,
        category: CategoryId,
        place: Place<CourseId>,
        expected_version: Option<i32>,
    ) -> Result<Course> {
        let mut tx = self.db.begin().await?;
        let stored = self.read_course_in(&mut tx, id, true).await?;
        self.access.require(actor, COURSE_UPDATE, stored.scope).await?;
        check_version(expected_version, stored.version, "courses.course_changed")?;
        let target = self.read_category_in(&mut tx, category, false).await?;
        if target.campus != stored.campus {
            return Err(Error::validation(
                "courses.category_parent_invalid",
                vec![FieldError::new("category", "campus")],
            ));
        }
        self.access.require(actor, COURSE_CREATE, target.scope).await?;
        if category != stored.category {
            self.access
                .move_scope_in(&mut tx, stored.scope, target.scope)
                .await?;
        }
        let siblings = self.course_siblings_in(&mut tx, category, Some(id)).await?;
        let key = sort_key::place(&siblings, place)?;
        let row = sqlx::query(sql(format!(
            "update {} set category_id = $2, sort_key = $3, version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("course")
        )))
        .bind(id.as_uuid())
        .bind(category.as_uuid())
        .bind(&key)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        course(&row)
    }

    /// Takes a course to another state of its lifecycle:
    ///
    /// | From | To | Needs |
    /// | --- | --- | --- |
    /// | draft | published | `courses.course.update` |
    /// | published | draft | `courses.course.update`, and nobody taking part |
    /// | published | archived (its scope frozen) | `courses.course.archive` |
    /// | archived | published | `courses.course.archive` |
    /// | any other | deletion scheduled, after the grace period | `courses.course.delete`, or owning it |
    /// | deletion scheduled | draft | `courses.course.delete`, or owning it |
    ///
    /// # Errors
    ///
    /// `permission_denied` without what it needs, `conflict` (`courses.transition_invalid`) for a change
    /// the lifecycle does not allow or a published course with participants going back to draft,
    /// `precondition_failed` when it changed since it was read.
    pub async fn transition_course(
        &self,
        actor: &Actor,
        id: CourseId,
        to: CourseState,
        expected_version: Option<i32>,
    ) -> Result<Course> {
        use CourseState::{Archived, DeletionScheduled, Draft, Published};
        let mut tx = self.db.begin().await?;
        let stored = self.read_course_in(&mut tx, id, true).await?;
        check_version(expected_version, stored.version, "courses.course_changed")?;
        let refused = || {
            Error::conflict("courses.transition_invalid")
                .with_param("from", stored.state.as_str())
                .with_param("to", to.as_str())
        };
        match (stored.state, to) {
            (Draft, Published) => self.access.require(actor, COURSE_UPDATE, stored.scope).await?,
            (Published, Draft) => {
                self.access.require(actor, COURSE_UPDATE, stored.scope).await?;
                if self.participants.any(id).await? {
                    return Err(refused().with_param("reason", "participants"));
                }
            }
            (Published, Archived) => self.access.require(actor, COURSE_ARCHIVE, stored.scope).await?,
            (Archived, Published) => {
                self.access
                    .require_while_frozen(actor, COURSE_ARCHIVE, stored.scope)
                    .await?;
            }
            (Draft | Published | Archived, DeletionScheduled) | (DeletionScheduled, Draft) => {
                self.require_owner_or(actor, &stored, &mut tx).await?;
            }
            _ => return Err(refused()),
        }
        let frozen = to == Archived;
        if frozen || stored.state == Archived {
            self.access.set_frozen_in(&mut tx, stored.scope, frozen).await?;
        }
        let due = (to == DeletionScheduled).then(|| self.now().as_offset_date_time() + DELETION_GRACE);
        let row = sqlx::query(sql(format!(
            "update {} set state = $2, state_changed_at = now(), deletion_due_at = $3, version = version + 1
             where id = $1 returning {COLUMNS}",
            self.table("course")
        )))
        .bind(id.as_uuid())
        .bind(to.as_str())
        .bind(due)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        course(&row)
    }

    /// Requires `courses.course.delete`, which the owners of the course hold as owners.
    async fn require_owner_or(&self, actor: &Actor, course: &Course, conn: &mut PgConnection) -> Result<()> {
        if let Some(p) = principal(actor) {
            let owner: bool = sqlx::query_scalar(sql(format!(
                "select exists (select 1 from {} where course_id = $1 and account_id = $2)",
                self.table("course_owner")
            )))
            .bind(course.id.as_uuid())
            .bind(p.as_uuid())
            .fetch_one(conn)
            .await
            .map_err(kampus_db::classify)?;
            if owner {
                return Ok(());
            }
        }
        self.access
            .require_while_frozen(actor, COURSE_DELETE, course.scope)
            .await
    }

    /// Reads a course, locked for a change when `lock`.
    pub(crate) async fn read_course_in(
        &self,
        conn: &mut PgConnection,
        id: CourseId,
        lock: bool,
    ) -> Result<Course> {
        let lock = if lock { " for update" } else { "" };
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1{lock}",
            self.table("course")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("courses.course_not_found"))?;
        course(&row)
    }

    /// The courses of a category, in order, without the one moving.
    async fn course_siblings_in(
        &self,
        conn: &mut PgConnection,
        category: CategoryId,
        moving: Option<CourseId>,
    ) -> Result<Vec<(CourseId, String)>> {
        let rows = sqlx::query(sql(format!(
            "select id, sort_key from {} where category_id = $1 and id is distinct from $2 order by sort_key",
            self.table("course")
        )))
        .bind(category.as_uuid())
        .bind(moving.map(|m| m.as_uuid()))
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| (CourseId::from_uuid(r.get("id")), r.get("sort_key")))
            .collect())
    }
}
