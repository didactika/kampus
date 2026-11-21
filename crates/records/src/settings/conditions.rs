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

//! The kind of access condition records provides: a grade of the course, as a share of its range, at least
//! a minimum, below a maximum, or both.

use kampus_common::{Error, FieldError, Result};
use kampus_module::{
    ConditionCheck, ConditionFuture, ConditionKindDeclaration, ConditionWrite, nothing_depends,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::Row;

use crate::{Records, sql};

/// The kind of a grade.
pub const GRADE: &str = "records.grade";

/// The kinds of this module.
pub const KINDS: &[ConditionKindDeclaration] = &[ConditionKindDeclaration {
    code: GRADE,
    summary: "A grade of the course is at least a percentage of its range, below one, or between both",
    store,
    params,
    met,
    depending_on: nothing_depends,
}];

/// A wrong parameter.
fn wrong(field: &str, rule: &str) -> Error {
    Error::validation("records.condition_invalid", vec![FieldError::new(field, rule)])
}

/// A percentage among the parameters, when given.
fn percent(params: &Value, field: &str) -> Result<Option<Decimal>> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let parsed: Decimal =
                serde_json::from_value(value.clone()).map_err(|_| wrong(field, "format"))?;
            if parsed < Decimal::ZERO || parsed > Decimal::ONE_HUNDRED {
                return Err(wrong(field, "range"));
            }
            Ok(Some(parsed))
        }
    }
}

/// Keeps the leaf: a record of the same course and at least one bound.
fn store(write: ConditionWrite<'_>) -> ConditionFuture<'_, ()> {
    Box::pin(async move {
        let records = write.services.service::<Records>()?;
        let record = write
            .params
            .get("record")
            .and_then(Value::as_str)
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .ok_or_else(|| wrong("record", "required"))?;
        let min = percent(write.params, "min")?;
        let max = percent(write.params, "max")?;
        if min.is_none() && max.is_none() {
            return Err(wrong("min", "required"));
        }
        if let (Some(min), Some(max)) = (min, max)
            && max <= min
        {
            return Err(wrong("max", "after"));
        }
        let course: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select course_id from {} where id = $1",
            records.table("record")
        )))
        .bind(record)
        .fetch_optional(&mut *write.conn)
        .await
        .map_err(kampus_db::classify)?;
        match course {
            None => return Err(wrong("record", "exists")),
            Some(course) if course != write.course => return Err(wrong("record", "same_course")),
            Some(_) => {}
        }
        sqlx::query(sql(format!(
            "insert into {} (condition_id, record_id, min_percent, max_percent) values ($1, $2, $3, $4)",
            records.table("condition_grade")
        )))
        .bind(write.condition)
        .bind(record)
        .bind(min)
        .bind(max)
        .execute(&mut *write.conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    })
}

/// The record and the bounds of a leaf.
async fn leaf(
    records: &Records,
    condition: uuid::Uuid,
) -> Result<Option<(uuid::Uuid, Option<Decimal>, Option<Decimal>)>> {
    let row = sqlx::query(sql(format!(
        "select record_id, min_percent, max_percent from {} where condition_id = $1",
        records.table("condition_grade")
    )))
    .bind(condition)
    .fetch_optional(records.db.pool())
    .await
    .map_err(kampus_db::classify)?;
    Ok(row.map(|r| {
        (
            r.get("record_id"),
            r.get::<Option<Decimal>, _>("min_percent").map(|d| d.normalize()),
            r.get::<Option<Decimal>, _>("max_percent").map(|d| d.normalize()),
        )
    }))
}

/// The parameters of a leaf.
fn params(check: ConditionCheck<'_>) -> ConditionFuture<'_, Option<Value>> {
    Box::pin(async move {
        let records = check.services.service::<Records>()?;
        Ok(leaf(&records, check.condition)
            .await?
            .map(|(record, min, max)| json!({ "record": record, "min": min, "max": max })))
    })
}

/// Whether the person has a grade on the record whose share of its range is at least the minimum and below
/// the maximum; no grade never holds. The grade is read as it is kept, whatever the person may see of it.
fn met(check: ConditionCheck<'_>) -> ConditionFuture<'_, bool> {
    Box::pin(async move {
        let records = check.services.service::<Records>()?;
        let Some((record, min, max)) = leaf(&records, check.condition).await? else {
            return Ok(false);
        };
        let row = sqlx::query(sql(format!(
            "select r.\"min\", r.\"max\", g.\"final\" from {} r
             join {} g on g.record_id = r.id and g.account_id = $2 and not g.excluded
             where r.id = $1",
            records.table("record"),
            records.table("grade")
        )))
        .bind(record)
        .bind(check.account)
        .fetch_optional(records.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let Some(row) = row else { return Ok(false) };
        let Some(value) = row.get::<Option<Decimal>, _>("final") else {
            return Ok(false);
        };
        let (low, high): (Decimal, Decimal) = (row.get("min"), row.get("max"));
        if high <= low {
            return Ok(false);
        }
        let share = (value - low) / (high - low) * Decimal::ONE_HUNDRED;
        Ok(min.is_none_or(|m| share >= m) && max.is_none_or(|m| share < m))
    })
}
