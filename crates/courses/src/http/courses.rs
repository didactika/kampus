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

//! The routes of courses: in a category, one course with its owners, its fields, its state and its place.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use serde::Deserialize;
use serde_json::json;

use super::{Shared, hidden};
use crate::permissions::{COURSE_ARCHIVE, COURSE_CREATE, COURSE_DELETE, COURSE_UPDATE};
use crate::{CategoryId, CourseFields, CourseId, CourseState, Place};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/categories/{id}/courses", get(list).post(create))
        .route("/courses/{id}", get(read).put(update))
        .route("/courses/{id}/state", put(transition))
        .route("/courses/{id}/move", post(move_to))
}

/// A new course.
#[derive(Debug, Deserialize)]
struct NewCourse {
    /// Where among the courses of the category.
    #[serde(default)]
    place: Place<CourseId>,
    /// Its fields.
    #[serde(flatten)]
    fields: CourseFields,
}

/// A change of state.
#[derive(Debug, Deserialize)]
struct StateBody {
    /// The state to go to.
    to: CourseState,
}

/// A move of a course.
#[derive(Debug, Deserialize)]
struct MoveBody {
    /// The category it goes to.
    category: CategoryId,
    /// Where among its courses.
    #[serde(default)]
    place: Place<CourseId>,
}

/// The courses of a category the caller sees.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
) -> ApiResult<Response> {
    let courses = shared.courses()?.courses(&caller.actor, id(&category)?).await?;
    Ok(Json(courses).into_response())
}

/// Creates a course in a category.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
    Body(body): Body<NewCourse>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let category = courses.category(id(&category)?).await?;
    let actor = shared.acting(&caller, COURSE_CREATE, category.scope).await?;
    let course = courses
        .create_course(actor, category.id, body.fields, body.place)
        .await?;
    let version = course.version;
    Ok(with_etag((StatusCode::CREATED, Json(course)), version))
}

/// A course the caller sees, with its owners.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let course = courses.course(id(&course)?).await?;
    if !courses.sees(&caller.actor, course.scope).await? {
        return Err(hidden("courses.course_not_found").into());
    }
    let owners = courses.owners(course.id).await?;
    let version = course.version;
    let mut body =
        serde_json::to_value(&course).map_err(|_| kampus_common::Error::internal("courses.json"))?;
    body["owners"] = json!(owners);
    Ok(with_etag(Json(body), version))
}

/// Replaces the fields of a course.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<CourseFields>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.course(id(&course)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, COURSE_UPDATE, stored.scope).await?;
    let course = courses.update_course(actor, stored.id, fields, expected).await?;
    let version = course.version;
    Ok(with_etag(Json(course), version))
}

/// Takes a course to another state.
async fn transition(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    headers: HeaderMap,
    Body(body): Body<StateBody>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.course(id(&course)?).await?;
    let expected = Some(if_match(&headers)?);
    let permission = match (stored.state, body.to) {
        (CourseState::Published, CourseState::Archived) | (CourseState::Archived, CourseState::Published) => {
            COURSE_ARCHIVE
        }
        (_, CourseState::DeletionScheduled) | (CourseState::DeletionScheduled, _) => COURSE_DELETE,
        _ => COURSE_UPDATE,
    };
    let actor = shared.acting(&caller, permission, stored.scope).await?;
    let course = courses
        .transition_course(actor, stored.id, body.to, expected)
        .await?;
    let version = course.version;
    Ok(with_etag(Json(course), version))
}

/// Moves a course to another category or place.
async fn move_to(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    headers: HeaderMap,
    Body(body): Body<MoveBody>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.course(id(&course)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, COURSE_UPDATE, stored.scope).await?;
    let course = courses
        .move_course(actor, stored.id, body.category, body.place, expected)
        .await?;
    let version = course.version;
    Ok(with_etag(Json(course), version))
}
