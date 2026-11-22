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
// @package   kampus-calendar
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of calendar: the entries of a scope and of a person, and the calendar of a course.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::{Error, Result, Timestamp};
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;

use crate::permissions::MANAGE;
use crate::{Calendar, EntryFields, EntryId};

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[CALENDAR];

/// The operations of entries, feeds and subscriptions.
const CALENDAR: &[Operation] = &[
    Operation::get(
        "/entries",
        "The entries and dates the caller sees in a window (query `from`, `to`)",
    )
    .returns::<Vec<crate::Item>>(),
    Operation::post("/entries", "Add an entry to a calendar")
        .body::<NewEntry>()
        .creates::<crate::Entry>(),
    Operation::get("/entries/{id}", "An entry, with its version as ETag").returns::<crate::Entry>(),
    Operation::put("/entries/{id}", "Replace an entry")
        .body::<crate::EntryFields>()
        .returns::<crate::Entry>(),
    Operation::delete("/entries/{id}", "Delete an entry").no_content(),
    Operation::put(
        "/entries/{id}/occurrences/{at}",
        "Move or cancel one occurrence of a repeating entry",
    )
    .body::<crate::OccurrenceChange>()
    .no_content(),
    Operation::delete(
        "/entries/{id}/occurrences/{at}",
        "Restore one occurrence of a repeating entry",
    )
    .no_content(),
    Operation::get("/courses/{id}/calendar", "The calendar of a course in a window")
        .returns::<Vec<crate::Item>>(),
    Operation::get("/feeds", "The calendar feeds of the caller").returns::<Vec<crate::Feed>>(),
    Operation::post("/feeds", "Make a calendar feed; its secret address is shown once")
        .body::<NewFeed>()
        .creates::<NewFeedAnswer>(),
    Operation::delete("/feeds/{id}", "Revoke a calendar feed").no_content(),
    Operation::get("/feed/{secret}", "The calendar of a feed, as iCalendar").returns_media("text/calendar"),
    Operation::get(
        "/scopes/{id}/subscriptions",
        "The external calendars a scope subscribes to",
    )
    .returns::<Vec<crate::Subscription>>(),
    Operation::post(
        "/scopes/{id}/subscriptions",
        "Subscribe a scope to an external calendar",
    )
    .body::<NewSubscription>()
    .creates::<crate::Subscription>(),
    Operation::delete("/subscriptions/{id}", "Remove a subscription with its events").no_content(),
    Operation::post("/subscriptions/{id}/refresh", "Read a subscription again now")
        .returns::<crate::service::subscription::Refreshed>(),
];

/// A new feed, with its secret address, shown once.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct NewFeedAnswer {
    /// The feed.
    feed: crate::Feed,
    /// Its secret.
    secret: String,
    /// The address of its calendar.
    url: String,
}

/// The routes of calendar.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        calendar: context.service::<Calendar>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .route("/entries", get(list).post(create))
        .route("/entries/{id}", get(read).put(update).delete(remove))
        .route(
            "/entries/{id}/occurrences/{at}",
            axum::routing::put(change_occurrence).delete(restore_occurrence),
        )
        .route("/courses/{id}/calendar", get(course))
        .route("/feeds", get(feeds).post(create_feed))
        .route("/feeds/{id}", axum::routing::delete(revoke_feed))
        .route("/feed/{secret}", get(feed_calendar))
        .route("/scopes/{id}/subscriptions", get(subscriptions).post(subscribe))
        .route("/subscriptions/{id}", axum::routing::delete(unsubscribe))
        .route("/subscriptions/{id}/refresh", axum::routing::post(refresh))
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    calendar: Option<Calendar>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn calendar(&self) -> Result<&Calendar> {
        self.calendar
            .as_ref()
            .ok_or_else(|| Error::internal("calendar.service_missing"))
    }
}

/// A window of time, and the scope to look in.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Window {
    /// The scope; the personal entries of the caller when left out.
    #[serde(default)]
    scope: Option<ScopeId>,
    /// The start of the window.
    from: Timestamp,
    /// The end of the window.
    to: Timestamp,
}

/// A window of time.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Span {
    /// The start of the window.
    from: Timestamp,
    /// The end of the window.
    to: Timestamp,
}

/// An entry to create.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewEntry {
    /// The scope it belongs to; a personal entry when left out.
    #[serde(default)]
    scope: Option<ScopeId>,
    /// Its fields.
    #[serde(flatten)]
    fields: EntryFields,
}

/// What a calendar shows in a window.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Query(window): Query<Window>,
) -> ApiResult<Response> {
    let items = shared
        .calendar()?
        .entries_for(&caller.actor, window.scope, window.from, window.to)
        .await?;
    Ok(Json(items).into_response())
}

/// Creates an entry.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<NewEntry>,
) -> ApiResult<Response> {
    if let (Some(scope), Some(identity)) = (body.scope, shared.identity.as_ref()) {
        caller.acting(identity, MANAGE, scope).await?;
    }
    let created = shared
        .calendar()?
        .create_entry(&caller.actor, body.scope, body.fields)
        .await?;
    let version = created.version;
    Ok(with_etag((StatusCode::CREATED, Json(created)), version))
}

/// An entry.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(entry): Path<String>,
) -> ApiResult<Response> {
    let found = shared.calendar()?.entry(&caller.actor, id(&entry)?).await?;
    let version = found.version;
    Ok(with_etag(Json(found), version))
}

/// Replaces an entry on the version it was read at.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(entry): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<EntryFields>,
) -> ApiResult<Response> {
    let entry: EntryId = id(&entry)?;
    let expected = Some(if_match(&headers)?);
    let updated = shared
        .calendar()?
        .update_entry(&caller.actor, entry, fields, expected)
        .await?;
    let version = updated.version;
    Ok(with_etag(Json(updated), version))
}

/// Deletes an entry.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(entry): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .calendar()?
        .delete_entry(&caller.actor, id(&entry)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The calendar of a course in a window.
async fn course(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Query(span): Query<Span>,
) -> ApiResult<Response> {
    let items = shared
        .calendar()?
        .course_calendar(&caller.actor, id(&course)?, span.from, span.to)
        .await?;
    Ok(Json(items).into_response())
}

/// The moment of an occurrence in a path.
fn moment(text: &str) -> Result<Timestamp> {
    Timestamp::parse_rfc3339(text).map_err(|_| {
        Error::validation(
            "calendar.occurrence_invalid",
            vec![kampus_common::FieldError::new("at", "format")],
        )
    })
}

/// Cancels or changes one occurrence of a recurring entry.
async fn change_occurrence(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((entry, at)): Path<(String, String)>,
    Body(body): Body<crate::OccurrenceChange>,
) -> ApiResult<StatusCode> {
    shared
        .calendar()?
        .change_occurrence(&caller.actor, id(&entry)?, moment(&at)?, body)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Takes back the change of one occurrence.
async fn restore_occurrence(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((entry, at)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    shared
        .calendar()?
        .restore_occurrence(&caller.actor, id(&entry)?, moment(&at)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// A feed to make.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewFeed {
    /// What its owner calls it.
    name: String,
}

/// The feeds of the caller.
async fn feeds(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.calendar()?.feeds(&caller.actor).await?).into_response())
}

/// Makes a feed; its secret address is shown once.
async fn create_feed(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<NewFeed>,
) -> ApiResult<Response> {
    let (feed, secret) = shared.calendar()?.create_feed(&caller.actor, &body.name).await?;
    let url = format!("/api/v1/{}/feed/{secret}", crate::module::NAME);
    Ok((
        StatusCode::CREATED,
        Json(NewFeedAnswer {
            feed,
            secret: secret.clone(),
            url,
        }),
    )
        .into_response())
}

/// Revokes a feed.
async fn revoke_feed(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(feed): Path<String>,
) -> ApiResult<StatusCode> {
    shared.calendar()?.revoke_feed(&caller.actor, id(&feed)?).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The calendar of a feed, as iCalendar, for the application that holds its secret address.
async fn feed_calendar(State(shared): State<Shared>, Path(secret): Path<String>) -> ApiResult<Response> {
    let secret = secret.trim_end_matches(".ics");
    let text = shared.calendar()?.feed_calendar(secret).await?;
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (axum::http::header::CACHE_CONTROL, "private, no-store"),
        ],
        text,
    )
        .into_response())
}

/// A subscription to make.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewSubscription {
    /// Where the calendar is read from.
    url: String,
    /// What it is called.
    title: String,
    /// Every how many minutes it is read.
    #[serde(default)]
    refresh_minutes: Option<i32>,
}

/// The subscriptions of a scope.
async fn subscriptions(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .calendar()?
            .subscriptions(&caller.actor, id(&scope)?)
            .await?,
    )
    .into_response())
}

/// Subscribes a scope to a calendar.
async fn subscribe(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(body): Body<NewSubscription>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    if let Some(identity) = shared.identity.as_ref() {
        caller.acting(identity, MANAGE, scope).await?;
    }
    let made = shared
        .calendar()?
        .subscribe(&caller.actor, scope, &body.url, &body.title, body.refresh_minutes)
        .await?;
    Ok((StatusCode::CREATED, Json(made)).into_response())
}

/// Removes a subscription.
async fn unsubscribe(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(subscription): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .calendar()?
        .unsubscribe(&caller.actor, id(&subscription)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reads a subscription again now.
async fn refresh(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(subscription): Path<String>,
) -> ApiResult<Response> {
    let done = shared
        .calendar()?
        .refresh_now(&caller.actor, id(&subscription)?)
        .await?;
    Ok(Json(done).into_response())
}
