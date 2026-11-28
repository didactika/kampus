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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a scope sets of access: the roles granted in it, the overrides of permissions it makes, and why a
//! principal may or may not do something there.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, put};
use axum::{Extension, Json, Router};
use kampus_common::Timestamp;
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, Identified, id};
use serde::Deserialize;

use super::Shared;
use crate::permissions::{GRANT_CREATE, GRANT_DELETE, OVERRIDE_UPDATE};
use crate::{Acting, Decision, GrantId, GrantView, OverrideView, PrincipalId, RoleId, ScopeId, Value};

/// The operations of grants, overrides and explanations.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get(
        "/scopes/{scope}/grants",
        "The roles granted in a scope itself, newest first",
    )
    .returns::<Vec<GrantView>>(),
    Operation::post("/scopes/{scope}/grants", "Grant a role to a principal in a scope")
        .body::<NewGrant>()
        .creates::<Identified<GrantId>>(),
    Operation::delete("/grants/{grant}", "Take a grant back").no_content(),
    Operation::get(
        "/principals/{principal}/grants",
        "The roles a principal holds in every scope, newest first",
    )
    .returns::<Vec<GrantView>>(),
    Operation::get(
        "/scopes/{scope}/overrides",
        "The values a scope gives permissions for roles, in place of theirs",
    )
    .returns::<Vec<OverrideView>>(),
    Operation::put(
        "/scopes/{scope}/overrides/{role}/{permission}",
        "Override the value of a permission for a role in a scope and below; null takes the override away",
    )
    .body::<OverrideValue>()
    .no_content(),
    Operation::get(
        "/scopes/{scope}/path",
        "A scope and the scopes above it, nearest first, with their kinds, for whoever is signed in",
    )
    .returns::<Vec<crate::PathScope>>(),
    Operation::get(
        "/scopes/{scope}/explain",
        "Why a principal may or may not use a permission in a scope (query `principal` and `permission`)",
    )
    .returns::<Decision>(),
];

/// The routes of grants, overrides and explanations.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/scopes/{scope}/grants", get(grants).post(grant))
        .route("/grants/{grant}", delete(revoke))
        .route("/principals/{principal}/grants", get(grants_of))
        .route("/scopes/{scope}/overrides", get(overrides))
        .route("/scopes/{scope}/overrides/{role}/{permission}", put(set_override))
        .route("/scopes/{scope}/explain", get(explain))
        .route("/scopes/{scope}/path", get(path))
}

/// A role to grant.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewGrant {
    /// The role.
    role: RoleId,
    /// Who receives it.
    principal: PrincipalId,
    /// Until when it counts; for good when absent.
    #[serde(default)]
    until: Option<Timestamp>,
}

/// The value a scope gives a permission for a role.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OverrideValue {
    /// The value; absent or null takes the override away.
    #[serde(default)]
    value: Option<Value>,
}

/// The check to explain.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Question {
    /// The principal.
    principal: PrincipalId,
    /// The permission.
    permission: String,
}

/// The grants of a scope.
async fn grants(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.access()?.grants_in(&acting, id(&scope)?).await?).into_response())
}

/// Grants a role in a scope.
async fn grant(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
    Body(body): Body<NewGrant>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let access = shared.access()?;
    let actor = access.acting(&acting, GRANT_CREATE, scope).await?;
    let grant = access
        .grant(actor, body.role, scope, body.principal, body.until)
        .await?;
    Ok((StatusCode::CREATED, Json(Identified { id: grant })).into_response())
}

/// Takes a grant back.
async fn revoke(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(grant): Path<String>,
) -> ApiResult<StatusCode> {
    let grant: GrantId = id(&grant)?;
    let access = shared.access()?;
    let actor = access
        .acting(&acting, GRANT_DELETE, access.grant_scope(grant).await?)
        .await?;
    access.revoke(actor, grant).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The overrides of a scope.
async fn overrides(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.access()?.overrides_in(&acting, id(&scope)?).await?).into_response())
}

/// Overrides the value of a permission for a role in a scope.
async fn set_override(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path((scope, role, permission)): Path<(String, String, String)>,
    Body(body): Body<OverrideValue>,
) -> ApiResult<StatusCode> {
    let scope: ScopeId = id(&scope)?;
    let access = shared.access()?;
    let actor = access.acting(&acting, OVERRIDE_UPDATE, scope).await?;
    access
        .set_override(actor, scope, id(&role)?, &permission, body.value)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Explains a check.
async fn explain(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
    Query(question): Query<Question>,
) -> ApiResult<Response> {
    let decision = shared
        .access()?
        .explain(&acting, question.principal, &question.permission, id(&scope)?)
        .await?;
    Ok(Json(decision).into_response())
}

/// A scope and the scopes above it.
async fn path(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    if matches!(acting.actor, crate::Actor::Visitor(_)) {
        return Err(kampus_common::Error::unauthenticated("access.sign_in_required").into());
    }
    Ok(Json(shared.access()?.path_of(id(&scope)?).await?).into_response())
}

/// The grants of a principal.
async fn grants_of(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(principal): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.access()?.grants_of(&acting, id(&principal)?).await?).into_response())
}
