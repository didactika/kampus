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

//! The routes of sections: the tree of a course as the caller sees it, and its changes.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use kampus_identity::http::Caller;
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use serde::Deserialize;

use super::{Shared, hidden};
use crate::permissions::SECTION_MANAGE;
use crate::{Place, Section, SectionFields, SectionId};

/// The operations of sections.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get(
        "/courses/{id}/sections",
        "The sections of a course the caller sees, depth first",
    )
    .returns::<Vec<Section>>(),
    Operation::post(
        "/courses/{id}/sections",
        "Add a section under a parent and at a place among its siblings",
    )
    .body::<NewSection>()
    .creates::<Section>(),
    Operation::get(
        "/sections/{id}",
        "A section the caller sees, with its version as ETag",
    )
    .returns::<Section>(),
    Operation::put(
        "/sections/{id}",
        "Replace the title, summary and visibility of a section",
    )
    .body::<SectionFields>()
    .returns::<Section>(),
    Operation::delete(
        "/sections/{id}",
        "Delete a section with the sections in it, recoverably",
    )
    .no_content(),
    Operation::post(
        "/sections/{id}/move",
        "Move a section with its subtree under another parent and/or place",
    )
    .body::<MoveBody>()
    .returns::<Section>(),
    Operation::post(
        "/sections/{id}/restore",
        "Restore a deleted section with the sections deleted with it",
    )
    .returns::<Section>(),
];

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/courses/{id}/sections", get(list).post(add))
        .route("/sections/{id}", get(read).put(update).delete(remove))
        .route("/sections/{id}/move", post(move_to))
        .route("/sections/{id}/restore", post(restore))
}

/// A move: the new parent and the place among the siblings.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MoveBody {
    /// The new parent; the top when absent.
    #[serde(default)]
    parent: Option<SectionId>,
    /// Where among the siblings.
    #[serde(default)]
    place: Place<SectionId>,
}

/// A new section.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewSection {
    /// The section it goes in; the top of the course when absent.
    #[serde(default)]
    parent: Option<SectionId>,
    /// Where among its siblings.
    #[serde(default)]
    place: Place<SectionId>,
    /// Its fields.
    #[serde(flatten)]
    fields: SectionFields,
}

/// The sections of a course the caller sees, depth first.
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
    Ok(Json(courses.sections(&caller.actor, course.id).await?).into_response())
}

/// Adds a section.
async fn add(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<NewSection>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let course = courses.course(id(&course)?).await?;
    let scope = match body.parent {
        Some(parent) => courses.section(parent).await?.scope,
        None => course.scope,
    };
    let actor = shared.acting(&caller, SECTION_MANAGE, scope).await?;
    let section = courses
        .add_section(actor, course.id, body.parent, body.fields, body.place)
        .await?;
    let version = section.version;
    Ok(with_etag((StatusCode::CREATED, Json(section)), version))
}

/// A section the caller sees.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let section = courses.section(id(&section)?).await?;
    if !courses.sees(&caller.actor, section.scope).await? {
        return Err(hidden("courses.section_not_found").into());
    }
    let version = section.version;
    Ok(with_etag(Json(section), version))
}

/// Replaces the fields of a section.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<SectionFields>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.section(id(&section)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, SECTION_MANAGE, stored.scope).await?;
    let section = courses.update_section(actor, stored.id, fields, expected).await?;
    let version = section.version;
    Ok(with_etag(Json(section), version))
}

/// Deletes a section with the sections nested in it, recoverably.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let stored = courses.section(id(&section)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, SECTION_MANAGE, stored.scope).await?;
    courses.delete_section(actor, stored.id, expected).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Moves a section.
async fn move_to(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
    headers: HeaderMap,
    Body(body): Body<MoveBody>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.section(id(&section)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, SECTION_MANAGE, stored.scope).await?;
    let section = courses
        .move_section(actor, stored.id, body.parent, body.place, expected)
        .await?;
    let version = section.version;
    Ok(with_etag(Json(section), version))
}

/// Restores a deleted section.
async fn restore(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
) -> ApiResult<Response> {
    let section = shared
        .courses()?
        .restore_section(&caller.actor, id(&section)?)
        .await?;
    let version = section.version;
    Ok(with_etag(Json(section), version))
}
