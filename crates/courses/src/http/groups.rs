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

//! The routes of groups and group sets.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_access::PrincipalId;
use kampus_identity::http::Caller;
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use serde::Deserialize;

use super::Shared;
use crate::permissions::GROUPS_MANAGE;
use crate::{CourseId, Group, GroupFields, GroupId, GroupSet, GroupSetId, Member};

/// The operations of groups and group sets.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get(
        "/courses/{id}/groups",
        "The groups of a course the caller may see",
    )
    .returns::<Vec<Group>>(),
    Operation::post(
        "/courses/{id}/groups",
        "Create a group, optionally limited to a section",
    )
    .body::<GroupFields>()
    .creates::<Group>(),
    Operation::get(
        "/groups/{id}",
        "A group the caller sees, with its version as ETag",
    )
    .returns::<Group>(),
    Operation::put("/groups/{id}", "Replace a group")
        .body::<GroupFields>()
        .returns::<Group>(),
    Operation::delete("/groups/{id}", "Delete a group").no_content(),
    Operation::get("/groups/{id}/members", "The members of a group with their origin")
        .returns::<Vec<Member>>(),
    Operation::post("/groups/{id}/members", "Add a participant to a group by hand")
        .body::<MemberBody>()
        .no_content(),
    Operation::delete("/groups/{id}/members/{account}", "Remove a manual membership").no_content(),
    Operation::get(
        "/courses/{id}/group-sets",
        "The group sets of a course with their groups",
    )
    .returns::<Vec<GroupSet>>(),
    Operation::post("/courses/{id}/group-sets", "Create a group set")
        .body::<SetBody>()
        .creates::<GroupSet>(),
    Operation::put(
        "/group-sets/{id}/groups/{group}",
        "Put a group of the same course in a set",
    )
    .no_content(),
    Operation::delete("/group-sets/{id}/groups/{group}", "Take a group out of a set").no_content(),
];

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/courses/{id}/groups", get(list).post(create))
        .route("/groups/{id}", get(read).put(update).delete(remove))
        .route("/groups/{id}/members", get(members).post(add_member))
        .route(
            "/groups/{id}/members/{account}",
            axum::routing::delete(remove_member),
        )
        .route("/courses/{id}/group-sets", get(sets).post(create_set))
        .route(
            "/group-sets/{id}/groups/{group}",
            put(add_to_set).delete(remove_from_set),
        )
}

/// A member to add.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MemberBody {
    /// The account.
    account: PrincipalId,
}

/// A new group set.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SetBody {
    /// Its name.
    name: String,
    /// Its code.
    #[serde(default)]
    code: Option<String>,
}

/// The actor managing the groups of a course.
async fn managing<'c>(
    shared: &Shared,
    caller: &'c Caller,
    course: CourseId,
) -> ApiResult<&'c kampus_access::Actor> {
    let scope = shared.courses()?.course(course).await?.scope;
    Ok(shared.acting(caller, GROUPS_MANAGE, scope).await?)
}

/// The groups of a course the caller sees.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let groups = shared.courses()?.groups(&caller.actor, id(&course)?).await?;
    Ok(Json(groups).into_response())
}

/// Creates a group.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(fields): Body<GroupFields>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = managing(&shared, &caller, course).await?;
    let group = shared.courses()?.create_group(actor, course, fields).await?;
    let version = group.version;
    Ok(with_etag((StatusCode::CREATED, Json(group)), version))
}

/// A group the caller sees.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let group = courses.group(id(&group)?).await?;
    let seen = courses.groups(&caller.actor, group.course).await?;
    if !seen.iter().any(|g| g.id == group.id) {
        return Err(kampus_common::Error::not_found("courses.group_not_found").into());
    }
    let version = group.version;
    Ok(with_etag(Json(group), version))
}

/// Replaces the fields of a group.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<GroupFields>,
) -> ApiResult<Response> {
    let courses = shared.courses()?;
    let stored = courses.group(id(&group)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = managing(&shared, &caller, stored.course).await?;
    let group = courses.update_group(actor, stored.id, fields, expected).await?;
    let version = group.version;
    Ok(with_etag(Json(group), version))
}

/// Deletes a group.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let stored = courses.group(id(&group)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = managing(&shared, &caller, stored.course).await?;
    courses.delete_group(actor, stored.id, expected).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The members of a group the caller sees.
async fn members(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
) -> ApiResult<Response> {
    let members = shared.courses()?.members(&caller.actor, id(&group)?).await?;
    Ok(Json(members).into_response())
}

/// Adds a member by hand.
async fn add_member(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
    Body(body): Body<MemberBody>,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let group = courses.group(id(&group)?).await?;
    let actor = managing(&shared, &caller, group.course).await?;
    courses.add_member(actor, group.id, body.account).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes a membership added by hand.
async fn remove_member(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((group, account)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let group = courses.group(id(&group)?).await?;
    let actor = managing(&shared, &caller, group.course).await?;
    courses.remove_member(actor, group.id, id(&account)?).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The group sets of a course.
async fn sets(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = managing(&shared, &caller, course).await?;
    Ok(Json(shared.courses()?.group_sets(actor, course).await?).into_response())
}

/// Creates a group set.
async fn create_set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<SetBody>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = managing(&shared, &caller, course).await?;
    let set = shared
        .courses()?
        .create_group_set(actor, course, &body.name, body.code)
        .await?;
    Ok((StatusCode::CREATED, Json(set)).into_response())
}

/// Puts a group in a set.
async fn add_to_set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((set, group)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let group = courses.group(id(&group)?).await?;
    let actor = managing(&shared, &caller, group.course).await?;
    let set: GroupSetId = id(&set)?;
    courses.add_to_set(actor, set, group.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Takes a group out of a set.
async fn remove_from_set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((set, group)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let courses = shared.courses()?;
    let group: GroupId = id(&group)?;
    let stored = courses.group(group).await?;
    let actor = managing(&shared, &caller, stored.course).await?;
    courses.remove_from_set(actor, id(&set)?, group).await?;
    Ok(StatusCode::NO_CONTENT)
}
