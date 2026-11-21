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

//! The service API of records, one file per area, and what they share: reading categories, records and
//! grades from their rows, the numbers a record accepts, and the events.

pub mod category;
pub mod export;
pub mod formula;
pub mod grade;
pub mod recalc;
pub mod record;
pub mod scale;

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, ErrorKind, FieldError, Result, Timestamp};
use kampus_courses::{Course, CourseId};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use crate::{
    Category, CategoryId, Entry, Grade, GradeType, Method, Record, RecordId, RecordKind, Records, Source, sql,
};

/// The columns of a category.
pub(crate) const CATEGORY_COLUMNS: &str =
    "id, course_id, parent_id, name, method, drop_lowest, keep_highest, exclude_empty, sort_key, version";

/// The columns of a grade record.
pub(crate) const RECORD_COLUMNS: &str =
    "id, course_id, kind, title, course_module_id, item_number, category_id,
    total_of_id, grade_type, scale_id, \"min\", \"max\", pass_grade, weight, extra_credit, multiplier, offset_value,
    hidden_until, locked_at, sort_key, version";

/// The columns of a grade.
pub(crate) const GRADE_COLUMNS: &str =
    "record_id, account_id, raw, \"final\", \"text\", overridden, excluded, hidden_until, locked_at, version";

/// The largest number of a grade or a limit: `numeric(12, 5)` holds seven digits before the point.
const GRADE_LIMIT: i64 = 9_999_999;

/// The largest weight or multiplier: `numeric(9, 5)` holds four digits before the point.
const WEIGHT_LIMIT: i64 = 9_999;

/// A number without the zeros the column adds after the point.
fn clean(value: Decimal) -> Decimal {
    value.normalize()
}

/// An instant read from a nullable column.
fn instant(row: &PgRow, column: &str) -> Option<Timestamp> {
    row.get::<Option<time::OffsetDateTime>, _>(column)
        .map(Timestamp::from_offset_date_time)
}

/// A category from its row.
pub(crate) fn category(row: &PgRow) -> Category {
    Category {
        id: CategoryId::from_uuid(row.get("id")),
        course: CourseId::from_uuid(row.get("course_id")),
        parent: row
            .get::<Option<uuid::Uuid>, _>("parent_id")
            .map(CategoryId::from_uuid),
        name: row.get("name"),
        method: Method::from_code(&row.get::<String, _>("method")),
        drop_lowest: row.get("drop_lowest"),
        keep_highest: row.get("keep_highest"),
        exclude_empty: row.get("exclude_empty"),
        sort_key: row.get("sort_key"),
        version: row.get("version"),
    }
}

/// A grade record from its row.
pub(crate) fn record(row: &PgRow) -> Record {
    Record {
        id: RecordId::from_uuid(row.get("id")),
        course: CourseId::from_uuid(row.get("course_id")),
        kind: RecordKind::from_code(&row.get::<String, _>("kind")),
        title: row.get("title"),
        course_module: row
            .get::<Option<uuid::Uuid>, _>("course_module_id")
            .map(kampus_activities::ModuleId::from_uuid),
        item_number: row.get("item_number"),
        category: row
            .get::<Option<uuid::Uuid>, _>("category_id")
            .map(CategoryId::from_uuid),
        total_of: row
            .get::<Option<uuid::Uuid>, _>("total_of_id")
            .map(CategoryId::from_uuid),
        grade_type: GradeType::from_code(&row.get::<String, _>("grade_type")),
        scale: row
            .get::<Option<uuid::Uuid>, _>("scale_id")
            .map(crate::ScaleId::from_uuid),
        min: clean(row.get("min")),
        max: clean(row.get("max")),
        pass_grade: row.get::<Option<Decimal>, _>("pass_grade").map(clean),
        weight: clean(row.get("weight")),
        extra_credit: row.get("extra_credit"),
        multiplier: clean(row.get("multiplier")),
        offset: clean(row.get("offset_value")),
        hidden_until: instant(row, "hidden_until"),
        locked_at: instant(row, "locked_at"),
        sort_key: row.get("sort_key"),
        version: row.get("version"),
    }
}

/// A grade from its row.
pub(crate) fn grade(row: &PgRow) -> Grade {
    Grade {
        record: RecordId::from_uuid(row.get("record_id")),
        account: PrincipalId::from_uuid(row.get("account_id")),
        raw: row.get::<Option<Decimal>, _>("raw").map(clean),
        final_value: row.get::<Option<Decimal>, _>("final").map(clean),
        text: row.get("text"),
        overridden: row.get("overridden"),
        excluded: row.get("excluded"),
        hidden_until: instant(row, "hidden_until"),
        locked_at: instant(row, "locked_at"),
        version: row.get("version"),
    }
}

/// An entry from its row.
pub(crate) fn entry(row: &PgRow) -> Entry {
    Entry {
        id: crate::EntryId::from_uuid(row.get("id")),
        record: RecordId::from_uuid(row.get("record_id")),
        account: PrincipalId::from_uuid(row.get("account_id")),
        raw: row.get::<Option<Decimal>, _>("raw").map(clean),
        text: row.get("text"),
        feedback: row.get("feedback"),
        source: Source::from_code(&row.get::<String, _>("source")),
        actor: row.get("actor"),
        at: Timestamp::from_offset_date_time(row.get("at")),
    }
}

/// The error of a change made on another version than the stored one.
pub(crate) fn changed(code: &'static str, stored: i32) -> Error {
    Error::new(ErrorKind::PreconditionFailed, code).with_param("version", stored.to_string())
}

/// Whether a number fits `numeric(12, 5)`.
pub(crate) fn fits_grade(value: Decimal) -> bool {
    value.abs() <= Decimal::from(GRADE_LIMIT)
}

/// Whether a number fits `numeric(9, 5)`.
pub(crate) fn fits_weight(value: Decimal) -> bool {
    value.abs() <= Decimal::from(WEIGHT_LIMIT)
}

/// The principal behind an actor.
pub(crate) const fn principal(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(p.as_uuid()),
        Actor::Visitor(_) => None,
    }
}

/// A field problem for a name.
pub(crate) fn problem(field: &str, rule: &str) -> FieldError {
    FieldError::new(field, rule)
}

impl Records {
    /// A grade record in any state, whoever asks.
    pub(crate) async fn record_in(
        &self,
        conn: &mut PgConnection,
        id: RecordId,
        lock: bool,
    ) -> Result<Record> {
        let row = sqlx::query(sql(format!(
            "select {RECORD_COLUMNS} from {} where id = $1 {}",
            self.table("record"),
            if lock { "for update" } else { "" }
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("records.record_not_found"))?;
        Ok(record(&row))
    }

    /// A grade record, whoever asks.
    pub(crate) async fn stored_record(&self, id: RecordId) -> Result<Record> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.record_in(&mut conn, id, false).await
    }

    /// A category in any state, whoever asks.
    pub(crate) async fn category_in(
        &self,
        conn: &mut PgConnection,
        id: CategoryId,
        lock: bool,
    ) -> Result<Category> {
        let row = sqlx::query(sql(format!(
            "select {CATEGORY_COLUMNS} from {} where id = $1 {}",
            self.table("category"),
            if lock { "for update" } else { "" }
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("records.category_not_found"))?;
        Ok(category(&row))
    }

    /// Publishes an event about a record, and about a person when there is one, in the transaction of the
    /// change.
    pub(crate) async fn publish_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        course: &Course,
        record: RecordId,
        account: Option<PrincipalId>,
        actor: Option<uuid::Uuid>,
    ) -> Result<()> {
        let mut payload = json!({ "record": record, "course": course.id });
        if let Some(account) = account {
            payload["account"] = json!(account);
        }
        let mut event = kampus_events::Publish::new(code, Some(record.as_uuid()), payload);
        event.scope = Some(course.scope.as_uuid());
        event.actor = actor;
        event.summary = Some(format!("{code} {record}"));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }
}
