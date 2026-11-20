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

//! Grade records: the manual ones a teacher defines, the ones an activity declares for each grade it gives,
//! their settings with the permission each setting needs, and their lock.

use kampus_access::Actor;
use kampus_activities::ModuleId;
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::support::sort_key;
use kampus_courses::{Course, Place};
use rust_decimal::Decimal;
use sqlx::PgConnection;

use super::{RECORD_COLUMNS, changed, fits_grade, fits_weight, principal, problem, record};
use crate::permissions::{HIDE, LOCK, MANAGE};
use crate::{CategoryId, GradeType, Record, RecordFields, RecordId, RecordKind, Records, events, sql};

/// The most characters of a title.
const TITLE_LIMIT: usize = 200;

/// The fields of a record, checked: the title cleaned up and the numbers in their ranges.
fn checked(fields: &RecordFields) -> Result<String> {
    let mut problems = Vec::new();
    let title = fields.title.trim();
    if title.is_empty() || title.chars().count() > TITLE_LIMIT {
        problems.push(problem("title", "length"));
    }
    if !fits_grade(fields.min) {
        problems.push(problem("min", "range"));
    }
    if !fits_grade(fields.max) {
        problems.push(problem("max", "range"));
    }
    if fields.max <= fields.min {
        problems.push(problem("max", "after_min"));
    }
    if fields
        .pass_grade
        .is_some_and(|p| p < fields.min || p > fields.max)
    {
        problems.push(problem("pass_grade", "between_min_and_max"));
    }
    if fields.weight < Decimal::ZERO || !fits_weight(fields.weight) {
        problems.push(problem("weight", "range"));
    }
    if fields.multiplier <= Decimal::ZERO || !fits_weight(fields.multiplier) {
        problems.push(problem("multiplier", "range"));
    }
    if !fits_grade(fields.offset) {
        problems.push(problem("offset", "range"));
    }
    if problems.is_empty() {
        Ok(title.to_owned())
    } else {
        Err(Error::validation("records.record_invalid", problems))
    }
}

impl Records {
    /// The place a new record takes in a category.
    async fn next_key_in(&self, conn: &mut PgConnection, category: CategoryId) -> Result<String> {
        let siblings = self.record_siblings_in(conn, category).await?;
        sort_key::place(&siblings, Place::Last)
    }

    /// Adds a manual grade record to a category, or to the root of the course; needs `records.manage` in
    /// the course, and `records.hide` to hide it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`records.record_invalid`).
    pub async fn define_record(
        &self,
        actor: &Actor,
        course: kampus_courses::CourseId,
        category: Option<CategoryId>,
        fields: RecordFields,
    ) -> Result<Record> {
        let course = self.courses.course(course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        if fields.hidden_until.is_some() {
            self.access.require(actor, HIDE, course.scope).await?;
        }
        let fields = self.prepared(&course, fields).await?;
        let title = checked(&fields)?;
        let mut tx = self.db.begin().await?;
        let root = self.ensure_root_in(&mut tx, &course).await?;
        let category = category.unwrap_or(root.id);
        let holder = self.category_in(&mut tx, category, true).await?;
        if holder.course != course.id {
            return Err(Error::not_found("records.category_not_found"));
        }
        let key = self.next_key_in(&mut tx, category).await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, course_id, kind, title, category_id, grade_type, \"min\", \"max\", pass_grade,
             weight, extra_credit, multiplier, offset_value, hidden_until, sort_key, scale_id)
             values ($1, $2, 'manual', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
             returning {RECORD_COLUMNS}",
            self.table("record")
        )))
        .bind(RecordId::generate().as_uuid())
        .bind(course.id.as_uuid())
        .bind(&title)
        .bind(category.as_uuid())
        .bind(fields.grade_type.code())
        .bind(fields.min)
        .bind(fields.max)
        .bind(fields.pass_grade)
        .bind(fields.weight)
        .bind(fields.extra_credit)
        .bind(fields.multiplier)
        .bind(fields.offset)
        .bind(fields.hidden_until.map(|t| t.as_offset_date_time()))
        .bind(key)
        .bind(fields.scale.map(|s| s.as_uuid()))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let created = record(&row);
        self.publish_in(
            &mut tx,
            events::RECORD_CREATED,
            &course,
            created.id,
            None,
            principal(actor),
        )
        .await?;
        self.queue_course_in(&mut tx, course.id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// The grade record of one of the grades a course module gives, created the first time and kept after:
    /// declaring it again updates its title and range, and leaves what a teacher set on it. It is meant
    /// for the module that provides the activity, which has already checked who may configure it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module, `validation` (`records.record_invalid`).
    pub async fn declare_activity_record(
        &self,
        module: ModuleId,
        item_number: i32,
        fields: RecordFields,
    ) -> Result<Record> {
        if fields.grade_type == GradeType::Scale {
            return Err(Error::validation(
                "records.record_invalid",
                vec![problem("grade_type", "unsupported")],
            ));
        }
        let title = checked(&fields)?;
        let course = self
            .courses
            .course(self.activities.course_of_module(module).await?)
            .await?;
        let mut tx = self.db.begin().await?;
        let root = self.ensure_root_in(&mut tx, &course).await?;
        let key = self.next_key_in(&mut tx, root.id).await?;
        let row = sqlx::query(sql(format!(
            "insert into {table} (id, course_id, kind, title, course_module_id, item_number, category_id, grade_type,
             \"min\", \"max\", pass_grade, weight, extra_credit, multiplier, offset_value, sort_key)
             values ($1, $2, 'activity', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
             on conflict (course_module_id, item_number) do update set title = excluded.title,
                 \"min\" = excluded.\"min\", \"max\" = excluded.\"max\", pass_grade = excluded.pass_grade,
                 version = case when {table}.title is distinct from excluded.title
                     or {table}.\"min\" is distinct from excluded.\"min\"
                     or {table}.\"max\" is distinct from excluded.\"max\"
                     or {table}.pass_grade is distinct from excluded.pass_grade
                     then {table}.version + 1 else {table}.version end
             returning {RECORD_COLUMNS}, (xmax = 0) as inserted",
            table = self.table("record"),
        )))
        .bind(RecordId::generate().as_uuid())
        .bind(course.id.as_uuid())
        .bind(&title)
        .bind(module.as_uuid())
        .bind(item_number)
        .bind(root.id.as_uuid())
        .bind(fields.grade_type.code())
        .bind(fields.min)
        .bind(fields.max)
        .bind(fields.pass_grade)
        .bind(fields.weight)
        .bind(fields.extra_credit)
        .bind(fields.multiplier)
        .bind(fields.offset)
        .bind(key)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let declared = record(&row);
        if sqlx::Row::get::<bool, _>(&row, "inserted") {
            self.publish_in(&mut tx, events::RECORD_CREATED, &course, declared.id, None, None)
                .await?;
        }
        self.queue_course_in(&mut tx, course.id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(declared)
    }

    /// A grade record, for someone who sees the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied` without `records.view_own` or `records.view_all`.
    pub async fn record(&self, actor: &Actor, id: RecordId) -> Result<Record> {
        let found = self.stored_record(id).await?;
        let course = self.courses.course(found.course).await?;
        self.require_view(actor, &course).await?;
        if !self.can_see_record(actor, &course, &found).await? {
            return Err(Error::not_found("records.record_not_found"));
        }
        Ok(found)
    }

    /// Whether the actor sees a record: it is not hidden, or they hold `records.view_hidden`.
    pub(crate) async fn can_see_record(
        &self,
        actor: &Actor,
        course: &Course,
        found: &Record,
    ) -> Result<bool> {
        if found
            .hidden_until
            .is_none_or(|until| until <= Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc()))
        {
            return Ok(true);
        }
        Ok(self
            .access
            .check(actor, crate::permissions::VIEW_HIDDEN, course.scope)
            .await?
            .allowed)
    }

    /// Changes the settings of a grade record, on the version it was read at; needs `records.manage` in the
    /// course, and `records.hide` when the date it is hidden until changes. A locked record is not changed.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation`, `conflict` (`records.locked`) and
    /// `precondition_failed` (`records.record_changed`).
    pub async fn update_record(
        &self,
        actor: &Actor,
        id: RecordId,
        fields: RecordFields,
        expected_version: Option<i32>,
    ) -> Result<Record> {
        let mut tx = self.db.begin().await?;
        let stored = self.record_in(&mut tx, id, true).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        if fields.hidden_until != stored.hidden_until {
            self.access.require(actor, HIDE, course.scope).await?;
        }
        if stored.locked_at.is_some() {
            return Err(Error::conflict("records.locked"));
        }
        if expected_version.is_some_and(|v| v != stored.version) {
            return Err(changed("records.record_changed", stored.version));
        }
        let fields = self.prepared(&course, fields).await?;
        let title = checked(&fields)?;
        let derived = matches!(stored.kind, RecordKind::CategoryTotal | RecordKind::CourseTotal);
        let row = sqlx::query(sql(format!(
            "update {} set title = $2, grade_type = $3, \"min\" = $4, \"max\" = $5, pass_grade = $6, weight = $7,
             extra_credit = $8, multiplier = $9, offset_value = $10, hidden_until = $11, scale_id = $12,
             version = version + 1
             where id = $1 returning {RECORD_COLUMNS}",
            self.table("record")
        )))
        .bind(id.as_uuid())
        .bind(if derived { stored.title.as_str() } else { title.as_str() })
        .bind(fields.grade_type.code())
        .bind(fields.min)
        .bind(fields.max)
        .bind(fields.pass_grade)
        .bind(fields.weight)
        .bind(fields.extra_credit)
        .bind(fields.multiplier)
        .bind(fields.offset)
        .bind(fields.hidden_until.map(|t| t.as_offset_date_time()))
        .bind(fields.scale.map(|s| s.as_uuid()))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            events::RECORD_UPDATED,
            &course,
            id,
            None,
            principal(actor),
        )
        .await?;
        self.queue_course_in(&mut tx, course.id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(record(&row))
    }

    /// Deletes a manual grade record with its grades; needs `records.manage` in the course. Activity
    /// records go with their course module, and totals with their category.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `conflict` (`records.record_not_deletable`).
    pub async fn delete_record(&self, actor: &Actor, id: RecordId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.record_in(&mut tx, id, true).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, MANAGE, course.scope).await?;
        if stored.kind != RecordKind::Manual {
            return Err(Error::conflict("records.record_not_deletable"));
        }
        self.publish_in(
            &mut tx,
            events::RECORD_DELETED,
            &course,
            id,
            None,
            principal(actor),
        )
        .await?;
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("record"))))
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        self.queue_course_in(&mut tx, course.id).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Locks or unlocks a record: while it is locked its settings and its grades do not change; needs
    /// `records.lock` in the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn set_record_locked(&self, actor: &Actor, id: RecordId, locked: bool) -> Result<Record> {
        let mut tx = self.db.begin().await?;
        let stored = self.record_in(&mut tx, id, true).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, LOCK, course.scope).await?;
        if stored.locked_at.is_some() == locked {
            return Ok(stored);
        }
        let row = sqlx::query(sql(format!(
            "update {} set locked_at = case when $2 then now() else null end, version = version + 1
             where id = $1 returning {RECORD_COLUMNS}",
            self.table("record")
        )))
        .bind(id.as_uuid())
        .bind(locked)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(&mut tx, events::LOCKED, &course, id, None, principal(actor))
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(record(&row))
    }
}

impl Records {
    /// The scope of a course, for the permission checked before a change.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course.
    pub async fn course_scope(&self, course: kampus_courses::CourseId) -> Result<kampus_access::ScopeId> {
        Ok(self.courses.course(course).await?.scope)
    }

    /// The scope of the course of a grade record.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown record.
    pub async fn record_scope(&self, id: RecordId) -> Result<kampus_access::ScopeId> {
        let stored = self.stored_record(id).await?;
        self.course_scope(stored.course).await
    }

    /// The scope of the course of a category.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown category.
    pub async fn category_scope(&self, id: CategoryId) -> Result<kampus_access::ScopeId> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let stored = self.category_in(&mut conn, id, false).await?;
        self.course_scope(stored.course).await
    }
}

/// What a person has on the first grade a course module gives, for the modules that react to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityGrade {
    /// What counts of the grade; none when the person has none.
    pub value: Option<Decimal>,
    /// The lowest grade of the record.
    pub min: Decimal,
    /// The highest grade of the record.
    pub max: Decimal,
    /// The grade that passes, when the record has one.
    pub pass_grade: Option<Decimal>,
}

impl Records {
    /// What a person has on the first grade a course module gives; none when the course module has no
    /// grade record. It is meant for modules that react to grades, which show what people may see through
    /// [`Records::grades_for`].
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn activity_grade(
        &self,
        module: ModuleId,
        account: kampus_access::PrincipalId,
    ) -> Result<Option<ActivityGrade>> {
        let row = sqlx::query(sql(format!(
            "select r.\"min\", r.\"max\", r.pass_grade, g.\"final\" from {} r
             left join {} g on g.record_id = r.id and g.account_id = $2 and not g.excluded
             where r.course_module_id = $1 order by r.item_number limit 1",
            self.table("record"),
            self.table("grade")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.map(|r| ActivityGrade {
            value: sqlx::Row::get::<Option<Decimal>, _>(&r, "final").map(|d| d.normalize()),
            min: sqlx::Row::get::<Decimal, _>(&r, "min").normalize(),
            max: sqlx::Row::get::<Decimal, _>(&r, "max").normalize(),
            pass_grade: sqlx::Row::get::<Option<Decimal>, _>(&r, "pass_grade").map(|d| d.normalize()),
        }))
    }

    /// The course module a grade record belongs to, when it is the record of an activity.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn module_of_record(&self, record: RecordId) -> Result<Option<ModuleId>> {
        let module: Option<Option<uuid::Uuid>> = sqlx::query_scalar(sql(format!(
            "select course_module_id from {} where id = $1",
            self.table("record")
        )))
        .bind(record.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(module.flatten().map(ModuleId::from_uuid))
    }
}

impl Records {
    /// The people with a grade on the first grade a course module gives.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn graded_accounts(&self, module: ModuleId) -> Result<Vec<kampus_access::PrincipalId>> {
        let people: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select g.account_id from {} g join {} r on r.id = g.record_id where r.course_module_id = $1",
            self.table("grade"),
            self.table("record")
        )))
        .bind(module.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(people
            .into_iter()
            .map(kampus_access::PrincipalId::from_uuid)
            .collect())
    }

    /// The course modules of a course whose grade a person may see, as the actor may see it: the same rule
    /// as [`Records::grades_for`], so what depends on a grade is shown only when the grade is.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn modules_with_visible_grade(
        &self,
        actor: &Actor,
        course: kampus_courses::CourseId,
        account: kampus_access::PrincipalId,
    ) -> Result<std::collections::HashSet<ModuleId>> {
        let visible: std::collections::HashSet<RecordId> = self
            .grades_for(actor, course, account)
            .await?
            .into_iter()
            .map(|g| g.record)
            .collect();
        let rows = sqlx::query(sql(format!(
            "select id, course_module_id from {} where course_id = $1 and course_module_id is not null",
            self.table("record")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .filter(|r| visible.contains(&RecordId::from_uuid(sqlx::Row::get(*r, "id"))))
            .map(|r| ModuleId::from_uuid(sqlx::Row::get(r, "course_module_id")))
            .collect())
    }
}
