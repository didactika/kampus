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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The kind of access condition progress provides: the state a person reached on another course module of
//! the course.

use kampus_activities::ModuleId;
use kampus_common::{Error, FieldError};
use kampus_module::{
    ConditionCheck, ConditionFuture, ConditionKindDeclaration, ConditionTarget, ConditionWrite,
};
use serde_json::{Value, json};
use sqlx::Row;

use crate::{Progress, sql};

/// The kind of the state of another course module.
pub const STATE: &str = "progress.state";

/// What a condition may ask of the other course module.
pub const REQUIRED: &[&str] = &["done", "not_done", "passed", "failed"];

/// The kinds of this module.
pub const KINDS: &[ConditionKindDeclaration] = &[ConditionKindDeclaration {
    code: STATE,
    summary: "Another course module of the course is done, not done, passed or failed",
    store,
    params,
    met,
    depending_on,
}];

/// A wrong parameter.
fn wrong(field: &str, rule: &str) -> Error {
    Error::validation("progress.condition_invalid", vec![FieldError::new(field, rule)])
}

/// Keeps the leaf: a course module of the same course and one of the required states.
fn store(write: ConditionWrite<'_>) -> ConditionFuture<'_, ()> {
    Box::pin(async move {
        let progress = write.services.service::<Progress>()?;
        let module = write
            .params
            .get("module")
            .and_then(Value::as_str)
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .ok_or_else(|| wrong("module", "required"))?;
        let required = write
            .params
            .get("state")
            .and_then(Value::as_str)
            .filter(|s| REQUIRED.contains(s))
            .ok_or_else(|| wrong("state", "one_of"))?;
        let course = progress
            .activities
            .course_of_module(ModuleId::from_uuid(module))
            .await
            .map_err(|_| wrong("module", "exists"))?;
        if course.as_uuid() != write.course {
            return Err(wrong("module", "same_course"));
        }
        sqlx::query(sql(format!(
            "insert into {} (condition_id, course_module_id, required) values ($1, $2, $3)",
            progress.table("condition_state")
        )))
        .bind(write.condition)
        .bind(module)
        .bind(required)
        .execute(&mut *write.conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    })
}

/// The course module and the required state of a leaf.
async fn leaf(
    progress: &Progress,
    condition: uuid::Uuid,
) -> kampus_common::Result<Option<(uuid::Uuid, String)>> {
    let row = sqlx::query(sql(format!(
        "select course_module_id, required from {} where condition_id = $1",
        progress.table("condition_state")
    )))
    .bind(condition)
    .fetch_optional(progress.db.pool())
    .await
    .map_err(kampus_db::classify)?;
    Ok(row.map(|r| (r.get("course_module_id"), r.get("required"))))
}

/// The parameters of a leaf.
fn params(check: ConditionCheck<'_>) -> ConditionFuture<'_, Option<Value>> {
    Box::pin(async move {
        let progress = check.services.service::<Progress>()?;
        Ok(leaf(&progress, check.condition)
            .await?
            .map(|(module, state)| json!({ "module": module, "state": state })))
    })
}

/// Whether the person reached the required state, read as it is kept, whatever the person may see of it.
fn met(check: ConditionCheck<'_>) -> ConditionFuture<'_, bool> {
    Box::pin(async move {
        let progress = check.services.service::<Progress>()?;
        let Some((module, required)) = leaf(&progress, check.condition).await? else {
            return Ok(false);
        };
        let state: Option<String> = sqlx::query_scalar(sql(format!(
            "select state from {} where course_module_id = $1 and account_id = $2",
            progress.table("activity_state")
        )))
        .bind(module)
        .bind(check.account)
        .fetch_optional(progress.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let state = state.unwrap_or_default();
        let done = matches!(state.as_str(), "done" | "done_passed" | "done_failed");
        Ok(match required.as_str() {
            "done" => done,
            "not_done" => !done,
            "passed" => state == "done_passed",
            "failed" => state == "done_failed",
            _ => false,
        })
    })
}

/// The leaves that ask about a course module.
fn depending_on(target: ConditionTarget<'_>) -> ConditionFuture<'_, Vec<uuid::Uuid>> {
    Box::pin(async move {
        let progress = target.services.service::<Progress>()?;
        sqlx::query_scalar(sql(format!(
            "select condition_id from {} where course_module_id = $1",
            progress.table("condition_state")
        )))
        .bind(target.module)
        .fetch_all(progress.db.pool())
        .await
        .map_err(kampus_db::classify)
    })
}
