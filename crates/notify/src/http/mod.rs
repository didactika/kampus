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
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_common::{Error, Result};
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;

use crate::{NotificationId, Notify};

mod messaging;

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[NOTIFY, messaging::OPERATIONS];

/// The operations of the inbox, preferences, templates and push.
const NOTIFY: &[Operation] = &[
    Operation::get(
        "/inbox",
        "A page of the notifications of the caller (query `limit`, `after`)",
    )
    .returns::<crate::Inbox>(),
    Operation::post("/inbox/read-all", "Mark every notification of the caller read").returns::<ReadAll>(),
    Operation::post("/inbox/{id}/read", "Mark a notification read").returns::<crate::Notification>(),
    Operation::get("/preferences", "How the caller wants each type of notification")
        .returns::<Vec<crate::Preference>>(),
    Operation::put(
        "/preferences/{type_code}/{channel}",
        "Turn a channel of a type of notification on or off",
    )
    .body::<Choice>()
    .returns::<crate::Preference>(),
    Operation::get(
        "/campuses/{id}/templates",
        "The templates a campus wrote for its notifications",
    )
    .returns::<Vec<crate::service::channel::Template>>(),
    Operation::put(
        "/campuses/{id}/templates",
        "Write the template of a type, channel and locale",
    )
    .body::<crate::service::channel::Template>()
    .returns::<crate::service::channel::Template>(),
    Operation::delete(
        "/campuses/{id}/templates/{type_code}/{channel}/{locale}",
        "Go back to the template a module ships",
    )
    .no_content(),
    Operation::get(
        "/campuses/{id}/defaults",
        "The channels a campus turns on by default",
    )
    .returns::<Vec<crate::service::channel::CampusDefault>>(),
    Operation::put(
        "/campuses/{id}/defaults",
        "Set whether a channel of a type is on by default in a campus",
    )
    .body::<crate::service::channel::CampusDefault>()
    .returns::<crate::service::channel::CampusDefault>(),
    Operation::get("/push-key", "The public key push subscriptions are made with").returns::<PushKey>(),
    Operation::get("/push-subscriptions", "The push subscriptions of the caller")
        .returns::<Vec<crate::service::channel::PushSubscription>>(),
    Operation::post("/push-subscriptions", "Register a push subscription of a browser")
        .body::<crate::service::channel::PushSubscription>()
        .creates::<crate::service::channel::PushSubscription>(),
    Operation::delete("/push-subscriptions/{id}", "Remove a push subscription").no_content(),
];

/// How many notifications were marked read.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ReadAll {
    /// The notifications that changed.
    changed: u64,
}

/// The public key of web push.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct PushKey {
    /// The key, in base64url.
    public_key: String,
}

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
        .route("/campuses/{id}/templates", get(templates).put(set_template))
        .route(
            "/campuses/{id}/templates/{type_code}/{channel}/{locale}",
            axum::routing::delete(remove_template),
        )
        .route("/campuses/{id}/defaults", get(defaults).put(set_default))
        .route("/push-key", get(push_key))
        .route("/push-subscriptions", get(push_subscriptions).post(register_push))
        .route("/push-subscriptions/{id}", axum::routing::delete(remove_push))
        .merge(messaging::routes())
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
pub(crate) struct Shared {
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
#[derive(Debug, Deserialize, schemars::JsonSchema)]
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
#[derive(Debug, Deserialize, schemars::JsonSchema)]
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
    Ok(Json(ReadAll { changed }).into_response())
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

/// The templates of a campus.
async fn templates(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.notify()?.templates(&caller.actor, id(&campus)?).await?).into_response())
}

/// Writes a template of a campus.
async fn set_template(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Body(body): Body<crate::service::channel::Template>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .notify()?
            .set_template(&caller.actor, id(&campus)?, body)
            .await?,
    )
    .into_response())
}

/// Removes a template of a campus.
async fn remove_template(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((campus, type_code, channel, locale)): Path<(String, String, String, String)>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .remove_template(&caller.actor, id(&campus)?, &type_code, &channel, &locale)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The defaults of a campus.
async fn defaults(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .notify()?
            .campus_defaults(&caller.actor, id(&campus)?)
            .await?,
    )
    .into_response())
}

/// Sets a default of a campus.
async fn set_default(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Body(body): Body<crate::service::channel::CampusDefault>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .notify()?
            .set_campus_default(&caller.actor, id(&campus)?, body)
            .await?,
    )
    .into_response())
}

/// The public key browsers subscribe with.
async fn push_key(State(shared): State<Shared>) -> ApiResult<Response> {
    let key = shared.notify()?.push_key()?;
    Ok(Json(PushKey { public_key: key }).into_response())
}

/// The push subscriptions of the caller.
async fn push_subscriptions(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    Ok(Json(shared.notify()?.push_subscriptions(&caller.actor).await?).into_response())
}

/// Registers a browser or phone of the caller for push notifications.
async fn register_push(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<crate::service::channel::PushSubscription>,
) -> ApiResult<Response> {
    let made = shared.notify()?.register_push(&caller.actor, body).await?;
    Ok((StatusCode::CREATED, Json(made)).into_response())
}

/// Removes a push subscription of the caller.
async fn remove_push(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(subscription): Path<String>,
) -> ApiResult<StatusCode> {
    let subscription = uuid::Uuid::parse_str(&subscription)
        .map_err(|_| kampus_common::Error::not_found("notify.push_not_found"))?;
    shared.notify()?.remove_push(&caller.actor, subscription).await?;
    Ok(StatusCode::NO_CONTENT)
}
