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

//! Getting an account and signing in: sign-up, confirmation, invitations, the second factor, step-up,
//! sign-out, recovery and the confirmation of a new email.

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json, Router};
use kampus_module::http::{ApiResult, Body};
use serde::Deserialize;
use serde_json::json;

use super::{Caller, SESSION_COOKIE, Shared, cookie};
use crate::service::account::SignUp;
use crate::service::second_factor::Proof;
use crate::service::session::NewSession;
use crate::service::sign_in::{Credential, SignIn, SignInOutcome};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/auth/sign-up", post(sign_up))
        .route("/auth/confirm", post(confirm))
        .route("/auth/invitations/accept", post(accept))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/second-factor", post(second_factor))
        .route("/auth/second-factor/email-code", post(email_code))
        .route("/auth/step-up", post(step_up))
        .route("/auth/sign-out", post(sign_out))
        .route("/auth/recovery", post(recovery))
        .route("/auth/recovery/needs", post(recovery_needs))
        .route("/auth/recovery/complete", post(complete_recovery))
        .route("/auth/email/confirm", post(confirm_email))
}

/// A sign-up.
#[derive(Debug, Deserialize)]
struct SignUpBody {
    /// The email.
    email: String,
    /// Whether the campus terms were accepted.
    #[serde(default)]
    accepted_terms: bool,
}

/// A one-time token from a link and the password chosen.
#[derive(Debug, Deserialize)]
struct TokenPassword {
    /// The token of the link.
    token: String,
    /// The password.
    password: String,
}

/// A one-time token from a link.
#[derive(Debug, Deserialize)]
struct TokenBody {
    /// The token of the link.
    token: String,
}

/// A sign-in with a password.
#[derive(Debug, Deserialize)]
struct SignInBody {
    /// The username or email.
    name: String,
    /// The password.
    password: String,
}

/// A second factor, as sent.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum ProofBody {
    /// A code of an authenticator app.
    Totp {
        /// The code.
        code: String,
    },
    /// A recovery code.
    RecoveryCode {
        /// The code.
        code: String,
    },
    /// A code sent by email.
    Email {
        /// The code.
        code: String,
    },
    /// A first authenticator app set up while signing in.
    EnrolTotp {
        /// Its name.
        label: String,
        /// The secret shown.
        secret: String,
        /// A current code.
        code: String,
    },
}

impl From<ProofBody> for Proof {
    fn from(body: ProofBody) -> Self {
        match body {
            ProofBody::Totp { code } => Self::Totp(code),
            ProofBody::RecoveryCode { code } => Self::RecoveryCode(code),
            ProofBody::Email { code } => Self::EmailCode(code),
            ProofBody::EnrolTotp { label, secret, code } => Self::EnrolTotp { label, secret, code },
        }
    }
}

/// The second factor that completes a sign-in.
#[derive(Debug, Deserialize)]
struct SecondFactorBody {
    /// The challenge of the sign-in.
    challenge: String,
    /// The factor.
    proof: ProofBody,
}

/// The challenge a code by email is sent for.
#[derive(Debug, Deserialize)]
struct ChallengeBody {
    /// The challenge of the sign-in.
    challenge: String,
}

/// The password, again.
#[derive(Debug, Deserialize)]
struct PasswordBody {
    /// The password.
    password: String,
}

/// An email.
#[derive(Debug, Deserialize)]
struct EmailBody {
    /// The email.
    email: String,
}

/// The end of a recovery.
#[derive(Debug, Deserialize)]
struct CompleteRecoveryBody {
    /// The token of the link.
    token: String,
    /// The second factor, when the account has one.
    proof: Option<ProofBody>,
    /// The new password.
    password: String,
}

/// A response that opens a session: the cookie and what the client may show.
fn opened(shared: &Shared, session: &NewSession, recovery_codes: Option<&[String]>) -> ApiResult<Response> {
    let cookie = shared.session_cookie(&session.token, session.session.expires_at)?;
    let body = json!({
        "status": "signed_in",
        "session": session.session,
        "recovery_codes": recovery_codes,
    });
    Ok(([(header::SET_COOKIE, cookie)], Json(body)).into_response())
}

/// The response to a sign-in that passed its credential.
pub(super) fn signed_in(shared: &Shared, outcome: SignInOutcome) -> ApiResult<Response> {
    match outcome {
        SignInOutcome::SignedIn(session) => opened(shared, &session, None),
        SignInOutcome::SecondFactorRequired { challenge, methods } => Ok(Json(json!({
            "status": "second_factor_required",
            "challenge": challenge,
            "methods": methods,
        }))
        .into_response()),
    }
}

/// Signs up a visitor; the answer is the same whether the email has an account or not.
async fn sign_up(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<SignUpBody>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .sign_up(
            caller.campus,
            SignUp {
                email: body.email,
                accepted_terms: body.accepted_terms,
            },
        )
        .await?;
    Ok(StatusCode::ACCEPTED)
}

/// Confirms a sign-up with its link and the password chosen.
async fn confirm(State(shared): State<Shared>, Body(body): Body<TokenPassword>) -> ApiResult<StatusCode> {
    shared.identity()?.confirm(&body.token, &body.password).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Accepts an invitation with its link and the password chosen.
async fn accept(State(shared): State<Shared>, Body(body): Body<TokenPassword>) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .accept_invitation(&body.token, &body.password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Signs in with a password.
async fn sign_in(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<SignInBody>,
) -> ApiResult<Response> {
    let outcome = shared
        .identity()?
        .sign_in(
            caller.campus,
            SignIn {
                name: body.name,
                credential: Credential::Password(body.password),
                client: caller.client.clone(),
            },
        )
        .await?;
    signed_in(&shared, outcome)
}

/// Completes a sign-in with its second factor.
async fn second_factor(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<SecondFactorBody>,
) -> ApiResult<Response> {
    let completed = shared
        .identity()?
        .complete_sign_in(&body.challenge, body.proof.into(), &caller.client)
        .await?;
    opened(&shared, &completed.session, completed.recovery_codes.as_deref())
}

/// Sends the code by email of a pending sign-in.
async fn email_code(State(shared): State<Shared>, Body(body): Body<ChallengeBody>) -> ApiResult<StatusCode> {
    shared.identity()?.send_email_code(&body.challenge).await?;
    Ok(StatusCode::ACCEPTED)
}

/// Proves the password again, for the operations that need a recent strong authentication.
async fn step_up(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<PasswordBody>,
) -> ApiResult<Response> {
    let session = shared
        .identity()?
        .step_up(caller.session()?, &body.password)
        .await?;
    Ok(Json(json!({ "session": session })).into_response())
}

/// Ends the session of the cookie.
async fn sign_out(State(shared): State<Shared>, headers: axum::http::HeaderMap) -> ApiResult<Response> {
    if let Some(token) = cookie(&headers, SESSION_COOKIE) {
        shared.identity()?.sign_out(token).await?;
    }
    Ok((
        [(header::SET_COOKIE, shared.cleared_session()?)],
        StatusCode::NO_CONTENT,
    )
        .into_response())
}

/// Sends a recovery link when the email has an active account; the answer is always the same.
async fn recovery(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<EmailBody>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .start_recovery(caller.campus, &body.email)
        .await?;
    Ok(StatusCode::ACCEPTED)
}

/// What completing a recovery needs.
async fn recovery_needs(State(shared): State<Shared>, Body(body): Body<TokenBody>) -> ApiResult<Response> {
    let needs = shared.identity()?.recovery_needs(&body.token).await?;
    Ok(Json(json!({ "methods": needs.methods })).into_response())
}

/// Completes a recovery with a new password.
async fn complete_recovery(
    State(shared): State<Shared>,
    Body(body): Body<CompleteRecoveryBody>,
) -> ApiResult<StatusCode> {
    shared
        .identity()?
        .complete_recovery(&body.token, body.proof.map(Into::into), &body.password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Confirms a new email with the link sent to it.
async fn confirm_email(State(shared): State<Shared>, Body(body): Body<TokenBody>) -> ApiResult<StatusCode> {
    shared.identity()?.confirm_email_change(&body.token).await?;
    Ok(StatusCode::NO_CONTENT)
}
