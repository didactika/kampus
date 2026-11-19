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

//! The routes of categories.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use serde::Deserialize;

use super::{Shared, hidden};
use crate::permissions::CATEGORY_MANAGE;
use crate::{CategoryFields, CategoryId, Place};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/campuses/{campus}/categories", get(list).post(create))
        .route("/categories/{id}", get(read).put(update).delete(remove))
        .route("/categories/{id}/move", post(move_to))
}

/// A move: the new parent and the place among the siblings.
#[derive(Debug, Deserialize)]
struct MoveBody {
    /// The new parent; the top when absent.
    #[serde(default)]
    parent: Option<CategoryId>,
    /// Where among the siblings.
    #[serde(default)]
    place: Place<CategoryId>,
}

/// The parent a list is under.
#[derive(Debug, Deserialize)]
struct Parent {
    /// The parent category; the top of the campus when absent.
    parent: Option<CategoryId>,
}

/// A new category.
#[derive(Debug, Deserialize)]
struct NewCategory {
    /// Its parent; the top of the campus when absent.
    #[serde(default)]
    parent: Option<CategoryId>,
    /// Where among its siblings.
    #[serde(default)]
    place: Place<CategoryId>,
    /// Its fields.
    #[serde(flatten)]
    fields: CategoryFields,
}

/// The categories under a parent the caller sees.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Query(parent): Query<Parent>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let categories = shared
        .courses()?
        .categories(&caller.actor, campus, parent.parent)
        .await?;
    Ok(Json(categories).into_response())
}

/// Creates a category.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Body(body): Body<NewCategory>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let actor = shared.acting(&caller, CATEGORY_MANAGE, campus).await?;
    let category = shared
        .courses()?
        .create_category(actor, campus, body.parent, body.fields, body.place)
        .await?;
    let version = category.version;
    Ok(with_etag((StatusCode::CREATED, Json(category)), version))
}

/// A category the caller sees.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let category = courses.category(id(&category)?).await?;
    if !courses.sees(&caller.actor, category.scope).await? {
        return Err(hidden("courses.category_not_found").into());
    }
    let version = category.version;
    Ok(with_etag(Json(category), version))
}

/// Replaces the fields of a category.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<CategoryFields>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.category(id(&category)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, CATEGORY_MANAGE, stored.scope).await?;
    let category = courses
        .update_category(actor, stored.id, fields, expected)
        .await?;
    let version = category.version;
    Ok(with_etag(Json(category), version))
}

/// Deletes an empty category.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let stored = courses.category(id(&category)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, CATEGORY_MANAGE, stored.scope).await?;
    courses.delete_category(actor, stored.id, expected).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Moves a category.
async fn move_to(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
    headers: HeaderMap,
    Body(body): Body<MoveBody>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.category(id(&category)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, CATEGORY_MANAGE, stored.scope).await?;
    let category = courses
        .move_category(actor, stored.id, body.parent, body.place, expected)
        .await?;
    let version = category.version;
    Ok(with_etag(Json(category), version))
}
