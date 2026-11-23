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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The routes of the layouts, menus and theme of a scope: each read as the caller sees it, replaced whole
//! or removed so it comes from above again.

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_identity::http::Caller;
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;
use serde_json::Value;

use super::Shared;
use crate::permissions::{LAYOUT_MANAGE, MENU_MANAGE, THEME_MANAGE};
use crate::{MenuItem, Placement};

/// The operations of layouts, menus and themes.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get(
        "/scopes/{scope}/layouts/{page_kind}",
        "The layout of a kind of page in a scope",
    )
    .returns::<crate::Layout>(),
    Operation::put(
        "/scopes/{scope}/layouts/{page_kind}",
        "Place widgets on the grid of a kind of page",
    )
    .body::<LayoutBody>()
    .returns::<crate::Layout>(),
    Operation::delete(
        "/scopes/{scope}/layouts/{page_kind}",
        "Go back to the layout the scope inherits",
    )
    .returns::<crate::Layout>(),
    Operation::get("/scopes/{scope}/menus/{kind}", "A navigation menu of a scope").returns::<crate::Menu>(),
    Operation::put(
        "/scopes/{scope}/menus/{kind}",
        "Replace the items of a navigation menu",
    )
    .body::<MenuBody>()
    .returns::<crate::Menu>(),
    Operation::delete(
        "/scopes/{scope}/menus/{kind}",
        "Go back to the menu the scope inherits",
    )
    .returns::<crate::Menu>(),
    Operation::get("/scopes/{scope}/theme", "The theme of a scope").returns::<crate::Theme>(),
    Operation::put("/scopes/{scope}/theme", "Set the tokens of the theme of a scope")
        .body::<ThemeBody>()
        .returns::<crate::Theme>(),
    Operation::delete("/scopes/{scope}/theme", "Go back to the theme the scope inherits")
        .returns::<crate::Theme>(),
    Operation::get(
        "/scopes/{scope}/theme.css",
        "The theme of a scope as CSS custom properties",
    )
    .returns_media("text/css"),
];

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route(
            "/scopes/{scope}/layouts/{page_kind}",
            get(layout).put(set_layout).delete(reset_layout),
        )
        .route(
            "/scopes/{scope}/menus/{kind}",
            get(menu).put(set_menu).delete(reset_menu),
        )
        .route(
            "/scopes/{scope}/theme",
            get(theme).put(set_theme).delete(reset_theme),
        )
        .route("/scopes/{scope}/theme.css", get(theme_css))
}

/// A layout to set.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LayoutBody {
    /// Its widgets, each a rectangle of cells of the grid.
    placements: Vec<Placement>,
    /// Whether the scopes below keep it.
    #[serde(default)]
    locked: bool,
}

/// A menu to set.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MenuBody {
    /// Its items, in order.
    items: Vec<MenuItem>,
    /// Whether the scopes below keep it.
    #[serde(default)]
    locked: bool,
}

/// A theme to set.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ThemeBody {
    /// The tokens this scope sets.
    tokens: BTreeMap<String, Value>,
    /// Whether the scopes below keep them.
    #[serde(default)]
    locked: bool,
}

/// The layout of a page kind in a scope, with the widgets the caller sees.
async fn layout(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, page_kind)): Path<(String, String)>,
) -> ApiResult<Response> {
    let layout = shared
        .customise()?
        .layout(&caller.actor, &page_kind, id(&scope)?)
        .await?;
    Ok(Json(layout).into_response())
}

/// Replaces the layout of a page kind in a scope.
async fn set_layout(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, page_kind)): Path<(String, String)>,
    Body(body): Body<LayoutBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, LAYOUT_MANAGE, scope).await?;
    let layout = shared
        .customise()?
        .set_layout(actor, &page_kind, scope, body.placements, body.locked)
        .await?;
    Ok(Json(layout).into_response())
}

/// Removes the layout a scope set for a page kind.
async fn reset_layout(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, page_kind)): Path<(String, String)>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, LAYOUT_MANAGE, scope).await?;
    let layout = shared.customise()?.reset_layout(actor, &page_kind, scope).await?;
    Ok(Json(layout).into_response())
}

/// The menu of a kind in a scope, with the items the caller sees.
async fn menu(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, kind)): Path<(String, String)>,
) -> ApiResult<Response> {
    let menu = shared
        .customise()?
        .menu(&caller.actor, &kind, id(&scope)?)
        .await?;
    Ok(Json(menu).into_response())
}

/// Replaces the menu of a kind in a scope.
async fn set_menu(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, kind)): Path<(String, String)>,
    Body(body): Body<MenuBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, MENU_MANAGE, scope).await?;
    let menu = shared
        .customise()?
        .set_menu(actor, &kind, scope, body.items, body.locked)
        .await?;
    Ok(Json(menu).into_response())
}

/// Removes the menu a scope set for a kind.
async fn reset_menu(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, kind)): Path<(String, String)>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, MENU_MANAGE, scope).await?;
    let menu = shared.customise()?.reset_menu(actor, &kind, scope).await?;
    Ok(Json(menu).into_response())
}

/// The theme of a scope, every token with where it came from.
async fn theme(State(shared): State<Shared>, Path(scope): Path<String>) -> ApiResult<Response> {
    Ok(Json(shared.customise()?.theme(id(&scope)?).await?).into_response())
}

/// The theme of a scope as CSS custom properties.
async fn theme_css(State(shared): State<Shared>, Path(scope): Path<String>) -> ApiResult<Response> {
    let theme = shared.customise()?.theme(id(&scope)?).await?;
    Ok(([(header::CONTENT_TYPE, "text/css; charset=utf-8")], theme.css()).into_response())
}

/// Sets the tokens of the theme of a scope.
async fn set_theme(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(body): Body<ThemeBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, THEME_MANAGE, scope).await?;
    let theme = shared
        .customise()?
        .set_theme(actor, scope, &body.tokens, body.locked)
        .await?;
    Ok(Json(theme).into_response())
}

/// Removes the theme a scope set.
async fn reset_theme(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, THEME_MANAGE, scope).await?;
    Ok(Json(shared.customise()?.reset_theme(actor, scope).await?).into_response())
}
