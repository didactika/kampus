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

//! The account of the person signed in: its password and email, sessions, API tokens, second factors and
//! the confirmation of a provider link. Credentials are managed with a session, never with a token.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use kampus_common::Timestamp;
use kampus_module::Operation;
use kampus_module::http::{ApiResult, Body, id, with_etag};
use serde::{Deserialize, Serialize};

use super::{Caller, Shared};
use crate::service::api_token::NewToken;
use crate::service::api_token::{TokenRequest, TokenScope};
use crate::service::second_factor::{FactorAdded, TotpSetup};
use crate::{Account, FactorId, SecondFactor, SessionSummary};

/// The operations of the account of the caller.
pub(super) const OPERATIONS: &[Operation] = &[
    Operation::get("/me", "The account of the caller, with its version as ETag").returns::<Account>(),
    Operation::post("/me/password", "Change the password, proving the current one")
        .body::<PasswordChange>()
        .no_content(),
    Operation::post("/me/email", "Change the email; a link to the new one confirms it")
        .body::<EmailBody>()
        .accepted(),
    Operation::get("/me/sessions", "The open sessions of the caller").returns::<Vec<SessionSummary>>(),
    Operation::delete("/me/sessions/{id}", "Close a session of the caller").no_content(),
    Operation::get("/me/tokens", "The API tokens of the caller")
        .returns::<Vec<crate::service::api_token::TokenSummary>>(),
    Operation::post("/me/tokens", "Create an API token; shown this once")
        .body::<TokenBody>()
        .creates::<NewToken>(),
    Operation::delete("/me/tokens/{id}", "Revoke an API token of the caller").no_content(),
    Operation::get("/me/second-factors", "The second factors of the caller").returns::<Vec<SecondFactor>>(),
    Operation::post(
        "/me/second-factors/totp",
        "A new secret for an authenticator app, to confirm",
    )
    .returns::<TotpSetup>(),
    Operation::post(
        "/me/second-factors/totp/confirm",
        "Add an authenticator app with a code it shows",
    )
    .body::<TotpBody>()
    .creates::<Added>(),
    Operation::post(
        "/me/second-factors/email",
        "Add codes by email as a second factor",
    )
    .body::<LabelBody>()
    .creates::<Added>(),
    Operation::delete("/me/second-factors/{id}", "Remove a second factor").no_content(),
    Operation::post(
        "/me/recovery-codes",
        "A new set of recovery codes, replacing the old ones",
    )
    .creates::<RecoveryCodes>(),
    Operation::post(
        "/me/identity-links",
        "Link an identity a provider vouched for to the account",
    )
    .body::<LinkBody>()
    .no_content(),
];

/// An added second factor.
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Added {
    /// The factor.
    id: FactorId,
    /// A new set of recovery codes, shown this once, when the account had none.
    recovery_codes: Option<Vec<String>>,
}

/// A new set of recovery codes, shown this once.
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct RecoveryCodes {
    /// The codes.
    recovery_codes: Vec<String>,
}

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/me", get(account))
        .route("/me/password", post(change_password))
        .route("/me/email", post(change_email))
        .route("/me/sessions", get(sessions))
        .route("/me/sessions/{id}", delete(revoke_session))
        .route("/me/tokens", get(tokens).post(create_token))
        .route("/me/tokens/{id}", delete(revoke_token))
        .route("/me/second-factors", get(second_factors))
        .route("/me/second-factors/totp", post(new_totp))
        .route("/me/second-factors/totp/confirm", post(confirm_totp))
        .route("/me/second-factors/email", post(add_email))
        .route("/me/second-factors/{id}", delete(remove_factor))
        .route("/me/recovery-codes", post(recovery_codes))
        .route("/me/identity-links", post(confirm_link))
}

/// A change of password.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PasswordChange {
    /// The current password.
    current: String,
    /// The new password.
    new: String,
}

/// A new email.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EmailBody {
    /// The email.
    email: String,
}

/// A permission a new token carries.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ScopeBody {
    /// The permission.
    permission: String,
    /// The scope, by id.
    scope: String,
}

/// A token to create.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct TokenBody {
    /// Its name.
    name: String,
    /// What it may do.
    scopes: Vec<ScopeBody>,
    /// When it expires.
    expires_at: Option<Timestamp>,
}

impl TokenBody {
    /// The request of the service.
    pub(super) fn request(self) -> ApiResult<TokenRequest> {
        let scopes = self
            .scopes
            .into_iter()
            .map(|s| {
                Ok(TokenScope {
                    permission: s.permission,
                    scope: id(&s.scope)?,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?;
        Ok(TokenRequest {
            name: self.name,
            scopes,
            expires_at: self.expires_at,
        })
    }
}

/// The name of a factor.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LabelBody {
    /// The name.
    label: String,
}

/// An authenticator app to confirm.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TotpBody {
    /// Its name.
    label: String,
    /// The secret shown.
    secret: String,
    /// A current code.
    code: String,
}

/// A provider link to confirm.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LinkBody {
    /// The sealed link.
    link: String,
}

/// A created token: shown this once.
pub(super) fn created_token(token: &crate::service::api_token::NewToken) -> Response {
    (StatusCode::CREATED, Json(token)).into_response()
}

/// An added factor, with the recovery codes of a first factor.
fn added(factor: &FactorAdded) -> Response {
    (
        StatusCode::CREATED,
        Json(Added {
            id: factor.factor,
            recovery_codes: factor.recovery_codes.clone(),
        }),
    )
        .into_response()
}

/// The account signed in.
async fn account(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    let account = shared
        .identity()?
        .account(&caller.actor, caller.account()?)
        .await?;
    let version = account.version;
    Ok(with_etag(Json(account), version))
}

/// Changes the password; every session and token ends.
async fn change_password(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<PasswordChange>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .change_password(caller.session()?, &body.current, &body.new)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Starts a change of email: a link goes to the new address.
async fn change_email(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<EmailBody>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .change_email(caller.session()?, &body.email)
        .await?;
    Ok(StatusCode::ACCEPTED)
}

/// The open sessions.
async fn sessions(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    let sessions = shared.identity()?.sessions(caller.session()?).await?;
    Ok(Json(sessions).into_response())
}

/// Ends one of the sessions.
async fn revoke_session(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(session): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .revoke_session(caller.session()?, id(&session)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The API tokens.
async fn tokens(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    let tokens = shared.identity()?.tokens(caller.session()?).await?;
    Ok(Json(tokens).into_response())
}

/// Creates an API token.
async fn create_token(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<TokenBody>,
) -> ApiResult<Response> {
    let token = shared
        .identity()?
        .create_own_token(caller.session()?, body.request()?)
        .await?;
    Ok(created_token(&token))
}

/// Revokes an API token.
async fn revoke_token(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(token): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .revoke_own_token(caller.session()?, id(&token)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The second factors.
async fn second_factors(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    let factors = shared.identity()?.second_factors(caller.session()?).await?;
    Ok(Json(factors).into_response())
}

/// A new secret for an authenticator app.
async fn new_totp(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    let setup = shared.identity()?.new_totp_secret(caller.session()?).await?;
    Ok(Json(setup).into_response())
}

/// Adds an authenticator app once a current code proves it.
async fn confirm_totp(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<TotpBody>,
) -> ApiResult<Response> {
    let factor = shared
        .identity()?
        .confirm_totp(caller.session()?, &body.label, &body.secret, &body.code)
        .await?;
    Ok(added(&factor))
}

/// Adds codes by email.
async fn add_email(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<LabelBody>,
) -> ApiResult<Response> {
    let factor = shared
        .identity()?
        .add_email_factor(caller.session()?, &body.label)
        .await?;
    Ok(added(&factor))
}

/// Removes a second factor.
async fn remove_factor(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(factor): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .remove_second_factor(caller.session()?, id(&factor)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// A new set of recovery codes, replacing the old one.
async fn recovery_codes(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    let codes = shared
        .identity()?
        .regenerate_recovery_codes(caller.session()?)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(RecoveryCodes {
            recovery_codes: codes,
        }),
    )
        .into_response())
}

/// Links a provider identity the person confirmed.
async fn confirm_link(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<LinkBody>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .confirm_link(caller.session()?, &body.link)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
