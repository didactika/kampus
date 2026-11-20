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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The routes of quotas: how much the files of a scope or of a person may take, and take.

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::{PrincipalId, ScopeId};
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::Shared;

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/scopes/{scope}/quota", get(scope_quota).put(set_scope_quota))
        .route("/people/{id}/quota", get(person_quota).put(set_person_quota))
}

/// A limit to set.
#[derive(Debug, Deserialize)]
struct Limit {
    /// The most, in bytes; no limit when null.
    limit_bytes: Option<i64>,
}

/// The quota of a scope.
async fn scope_quota(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    Ok(Json(shared.files()?.quota(&caller.actor, scope).await?).into_response())
}

/// Sets the quota of a scope.
async fn set_scope_quota(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(body): Body<Limit>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let quota = shared
        .files()?
        .set_quota(&caller.actor, scope, body.limit_bytes)
        .await?;
    Ok(Json(quota).into_response())
}

/// The quota of a person.
async fn person_quota(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(person): Path<String>,
) -> ApiResult<Response> {
    let person: PrincipalId = id(&person)?;
    Ok(Json(shared.files()?.person_quota(&caller.actor, person).await?).into_response())
}

/// Sets the quota of a person.
async fn set_person_quota(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(person): Path<String>,
    Body(body): Body<Limit>,
) -> ApiResult<Response> {
    let person: PrincipalId = id(&person)?;
    let quota = shared
        .files()?
        .set_person_quota(&caller.actor, person, body.limit_bytes)
        .await?;
    Ok(Json(quota).into_response())
}
