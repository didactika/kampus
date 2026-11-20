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

//! The routes of profile pictures: setting and removing one's own, and reading whose file is someone's
//! picture; the file itself is read through files.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_files::FileId;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;
use serde_json::json;

use super::Shared;

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/me/picture", put(set).delete(remove))
        .route("/accounts/{id}/picture", get(read))
}

/// The file to make one's picture.
#[derive(Debug, Deserialize)]
struct PictureBody {
    /// A file uploaded into `users.picture`.
    file: FileId,
}

/// Makes an uploaded file the caller's profile picture.
async fn set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<PictureBody>,
) -> ApiResult<StatusCode> {
    shared.users()?.set_picture(&caller.actor, body.file).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes the caller's profile picture.
async fn remove(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<StatusCode> {
    shared.users()?.remove_picture(&caller.actor).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The file of someone's profile picture.
async fn read(State(shared): State<Shared>, Path(account): Path<String>) -> ApiResult<Response> {
    let file = shared
        .users()?
        .picture(id(&account)?)
        .await?
        .ok_or_else(|| kampus_common::Error::not_found("users.picture_not_found"))?;
    Ok(Json(json!({ "file": file })).into_response())
}
