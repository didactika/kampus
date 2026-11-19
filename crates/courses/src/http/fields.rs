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

//! The routes of the custom fields of a course: its fields with the values the caller sees, and one value
//! set or cleared.

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_customise::permissions::FIELD_EDIT;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::{Shared, hidden};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/courses/{id}/fields", get(list))
        .route("/courses/{id}/fields/{code}", put(set))
}

/// The value of a field to set.
#[derive(Debug, Deserialize)]
struct Value {
    /// The value, or null to clear it.
    value: serde_json::Value,
}

/// The custom fields of a course the caller sees, with their values.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let course = courses.course(id(&course)?).await?;
    if !courses.sees(&caller.actor, course.scope).await? {
        return Err(hidden("courses.course_not_found").into());
    }
    Ok(Json(courses.course_fields(&caller.actor, course.id).await?).into_response())
}

/// Sets or clears the value of a custom field of a course.
async fn set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((course, code)): Path<(String, String)>,
    Body(body): Body<Value>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let course = courses.course(id(&course)?).await?;
    let actor = shared.acting(&caller, FIELD_EDIT, course.scope).await?;
    let entry = courses
        .set_course_field(actor, course.id, &code, &body.value)
        .await?;
    Ok(Json(entry).into_response())
}
