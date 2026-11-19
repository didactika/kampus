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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The accounts of a campus, for those who manage them: invitations, service accounts, reading an account,
//! the changes of its status with `If-Match`, and the API tokens of other accounts.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::{Error, FieldError};
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use serde::Deserialize;
use serde_json::json;

use super::me::{TokenBody, created_token};
use super::{Caller, Shared};
use crate::permissions::{ACCOUNT_DELETE, ACCOUNT_INVITE, ACCOUNT_READ, ACCOUNT_SUSPEND, TOKEN_MANAGE_ANY};
use crate::{AccountId, TokenId};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/campuses/{campus}/accounts", post(invite))
        .route(
            "/campuses/{campus}/service-accounts",
            post(create_service_account),
        )
        .route("/accounts/{id}", get(account))
        .route("/accounts/{id}/transitions", post(transition))
        .route("/accounts/{id}/tokens", post(create_token))
        .route("/tokens/{id}", delete(revoke_token))
}

/// An invitation.
#[derive(Debug, Deserialize)]
struct InviteBody {
    /// The email.
    email: String,
    /// The username; the email when absent.
    username: Option<String>,
}

/// A service account.
#[derive(Debug, Deserialize)]
struct ServiceBody {
    /// Its username.
    username: String,
}

/// A change of status.
#[derive(Debug, Deserialize)]
struct TransitionBody {
    /// `suspended`, `active`, `deletion_scheduled` or `deletion_cancelled`.
    to: String,
    /// Why; kept with the change.
    #[serde(default)]
    reason: String,
}

/// A created account.
fn created(account: AccountId) -> Response {
    (StatusCode::CREATED, Json(json!({ "id": account }))).into_response()
}

/// Invites a person.
async fn invite(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Body(body): Body<InviteBody>,
) -> ApiResult<Response> {
    let identity = shared.identity()?;
    let campus: ScopeId = id(&campus)?;
    let actor = caller.acting(identity, ACCOUNT_INVITE, campus).await?;
    let account = identity
        .invite(actor, campus, &body.email, body.username.as_deref())
        .await?;
    Ok(created(account))
}

/// Creates a service account.
async fn create_service_account(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Body(body): Body<ServiceBody>,
) -> ApiResult<Response> {
    let identity = shared.identity()?;
    let campus: ScopeId = id(&campus)?;
    let actor = caller.acting(identity, ACCOUNT_INVITE, campus).await?;
    let account = identity
        .create_service_account(actor, campus, &body.username)
        .await?;
    Ok(created(account))
}

/// Reads an account, with its version as `ETag`.
async fn account(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
) -> ApiResult<Response> {
    let identity = shared.identity()?;
    let account: AccountId = id(&account)?;
    let campus = identity.campus_of(account).await?;
    let actor = caller.acting(identity, ACCOUNT_READ, campus).await?;
    let read = identity.account(actor, account).await?;
    let version = read.version;
    Ok(with_etag(Json(read), version))
}

/// Changes the status of an account; `If-Match` refuses a change made on an old version.
async fn transition(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
    headers: HeaderMap,
    Body(body): Body<TransitionBody>,
) -> ApiResult<StatusCode> {
    let identity = shared.identity()?;
    let account: AccountId = id(&account)?;
    let expected = Some(if_match(&headers)?);
    let campus = identity.campus_of(account).await?;
    let permission = match body.to.as_str() {
        "suspended" | "active" => ACCOUNT_SUSPEND,
        "deletion_scheduled" | "deletion_cancelled" => ACCOUNT_DELETE,
        _ => {
            return Err(Error::validation(
                "identity.transition_invalid",
                vec![FieldError::new("to", "one_of")],
            )
            .into());
        }
    };
    let actor = caller.acting(identity, permission, campus).await?;
    match body.to.as_str() {
        "suspended" => identity.suspend(actor, account, &body.reason, expected).await?,
        "active" => {
            identity
                .reactivate(actor, account, &body.reason, expected)
                .await?;
        }
        "deletion_scheduled" => {
            identity
                .schedule_deletion(actor, account, &body.reason, expected)
                .await?;
        }
        _ => identity.cancel_deletion(actor, account, expected).await?,
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Creates an API token for another account, such as a service account.
async fn create_token(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
    Body(body): Body<TokenBody>,
) -> ApiResult<Response> {
    let identity = shared.identity()?;
    let account: AccountId = id(&account)?;
    let campus = identity.campus_of(account).await?;
    let actor = caller.acting(identity, TOKEN_MANAGE_ANY, campus).await?;
    let token = identity.create_token_for(actor, account, body.request()?).await?;
    Ok(created_token(&token))
}

/// Revokes the API token of any account.
async fn revoke_token(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(token): Path<String>,
) -> ApiResult<StatusCode> {
    let identity = shared.identity()?;
    let token: TokenId = id(&token)?;
    let actor = caller.acting(identity, TOKEN_MANAGE_ANY, caller.campus).await?;
    identity.revoke_token(actor, token).await?;
    Ok(StatusCode::NO_CONTENT)
}
