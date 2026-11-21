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

//! Grades: written by idempotent upserts on a record and a person, each change kept as an entry, overridden
//! and locked by those who may, and read through one visibility rule that every consumer shares.

use std::collections::{HashMap, HashSet};

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::{Course, CourseId};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgConnection;

use super::{
    CATEGORY_COLUMNS, GRADE_COLUMNS, RECORD_COLUMNS, category, entry, fits_grade, grade, principal, problem,
    record,
};
use crate::permissions::{GRADE, HIDE, LOCK, OVERRIDE, VIEW_ALL, VIEW_HIDDEN, VIEW_OWN};
use crate::{
    Category, CategoryId, Entry, Grade, GradeChange, GradeType, GradeWrite, Record, RecordId, RecordKind,
    Records, Source, events, sql,
};

/// The most grades one call sets.
const BATCH_LIMIT: usize = 500;

/// The most characters of a text grade or of feedback.
const TEXT_LIMIT: usize = 2000;

/// The most people in a page of a report.
const PAGE_LIMIT: i64 = 200;

/// A grade with the entries that made it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Detail {
    /// The grade.
    pub grade: Grade,
    /// Its history, oldest first.
    pub entries: Vec<Entry>,
}

/// The grades of one person in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportRow {
    /// The person.
    pub account: PrincipalId,
    /// Whether they take part in the course now; the grades of those who left stay, out of the report
    /// unless asked for.
    pub active: bool,
    /// Their grades, those the viewer may see.
    pub grades: Vec<Grade>,
}

/// A page of the grader report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The people of the page, in the order of their ids.
    pub rows: Vec<ReportRow>,
    /// Where the next page starts, when there is one.
    pub next: Option<PrincipalId>,
}

/// What the viewer of a course may see.
#[derive(Debug, Clone, Copy)]
struct Viewer {
    /// The person, when it is one.
    me: Option<uuid::Uuid>,
    /// Whether they see their own grades.
    own: bool,
    /// Whether they see everyone's.
    all: bool,
    /// Whether they see what is hidden.
    hidden: bool,
}

impl Viewer {
    /// Whether the viewer sees the grade of a person on a record.
    fn sees(&self, record: &Record, grade: &Grade) -> bool {
        let now = Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc());
        let hidden = record.hidden_until.is_some_and(|until| until > now)
            || grade.hidden_until.is_some_and(|until| until > now);
        if hidden && !self.hidden {
            return false;
        }
        self.all || (self.own && self.me == Some(grade.account.as_uuid()))
    }
}

/// What counts of a raw grade: multiplied, offset and kept inside the range of the record.
fn counted(record: &Record, raw: Decimal) -> Decimal {
    (raw * record.multiplier + record.offset)
        .clamp(record.min, record.max)
        .round_dp(5)
}

/// A grade the record accepts: a number inside its range or a text, and the feedback cleaned up.
fn checked(record: &Record, write: &GradeWrite) -> Result<(Option<Decimal>, Option<String>, Option<String>)> {
    let mut problems = Vec::new();
    let mut raw = None;
    let mut text = None;
    match record.grade_type {
        GradeType::Value => match write.raw {
            Some(value) if fits_grade(value) && value >= record.min && value <= record.max => {
                raw = Some(value.round_dp(5));
            }
            _ => problems.push(problem("raw", "range")),
        },
        GradeType::Scale => match write.raw {
            Some(value) if value.fract().is_zero() && value >= record.min && value <= record.max => {
                raw = Some(value);
            }
            _ => problems.push(problem("raw", "item")),
        },
        GradeType::Text => match write.text.as_deref().map(str::trim) {
            Some(t) if !t.is_empty() && t.chars().count() <= TEXT_LIMIT => text = Some(t.to_owned()),
            _ => problems.push(problem("text", "length")),
        },
    }
    let feedback = write
        .feedback
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .map(str::to_owned);
    if feedback.as_ref().is_some_and(|f| f.chars().count() > TEXT_LIMIT) {
        problems.push(problem("feedback", "length"));
    }
    if problems.is_empty() {
        Ok((raw, text, feedback))
    } else {
        Err(Error::validation("records.grade_invalid", problems))
    }
}

impl Records {
    /// What the actor may see in a course.
    async fn viewer(&self, actor: &Actor, course: &Course) -> Result<Viewer> {
        Ok(Viewer {
            me: principal(actor),
            own: self.access.check(actor, VIEW_OWN, course.scope).await?.allowed,
            all: self.access.check(actor, VIEW_ALL, course.scope).await?.allowed,
            hidden: self.access.check(actor, VIEW_HIDDEN, course.scope).await?.allowed,
        })
    }

    /// The grade of a person on a record, locked for the change.
    async fn grade_in(
        &self,
        conn: &mut PgConnection,
        record: RecordId,
        account: PrincipalId,
        lock: bool,
    ) -> Result<Option<Grade>> {
        let row = sqlx::query(sql(format!(
            "select {GRADE_COLUMNS} from {} where record_id = $1 and account_id = $2 {}",
            self.table("grade"),
            if lock { "for update" } else { "" }
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.as_ref().map(grade))
    }

    /// Keeps a change of a grade.
    async fn add_entry_in(
        &self,
        conn: &mut PgConnection,
        record: RecordId,
        account: PrincipalId,
        content: (Option<Decimal>, Option<&str>, Option<&str>),
        source: Source,
        actor: Option<uuid::Uuid>,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, record_id, account_id, raw, \"text\", feedback, source, actor)
             values ($1, $2, $3, $4, $5, $6, $7, $8)",
            self.table("entry")
        )))
        .bind(crate::EntryId::generate().as_uuid())
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .bind(content.0)
        .bind(content.1)
        .bind(content.2)
        .bind(source.code())
        .bind(actor)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    }

    /// Writes one grade: an upsert on the record and the person that changes nothing when the same grade
    /// arrives again, and otherwise keeps an entry and publishes one event.
    async fn write_in(
        &self,
        conn: &mut PgConnection,
        course: &Course,
        stored: &Record,
        write: &GradeWrite,
        source: Source,
        actor: Option<uuid::Uuid>,
    ) -> Result<GradeChange> {
        if matches!(stored.kind, RecordKind::CategoryTotal | RecordKind::CourseTotal)
            || self.has_formula_in(&mut *conn, stored.id).await?
        {
            return Err(Error::validation(
                "records.record_derived",
                vec![problem("record", "derived")],
            ));
        }
        if stored.locked_at.is_some() {
            return Err(Error::conflict("records.locked"));
        }
        let (raw, text, feedback) = checked(stored, write)?;
        let counts = raw.map(|r| counted(stored, r));
        let row = sqlx::query(sql(format!(
            "insert into {t} (record_id, account_id, raw, \"final\", \"text\") values ($1, $2, $3, $4, $5)
             on conflict (record_id, account_id) do update set raw = excluded.raw, \"text\" = excluded.\"text\",
                 \"final\" = case when {t}.overridden then {t}.\"final\" else excluded.\"final\" end,
                 version = {t}.version + 1, updated_at = now()
             where {t}.locked_at is null
                 and ({t}.raw is distinct from excluded.raw or {t}.\"text\" is distinct from excluded.\"text\")
             returning {GRADE_COLUMNS}",
            t = self.table("grade")
        )))
        .bind(stored.id.as_uuid())
        .bind(write.account.as_uuid())
        .bind(raw)
        .bind(counts)
        .bind(text.as_deref())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(row) = row {
            self.add_entry_in(
                conn,
                stored.id,
                write.account,
                (raw, text.as_deref(), feedback.as_deref()),
                source,
                actor,
            )
            .await?;
            self.publish_in(
                conn,
                events::GRADE_CHANGED,
                course,
                stored.id,
                Some(write.account),
                actor,
            )
            .await?;
            self.queue_in(conn, course.id, &[write.account]).await?;
            return Ok(GradeChange {
                grade: grade(&row),
                changed: true,
            });
        }
        let current = self
            .grade_in(conn, stored.id, write.account, false)
            .await?
            .ok_or_else(|| Error::internal("records.grade_missing"))?;
        if current.locked_at.is_some() && (current.raw != raw || current.text != text) {
            return Err(Error::conflict("records.locked"));
        }
        Ok(GradeChange {
            grade: current,
            changed: false,
        })
    }

    /// Sets the grades of people on a record, all or none; needs `records.grade` in the course. The same
    /// grade sent twice changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`records.grade_invalid`, `records.record_derived`,
    /// `records.batch_too_large`), `conflict` (`records.locked`).
    pub async fn set_grades(
        &self,
        actor: &Actor,
        record: RecordId,
        writes: Vec<GradeWrite>,
        source: Source,
    ) -> Result<Vec<GradeChange>> {
        if writes.len() > BATCH_LIMIT {
            return Err(Error::validation(
                "records.batch_too_large",
                vec![problem("grades", "max")],
            ));
        }
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, GRADE, course.scope).await?;
        let source = if source == Source::Import {
            Source::Import
        } else {
            Source::Teacher
        };
        let mut tx = self.db.begin().await?;
        let stored = self.record_in(&mut tx, record, true).await?;
        let mut changes = Vec::with_capacity(writes.len());
        for write in &writes {
            changes.push(
                self.write_in(&mut tx, &course, &stored, write, source, principal(actor))
                    .await?,
            );
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(changes)
    }

    /// Sets the grade an activity gives to a person. It is meant for the module that provides the
    /// activity, which has already checked the submission; the same grade sent twice changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found`, `validation`, `conflict` (`records.locked`).
    pub async fn submit_activity_grade(&self, record: RecordId, write: GradeWrite) -> Result<GradeChange> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        let mut tx = self.db.begin().await?;
        let change = self
            .write_in(&mut tx, &course, &stored, &write, Source::Activity, None)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(change)
    }

    /// Overrides the grade of a person with a value that wins over later activity grades until it is
    /// removed; needs `records.override` in the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`records.grade_invalid`), `conflict`
    /// (`records.locked`).
    pub async fn override_grade(
        &self,
        actor: &Actor,
        record: RecordId,
        account: PrincipalId,
        value: Decimal,
        feedback: Option<&str>,
    ) -> Result<GradeChange> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, OVERRIDE, course.scope).await?;
        let counted_value = value.round_dp(5);
        if !fits_grade(value) || counted_value < stored.min || counted_value > stored.max {
            return Err(Error::validation(
                "records.grade_invalid",
                vec![problem("value", "range")],
            ));
        }
        let mut tx = self.db.begin().await?;
        let stored = self.record_in(&mut tx, record, true).await?;
        if stored.locked_at.is_some() {
            return Err(Error::conflict("records.locked"));
        }
        let current = self.grade_in(&mut tx, record, account, true).await?;
        if let Some(current) = &current {
            if current.locked_at.is_some() {
                return Err(Error::conflict("records.locked"));
            }
            if current.overridden && current.final_value == Some(counted_value) {
                return Ok(GradeChange {
                    grade: current.clone(),
                    changed: false,
                });
            }
        }
        let row = sqlx::query(sql(format!(
            "insert into {t} (record_id, account_id, \"final\", overridden) values ($1, $2, $3, true)
             on conflict (record_id, account_id) do update set \"final\" = excluded.\"final\", overridden = true,
                 version = {t}.version + 1, updated_at = now()
             returning {GRADE_COLUMNS}",
            t = self.table("grade")
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .bind(counted_value)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let feedback = feedback.map(str::trim).filter(|f| !f.is_empty());
        self.add_entry_in(
            &mut tx,
            record,
            account,
            (Some(counted_value), None, feedback),
            Source::Teacher,
            principal(actor),
        )
        .await?;
        self.publish_in(
            &mut tx,
            events::OVERRIDDEN,
            &course,
            record,
            Some(account),
            principal(actor),
        )
        .await?;
        self.queue_in(&mut tx, course.id, &[account]).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(GradeChange {
            grade: grade(&row),
            changed: true,
        })
    }

    /// Removes the override of a grade, which goes back to what its raw grade counts; needs
    /// `records.override` in the course.
    ///
    /// # Errors
    ///
    /// `not_found` (`records.grade_not_found`), `permission_denied`, `conflict` (`records.locked`).
    pub async fn clear_override(
        &self,
        actor: &Actor,
        record: RecordId,
        account: PrincipalId,
    ) -> Result<GradeChange> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, OVERRIDE, course.scope).await?;
        let mut tx = self.db.begin().await?;
        let stored = self.record_in(&mut tx, record, true).await?;
        let current = self
            .grade_in(&mut tx, record, account, true)
            .await?
            .ok_or_else(|| Error::not_found("records.grade_not_found"))?;
        if !current.overridden {
            return Ok(GradeChange {
                grade: current,
                changed: false,
            });
        }
        if stored.locked_at.is_some() || current.locked_at.is_some() {
            return Err(Error::conflict("records.locked"));
        }
        let counts = current.raw.map(|r| counted(&stored, r));
        let row = sqlx::query(sql(format!(
            "update {} set \"final\" = $3, overridden = false, version = version + 1, updated_at = now()
             where record_id = $1 and account_id = $2 returning {GRADE_COLUMNS}",
            self.table("grade")
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .bind(counts)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.add_entry_in(
            &mut tx,
            record,
            account,
            (current.raw, current.text.as_deref(), None),
            Source::Teacher,
            principal(actor),
        )
        .await?;
        self.publish_in(
            &mut tx,
            events::OVERRIDDEN,
            &course,
            record,
            Some(account),
            principal(actor),
        )
        .await?;
        self.queue_in(&mut tx, course.id, &[account]).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(GradeChange {
            grade: grade(&row),
            changed: true,
        })
    }

    /// Locks or unlocks the grade of a person; while it is locked it does not change; needs `records.lock`
    /// in the course.
    ///
    /// # Errors
    ///
    /// `not_found` (`records.grade_not_found`), `permission_denied`.
    pub async fn set_grade_locked(
        &self,
        actor: &Actor,
        record: RecordId,
        account: PrincipalId,
        locked: bool,
    ) -> Result<Grade> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, LOCK, course.scope).await?;
        let mut tx = self.db.begin().await?;
        let current = self
            .grade_in(&mut tx, record, account, true)
            .await?
            .ok_or_else(|| Error::not_found("records.grade_not_found"))?;
        if current.locked_at.is_some() == locked {
            return Ok(current);
        }
        let row = sqlx::query(sql(format!(
            "update {} set locked_at = case when $3 then now() else null end, version = version + 1
             where record_id = $1 and account_id = $2 returning {GRADE_COLUMNS}",
            self.table("grade")
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .bind(locked)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            events::LOCKED,
            &course,
            record,
            Some(account),
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(grade(&row))
    }

    /// Hides the grade of a person until a date, or shows it again; needs `records.hide` in the course.
    ///
    /// # Errors
    ///
    /// `not_found` (`records.grade_not_found`), `permission_denied`.
    pub async fn set_grade_hidden(
        &self,
        actor: &Actor,
        record: RecordId,
        account: PrincipalId,
        until: Option<Timestamp>,
    ) -> Result<Grade> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, HIDE, course.scope).await?;
        let row = sqlx::query(sql(format!(
            "update {} set hidden_until = $3, version = version + 1
             where record_id = $1 and account_id = $2 returning {GRADE_COLUMNS}",
            self.table("grade")
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .bind(until.map(|t| t.as_offset_date_time()))
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("records.grade_not_found"))?;
        Ok(grade(&row))
    }

    /// Excludes the grade of a person from the totals, or counts it again; needs `records.override` in the
    /// course.
    ///
    /// # Errors
    ///
    /// `not_found` (`records.grade_not_found`), `permission_denied`.
    pub async fn set_grade_excluded(
        &self,
        actor: &Actor,
        record: RecordId,
        account: PrincipalId,
        excluded: bool,
    ) -> Result<Grade> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, OVERRIDE, course.scope).await?;
        let mut tx = self.db.begin().await?;
        let current = self
            .grade_in(&mut tx, record, account, true)
            .await?
            .ok_or_else(|| Error::not_found("records.grade_not_found"))?;
        if current.excluded == excluded {
            return Ok(current);
        }
        let row = sqlx::query(sql(format!(
            "update {} set excluded = $3, version = version + 1, updated_at = now()
             where record_id = $1 and account_id = $2 returning {GRADE_COLUMNS}",
            self.table("grade")
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .bind(excluded)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            events::GRADE_CHANGED,
            &course,
            record,
            Some(account),
            principal(actor),
        )
        .await?;
        self.queue_in(&mut tx, course.id, &[account]).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(grade(&row))
    }

    /// The grades of some people in a course, all of them.
    async fn grades_in_course(&self, course: CourseId, accounts: &[uuid::Uuid]) -> Result<Vec<Grade>> {
        let rows = sqlx::query(sql(format!(
            "select g.record_id, g.account_id, g.raw, g.\"final\", g.\"text\", g.overridden, g.excluded,
             g.hidden_until, g.locked_at, g.version from {} g join {} r on r.id = g.record_id
             where r.course_id = $1 and g.account_id = any($2) order by g.account_id, r.sort_key, g.record_id",
            self.table("grade"),
            self.table("record")
        )))
        .bind(course.as_uuid())
        .bind(accounts)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(grade).collect())
    }

    /// The grades the viewer may see out of those of some people: what is hidden from them is left out, and
    /// so is a total that adds up something hidden, so a hidden grade never leaks through a total. The
    /// grades must be all those of each person in the course.
    async fn visible_grades(&self, viewer: &Viewer, course: CourseId, all: Vec<Grade>) -> Result<Vec<Grade>> {
        let rows = sqlx::query(sql(format!(
            "select {RECORD_COLUMNS} from {} where course_id = $1",
            self.table("record")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let records: HashMap<RecordId, Record> = rows.iter().map(record).map(|r| (r.id, r)).collect();
        let hidden_totals = if viewer.hidden {
            HashSet::new()
        } else {
            let categories = sqlx::query(sql(format!(
                "select {CATEGORY_COLUMNS} from {} where course_id = $1",
                self.table("category")
            )))
            .bind(course.as_uuid())
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            let categories: Vec<Category> = categories.iter().map(category).collect();
            totals_with_hidden(&categories, &records, &all)
        };
        Ok(all
            .into_iter()
            .filter(|g| {
                records.get(&g.record).is_some_and(|r| viewer.sees(r, g))
                    && !hidden_totals.contains(&(g.account, g.record))
            })
            .collect())
    }

    /// One grade with its entries, as the actor may see it: their own when it is shown, anyone's with
    /// `records.view_all`, hidden ones only with `records.view_hidden`.
    ///
    /// # Errors
    ///
    /// `not_found` (`records.grade_not_found`) for a grade that does not exist or that the actor may not see.
    pub async fn grade_detail(
        &self,
        actor: &Actor,
        record: RecordId,
        account: PrincipalId,
    ) -> Result<Detail> {
        let stored = self.stored_record(record).await?;
        let course = self.courses.course(stored.course).await?;
        self.require_view(actor, &course).await?;
        let viewer = self.viewer(actor, &course).await?;
        let all = self.grades_in_course(course.id, &[account.as_uuid()]).await?;
        let found = self
            .visible_grades(&viewer, course.id, all)
            .await?
            .into_iter()
            .find(|g| g.record == record)
            .ok_or_else(|| Error::not_found("records.grade_not_found"))?;
        let rows = sqlx::query(sql(format!(
            "select id, record_id, account_id, raw, \"text\", feedback, source, actor, \"at\" from {}
             where record_id = $1 and account_id = $2 order by \"at\", id",
            self.table("entry")
        )))
        .bind(record.as_uuid())
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(Detail {
            grade: found,
            entries: rows.iter().map(entry).collect(),
        })
    }

    /// The grades of a person in a course that the actor may see; a person sees their own, and someone with
    /// `records.view_all` sees anyone's.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn grades_for(
        &self,
        actor: &Actor,
        course: CourseId,
        account: PrincipalId,
    ) -> Result<Vec<Grade>> {
        let course = self.courses.course(course).await?;
        self.require_view(actor, &course).await?;
        let viewer = self.viewer(actor, &course).await?;
        let all = self.grades_in_course(course.id, &[account.as_uuid()]).await?;
        self.visible_grades(&viewer, course.id, all).await
    }

    /// A page of the grader report: the people with grades in a course, each with the grades the actor may
    /// see, ordered by person; needs `records.view_all`. The grades of people who no longer take part stay,
    /// and they are left out unless `include_inactive` asks for them, marked as not active.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn grader_report(
        &self,
        actor: &Actor,
        course: CourseId,
        limit: i64,
        after: Option<PrincipalId>,
        include_inactive: bool,
    ) -> Result<Report> {
        let course = self.courses.course(course).await?;
        if !self.courses.sees(actor, course.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        self.access.require(actor, VIEW_ALL, course.scope).await?;
        let viewer = self.viewer(actor, &course).await?;
        let limit = usize::try_from(limit.clamp(1, PAGE_LIMIT)).unwrap_or(1);
        let mut chosen: Vec<(uuid::Uuid, bool)> = Vec::new();
        let mut cursor = after.map(|a| a.as_uuid());
        let mut exhausted = false;
        while chosen.len() <= limit && !exhausted {
            let candidates: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
                "select distinct g.account_id from {} g join {} r on r.id = g.record_id
                 where r.course_id = $1 and ($2::uuid is null or g.account_id > $2) order by g.account_id limit $3",
                self.table("grade"),
                self.table("record")
            )))
            .bind(course.id.as_uuid())
            .bind(cursor)
            .bind(i64::try_from(limit + 1).unwrap_or(PAGE_LIMIT))
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            exhausted = candidates.len() <= limit;
            cursor = candidates.last().copied();
            for id in candidates {
                let active = self
                    .courses
                    .takes_part(course.id, PrincipalId::from_uuid(id))
                    .await?;
                if active || include_inactive {
                    chosen.push((id, active));
                }
            }
        }
        let more = chosen.len() > limit;
        chosen.truncate(limit);
        let page: Vec<uuid::Uuid> = chosen.iter().map(|(id, _)| *id).collect();
        let all = self.grades_in_course(course.id, &page).await?;
        let mut report: Vec<ReportRow> = chosen
            .iter()
            .map(|(id, active)| ReportRow {
                account: PrincipalId::from_uuid(*id),
                active: *active,
                grades: Vec::new(),
            })
            .collect();
        for found in self.visible_grades(&viewer, course.id, all).await? {
            if let Some(row) = report.iter_mut().find(|r| r.account == found.account) {
                row.grades.push(found);
            }
        }
        let next = if more {
            report.last().map(|r| r.account)
        } else {
            None
        };
        Ok(Report { rows: report, next })
    }
}

/// The totals each person may not see because they add up a record or a grade that is hidden: a total
/// contains every record of its category and of the categories below it.
fn totals_with_hidden(
    categories: &[Category],
    records: &HashMap<RecordId, Record>,
    grades: &[Grade],
) -> HashSet<(PrincipalId, RecordId)> {
    let now = Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc());
    let hidden_grades: HashSet<(PrincipalId, RecordId)> = grades
        .iter()
        .filter(|g| g.hidden_until.is_some_and(|until| until > now))
        .map(|g| (g.account, g.record))
        .collect();
    let mut below: HashMap<CategoryId, Vec<CategoryId>> = HashMap::new();
    for category in categories {
        if let Some(parent) = category.parent {
            below.entry(parent).or_default().push(category.id);
        }
    }
    let people: HashSet<PrincipalId> = grades.iter().map(|g| g.account).collect();
    let mut hidden = HashSet::new();
    for total in records.values().filter(|r| r.total_of.is_some()) {
        let Some(root) = total.total_of else { continue };
        let mut inside = vec![root];
        let mut next = 0;
        while next < inside.len() {
            if let Some(children) = below.get(&inside[next]) {
                inside.extend(children.iter().copied());
            }
            next += 1;
        }
        let contained: Vec<&Record> = records
            .values()
            .filter(|r| r.category.is_some_and(|c| inside.contains(&c)))
            .collect();
        let hidden_for_all = contained
            .iter()
            .any(|r| r.hidden_until.is_some_and(|until| until > now));
        for person in &people {
            if hidden_for_all || contained.iter().any(|r| hidden_grades.contains(&(*person, r.id))) {
                hidden.insert((*person, total.id));
            }
        }
    }
    hidden
}
