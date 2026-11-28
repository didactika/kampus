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

//! The roles of a campus: what each gives every permission, and whether it asks for a second factor.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::Shared;
use crate::permissions::ROLE_MANAGE;
use crate::{Acting, PermissionInfo, RoleId, RoleView, ScopeId, ScopeKind, Value};

/// The operations of roles.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get(
        "/scopes/{scope}/permissions",
        "Every declared permission, with what it can expose and the lowest scope where it means something",
    )
    .returns::<Vec<PermissionInfo>>(),
    Operation::get(
        "/scopes/{scope}/roles",
        "The roles of the campus of a scope, with their values",
    )
    .returns::<Vec<RoleView>>(),
    Operation::post(
        "/scopes/{scope}/roles",
        "Create a role in a campus from a template",
    )
    .body::<NewRole>()
    .creates::<RoleView>(),
    Operation::get("/roles/{role}", "A role with its values").returns::<RoleView>(),
    Operation::put(
        "/roles/{role}/permissions/{permission}",
        "Set the value a role gives a permission; null takes it away",
    )
    .body::<RoleValue>()
    .returns::<RoleView>(),
    Operation::put(
        "/roles/{role}/second-factor",
        "Whether whoever holds a role must sign in with a second factor",
    )
    .body::<SecondFactor>()
    .returns::<RoleView>(),
];

/// The routes of roles.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/scopes/{scope}/permissions", get(permissions))
        .route("/scopes/{scope}/roles", get(roles).post(create))
        .route("/roles/{role}", get(role))
        .route("/roles/{role}/permissions/{permission}", put(set_value))
        .route("/roles/{role}/second-factor", put(second_factor))
}

/// A role to create.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewRole {
    /// Its short name, unique in its campus.
    short_name: String,
    /// The code of the template it starts from.
    template: String,
    /// The kinds of scope where it may be granted.
    kinds: Vec<ScopeKind>,
}

/// The value a role gives a permission.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RoleValue {
    /// The value; absent or null takes it away, so the role does not allow it.
    #[serde(default)]
    value: Option<Value>,
}

/// Whether a role asks for a second factor.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SecondFactor {
    /// Whether it does.
    required: bool,
}

/// The declared permissions.
async fn permissions(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    let found = shared.access()?.permissions(&acting, id(&scope)?).await?;
    Ok(Json(found).into_response())
}

/// The roles of the campus of a scope.
async fn roles(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    let found = shared.access()?.roles(&acting, id(&scope)?).await?;
    Ok(Json(found).into_response())
}

/// Creates a role in a campus.
async fn create(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
    Body(body): Body<NewRole>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&scope)?;
    let access = shared.access()?;
    let actor = access.acting(&acting, ROLE_MANAGE, campus).await?;
    let role = access
        .create_role(actor, campus, body.short_name.trim(), &body.template, &body.kinds)
        .await?;
    Ok((StatusCode::CREATED, Json(access.role(&acting, role).await?)).into_response())
}

/// A role.
async fn role(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(role): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.access()?.role(&acting, id(&role)?).await?).into_response())
}

/// Sets the value a role gives a permission.
async fn set_value(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path((role, permission)): Path<(String, String)>,
    Body(body): Body<RoleValue>,
) -> ApiResult<Response> {
    let role: RoleId = id(&role)?;
    let access = shared.access()?;
    let actor = access
        .acting(&acting, ROLE_MANAGE, access.campus_of_role(role).await?)
        .await?;
    access
        .set_role_value(actor, role, &permission, body.value)
        .await?;
    Ok(Json(access.role(&acting, role).await?).into_response())
}

/// Sets whether a role asks for a second factor.
async fn second_factor(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(role): Path<String>,
    Body(body): Body<SecondFactor>,
) -> ApiResult<Response> {
    let role: RoleId = id(&role)?;
    let access = shared.access()?;
    let actor = access
        .acting(&acting, ROLE_MANAGE, access.campus_of_role(role).await?)
        .await?;
    access.set_role_second_factor(actor, role, body.required).await?;
    Ok(Json(access.role(&acting, role).await?).into_response())
}
