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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of notify: the inbox of the caller and their preferences.

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_common::{Error, Result};
use kampus_identity::http::Caller;
use kampus_module::ServiceContext;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;
use serde_json::json;

use crate::{NotificationId, Notify};

/// The routes of notify.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        notify: context.service::<Notify>().ok(),
    };
    Router::new()
        .route("/inbox", get(inbox))
        .route("/inbox/read-all", post(read_all))
        .route("/inbox/{id}/read", post(read))
        .route("/preferences", get(preferences))
        .route("/preferences/{type_code}/{channel}", put(set_preference))
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    notify: Option<Notify>,
}

impl Shared {
    /// The service.
    fn notify(&self) -> Result<&Notify> {
        self.notify
            .as_ref()
            .ok_or_else(|| Error::internal("notify.service_missing"))
    }
}

/// A page of the inbox.
#[derive(Debug, Deserialize)]
struct Page {
    /// Only what is not read.
    #[serde(default)]
    unread: bool,
    /// The most notifications.
    #[serde(default = "twenty")]
    limit: i64,
    /// The last notification of the page before.
    #[serde(default)]
    after: Option<NotificationId>,
}

/// The default size of a page.
const fn twenty() -> i64 {
    20
}

/// A choice about a type.
#[derive(Debug, Deserialize)]
struct Choice {
    /// Whether the person receives it.
    enabled: bool,
}

/// The inbox of the caller.
async fn inbox(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Query(page): Query<Page>,
) -> ApiResult<Response> {
    let found = shared
        .notify()?
        .inbox(&caller.actor, page.unread, page.limit, page.after)
        .await?;
    Ok(Json(found).into_response())
}

/// Marks a notification read.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(notification): Path<String>,
) -> ApiResult<Response> {
    let found = shared
        .notify()?
        .mark_read(&caller.actor, id(&notification)?)
        .await?;
    Ok(Json(found).into_response())
}

/// Marks every notification read.
async fn read_all(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    let changed = shared.notify()?.mark_all_read(&caller.actor).await?;
    Ok(Json(json!({ "changed": changed })).into_response())
}

/// What the caller receives.
async fn preferences(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    Ok(Json(shared.notify()?.preferences(&caller.actor).await?).into_response())
}

/// Turns a type on or off through a channel.
async fn set_preference(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((type_code, channel)): Path<(String, String)>,
    Body(body): Body<Choice>,
) -> ApiResult<Response> {
    let found = shared
        .notify()?
        .set_preference(&caller.actor, &type_code, &channel, body.enabled)
        .await?;
    Ok(Json(found).into_response())
}
