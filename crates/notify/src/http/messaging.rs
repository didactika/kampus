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

//! The routes of messaging: the conversations of the caller, their messages and the people they blocked.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_access::PrincipalId;
use kampus_common::PageRequest;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::Shared;
use crate::service::conversation::Opening;
use crate::{ConversationId, MessageId};

/// The routes of messaging, under those of notify.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/conversations", get(conversations).post(open))
        .route("/conversations/{id}", get(conversation))
        .route("/conversations/{id}/messages", get(messages).post(send))
        .route("/conversations/{id}/read", post(read))
        .route("/conversations/{id}/mute", put(mute))
        .route("/conversations/{id}/membership", axum::routing::delete(leave))
        .route("/messages/{id}", axum::routing::delete(delete_message))
        .route("/blocks", get(blocked))
        .route("/blocks/{account}", put(block).delete(unblock))
}

/// A page of a list.
#[derive(Debug, Deserialize)]
struct Listing {
    /// How many items.
    limit: Option<u32>,
    /// Where the page starts.
    after: Option<String>,
}

impl Listing {
    /// The page asked for.
    fn page(&self) -> kampus_common::Result<PageRequest> {
        PageRequest::new(self.limit, self.after.as_deref())
    }
}

/// A message to send.
#[derive(Debug, Deserialize)]
struct Writing {
    /// What it says, in rich text.
    body: String,
}

/// Whether a conversation is muted.
#[derive(Debug, Deserialize)]
struct Muting {
    /// Muted or not.
    muted: bool,
}

/// The conversations of the caller.
async fn conversations(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Query(listing): Query<Listing>,
) -> ApiResult<Response> {
    let page = shared
        .notify()?
        .conversations(&caller.actor, &listing.page()?)
        .await?;
    Ok(Json(page).into_response())
}

/// Opens a conversation.
async fn open(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(opening): Body<Opening>,
) -> ApiResult<Response> {
    let opened = shared.notify()?.open_conversation(&caller.actor, opening).await?;
    Ok((StatusCode::CREATED, Json(opened)).into_response())
}

/// A conversation of the caller.
async fn conversation(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(conversation): Path<String>,
) -> ApiResult<Response> {
    let found = shared
        .notify()?
        .conversation(&caller.actor, id::<ConversationId>(&conversation)?)
        .await?;
    Ok(Json(found).into_response())
}

/// The messages of a conversation.
async fn messages(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(conversation): Path<String>,
    Query(listing): Query<Listing>,
) -> ApiResult<Response> {
    let page = shared
        .notify()?
        .messages(
            &caller.actor,
            id::<ConversationId>(&conversation)?,
            &listing.page()?,
        )
        .await?;
    Ok(Json(page).into_response())
}

/// Sends a message.
async fn send(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(conversation): Path<String>,
    Body(writing): Body<Writing>,
) -> ApiResult<Response> {
    let sent = shared
        .notify()?
        .send_message(&caller.actor, id::<ConversationId>(&conversation)?, &writing.body)
        .await?;
    Ok((StatusCode::CREATED, Json(sent)).into_response())
}

/// Marks a conversation read.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(conversation): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .mark_conversation_read(&caller.actor, id::<ConversationId>(&conversation)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Mutes or unmutes a conversation.
async fn mute(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(conversation): Path<String>,
    Body(muting): Body<Muting>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .mute(&caller.actor, id::<ConversationId>(&conversation)?, muting.muted)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Leaves a group conversation.
async fn leave(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(conversation): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .leave(&caller.actor, id::<ConversationId>(&conversation)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a message for the caller.
async fn delete_message(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(message): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .delete_message(&caller.actor, id::<MessageId>(&message)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The accounts the caller blocked.
async fn blocked(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.notify()?.blocked(&caller.actor).await?).into_response())
}

/// Blocks an account.
async fn block(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .block(&caller.actor, id::<PrincipalId>(&account)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes a block.
async fn unblock(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .notify()?
        .unblock(&caller.actor, id::<PrincipalId>(&account)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
