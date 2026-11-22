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

//! The routes of external references: the identifiers systems give each record, a record found by one,
//! and categories and courses created or updated by one.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_db::references::ExternalReference;
use kampus_identity::http::Caller;
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::Shared;
use crate::permissions::{CATEGORY_MANAGE, COURSE_CREATE};
use crate::{Category, CategoryFields, CategoryId, Course, CourseFields, RecordKind};

/// The operations of the identifiers systems give categories, courses, sections, groups and group sets.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get(
        "/{kind}/{id}/references",
        "The identifiers systems give a record (kind: categories, courses, sections, groups, group-sets)",
    )
    .returns::<Vec<ExternalReference>>(),
    Operation::put(
        "/{kind}/{id}/references/{system}",
        "Give a record the identifier a system uses for it",
    )
    .body::<ExternalId>()
    .returns::<Vec<ExternalReference>>(),
    Operation::delete(
        "/{kind}/{id}/references/{system}",
        "Remove the identifier a system gave a record",
    )
    .no_content(),
    Operation::get(
        "/campuses/{campus}/{kind}/by-reference/{system}/{external_id}",
        "The record of a kind a system knows by an identifier in a campus",
    )
    .returns::<Found>(),
    Operation::put(
        "/campuses/{campus}/categories/by-reference/{system}/{external_id}",
        "Update the category a system knows by an identifier; 201 when it was created last under its parent",
    )
    .body::<CategoryByReference>()
    .returns::<Category>(),
    Operation::put(
        "/categories/{category}/courses/by-reference/{system}/{external_id}",
        "Update the course a system knows by an identifier; 201 when it was created last in the category",
    )
    .body::<CourseFields>()
    .returns::<Course>(),
];

/// The record a system knows by an identifier.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct Found {
    /// Its kind.
    kind: RecordKind,
    /// Its id.
    id: uuid::Uuid,
}

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/{kind}/{id}/references", get(list))
        .route("/{kind}/{id}/references/{system}", put(set).delete(remove))
        .route(
            "/campuses/{campus}/{kind}/by-reference/{system}/{external_id}",
            get(find),
        )
        .route(
            "/campuses/{campus}/categories/by-reference/{system}/{external_id}",
            put(upsert_category),
        )
        .route(
            "/categories/{category}/courses/by-reference/{system}/{external_id}",
            put(upsert_course),
        )
}

/// The kind of record a segment of the routes names.
fn kind(segment: &str) -> kampus_common::Result<RecordKind> {
    RecordKind::ALL
        .into_iter()
        .find(|k| k.segment() == segment)
        .ok_or_else(|| kampus_common::Error::not_found("courses.route_not_found"))
}

/// The identifier a system gives a record.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExternalId {
    /// The identifier.
    external_id: String,
}

/// A category to create or update by its identifier.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CategoryByReference {
    /// Its parent when it is created; the top of the campus when absent.
    #[serde(default)]
    parent: Option<CategoryId>,
    /// Its fields.
    #[serde(flatten)]
    fields: CategoryFields,
}

/// The identifiers systems give a record.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((segment, record)): Path<(String, String)>,
) -> ApiResult<Response> {
    let record: uuid::Uuid = id(&record)?;
    let references = shared
        .courses()?
        .external_references(&caller.actor, kind(&segment)?, record)
        .await?;
    Ok(Json(references).into_response())
}

/// Gives a record the identifier a system uses for it.
async fn set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((segment, record, system)): Path<(String, String, String)>,
    Body(body): Body<ExternalId>,
) -> ApiResult<Response> {
    let record: uuid::Uuid = id(&record)?;
    let reference = ExternalReference::new(&system, &body.external_id)?;
    let references = shared
        .courses()?
        .set_external_reference(&caller.actor, kind(&segment)?, record, &reference)
        .await?;
    Ok(Json(references).into_response())
}

/// Removes the identifier a system gave a record.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((segment, record, system)): Path<(String, String, String)>,
) -> ApiResult<StatusCode> {
    let record: uuid::Uuid = id(&record)?;
    shared
        .courses()?
        .remove_external_reference(&caller.actor, kind(&segment)?, record, &system)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The record of a kind a system knows by an identifier in a campus.
async fn find(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((campus, segment, system, external_id)): Path<(String, String, String, String)>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let kind = kind(&segment)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let courses = shared.courses()?;
    let found = courses
        .find_by_reference(kind, campus, &reference)
        .await?
        .ok_or_else(|| kampus_common::Error::not_found("references.not_found"))?;
    courses.external_references(&caller.actor, kind, found).await?;
    Ok(Json(Found { kind, id: found }).into_response())
}

/// Updates the category a system knows by an identifier, or creates it.
async fn upsert_category(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((campus, system, external_id)): Path<(String, String, String)>,
    Body(body): Body<CategoryByReference>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let actor = shared.acting(&caller, CATEGORY_MANAGE, campus).await?;
    let (category, created) = shared
        .courses()?
        .upsert_category_by_reference(actor, campus, &reference, body.parent, body.fields)
        .await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(category)).into_response())
}

/// Updates the course a system knows by an identifier, or creates it in the category.
async fn upsert_course(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((category, system, external_id)): Path<(String, String, String)>,
    Body(fields): Body<CourseFields>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let category = courses.category(id(&category)?).await?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let actor = shared.acting(&caller, COURSE_CREATE, category.scope).await?;
    let (course, created) = courses
        .upsert_course_by_reference(actor, category.id, &reference, fields)
        .await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(course)).into_response())
}
