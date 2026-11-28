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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of settings: the settings of a scope as its administration reads and changes them.

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::{Acting, ScopeId};
use kampus_common::{Error, Result};
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;
use serde_json::Value;

use crate::{ScopeSetting, SettingChange, Settings};

/// The operations of the routes.
pub const OPERATIONS: &[&[Operation]] = &[&[
    Operation::get(
        "/scopes/{scope}",
        "The settings a scope may set and the caller may change, with their values, origins and locks (query `q`)",
    )
    .returns::<Vec<ScopeSetting>>(),
    Operation::get("/scopes/{scope}/{key}", "A setting of a scope").returns::<ScopeSetting>(),
    Operation::put(
        "/scopes/{scope}/{key}",
        "Set a setting in a scope, and lock it for the scopes below; `If-Match` carries its version, 0 when the scope sets none",
    )
    .body::<Change>()
    .returns::<ScopeSetting>(),
    Operation::delete(
        "/scopes/{scope}/{key}",
        "Remove the value a scope sets, so it comes from above again",
    )
    .returns::<ScopeSetting>(),
    Operation::get(
        "/scopes/{scope}/{key}/history",
        "The changes of a setting in a scope, newest first",
    )
    .returns::<Vec<SettingChange>>(),
]];

/// The routes of settings.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        settings: context.service::<Settings>().ok(),
    };
    Router::new()
        .route("/scopes/{scope}", get(list))
        .route("/scopes/{scope}/{key}", get(one).put(set).delete(unset))
        .route("/scopes/{scope}/{key}/history", get(history))
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    settings: Option<Settings>,
}

impl Shared {
    /// The service.
    fn settings(&self) -> Result<&Settings> {
        self.settings
            .as_ref()
            .ok_or_else(|| Error::internal("settings.service_missing"))
    }
}

/// Words to find settings by.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Search {
    /// Words found in the key, the module or the summary.
    #[serde(default)]
    q: Option<String>,
}

/// The value a scope gives a setting.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Change {
    /// The value, of the kind of the setting and within its bounds.
    value: Value,
    /// Whether the scopes below keep it.
    #[serde(default)]
    locked: bool,
}

/// A setting answered with its version as `ETag`.
fn answer(setting: ScopeSetting) -> Response {
    let version = setting.version;
    with_etag(Json(setting), version)
}

/// The settings of a scope.
async fn list(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path(scope): Path<String>,
    Query(search): Query<Search>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let found = shared
        .settings()?
        .of_scope(
            &acting,
            scope,
            search.q.as_deref().filter(|q| !q.trim().is_empty()),
        )
        .await?;
    Ok(Json(found).into_response())
}

/// A setting of a scope.
async fn one(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path((scope, key)): Path<(String, String)>,
) -> ApiResult<Response> {
    let setting = shared.settings()?.in_scope(&acting, &key, id(&scope)?).await?;
    Ok(answer(setting))
}

/// Sets a setting in a scope.
async fn set(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path((scope, key)): Path<(String, String)>,
    headers: HeaderMap,
    Body(change): Body<Change>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let version = if_match(&headers)?;
    let settings = shared.settings()?;
    settings.in_scope(&acting, &key, scope).await?;
    settings
        .set(
            &acting.actor,
            &key,
            scope,
            change.value,
            change.locked,
            Some(version),
        )
        .await?;
    Ok(answer(settings.in_scope(&acting, &key, scope).await?))
}

/// Removes the value a scope sets.
async fn unset(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path((scope, key)): Path<(String, String)>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let settings = shared.settings()?;
    settings.in_scope(&acting, &key, scope).await?;
    settings.unset(&acting.actor, &key, scope).await?;
    Ok(answer(settings.in_scope(&acting, &key, scope).await?))
}

/// The changes of a setting in a scope, to whoever may change it there.
async fn history(
    State(shared): State<Shared>,
    Extension(acting): Extension<Acting>,
    Path((scope, key)): Path<(String, String)>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let settings = shared.settings()?;
    settings.in_scope(&acting, &key, scope).await?;
    Ok(Json(settings.history(&key, scope).await?).into_response())
}
