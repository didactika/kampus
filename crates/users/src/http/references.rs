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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The routes of the external references of system groups: the identifiers systems give a group, a group
//! found by one, and a group created by one.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_db::references::ExternalReference;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;
use serde_json::json;

use super::{GroupBody, Shared};
use crate::GroupKind;
use crate::permissions::GROUP_MANAGE;

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/system-groups/{id}/references", get(list))
        .route("/system-groups/{id}/references/{system}", put(set).delete(remove))
        .route(
            "/campuses/{campus}/system-groups/by-reference/{system}/{external_id}",
            get(find),
        )
        .route(
            "/scopes/{scope}/system-groups/by-reference/{system}/{external_id}",
            put(ensure),
        )
}

/// The identifier a system gives a group.
#[derive(Debug, Deserialize)]
struct ExternalId {
    /// The identifier.
    external_id: String,
}

/// The identifiers systems give a group.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
) -> ApiResult<Response> {
    let references = shared
        .users()?
        .group_external_references(&caller.actor, id(&group)?)
        .await?;
    Ok(Json(references).into_response())
}

/// Gives a group the identifier a system uses for it.
async fn set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((group, system)): Path<(String, String)>,
    Body(body): Body<ExternalId>,
) -> ApiResult<Response> {
    let reference = ExternalReference::new(&system, &body.external_id)?;
    let references = shared
        .users()?
        .set_group_reference(&caller.actor, id(&group)?, &reference)
        .await?;
    Ok(Json(references).into_response())
}

/// Removes the identifier a system gave a group.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((group, system)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    shared
        .users()?
        .remove_group_reference(&caller.actor, id(&group)?, &system)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The group a system knows by an identifier in a campus.
async fn find(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((campus, system, external_id)): Path<(String, String, String)>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let users = shared.users()?;
    let group = users
        .group_by_reference(campus, &reference)
        .await?
        .ok_or_else(|| kampus_common::Error::not_found("references.not_found"))?;
    users.group_external_references(&caller.actor, group).await?;
    Ok(Json(users.group(group).await?).into_response())
}

/// The group a system knows by an identifier, created in the scope when there is none.
async fn ensure(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, system, external_id)): Path<(String, String, String)>,
    Body(body): Body<GroupBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let actor = caller.acting(shared.identity()?, GROUP_MANAGE, scope).await?;
    let kind = body.rule.map_or(GroupKind::Manual, GroupKind::Rule);
    let (group, created) = shared
        .users()?
        .ensure_group_by_reference(actor, scope, &reference, (&body.code, &body.name), kind)
        .await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(json!({ "id": group }))).into_response())
}
