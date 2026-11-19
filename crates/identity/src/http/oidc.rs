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

//! Signing in through an identity provider and configuring the providers of a campus. The state of a
//! sign-in waits in a cookie while the browser is at the provider.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::Error;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;
use serde_json::json;

use super::{Caller, FLOW_COOKIE, Shared, cookie};
use crate::permissions::PROVIDER_MANAGE;
use crate::service::oidc::{OidcOutcome, ProviderConfig};
use crate::{LinkingPolicy, ProviderId};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route(
            "/campuses/{campus}/identity-providers",
            get(providers).post(add_provider),
        )
        .route("/auth/oidc/{provider}/start", get(start))
        .route("/auth/oidc/{provider}/callback", get(callback))
}

/// A provider to configure.
#[derive(Debug, Deserialize)]
struct ProviderBody {
    /// The issuer URL.
    issuer: String,
    /// The client id at the provider.
    client: String,
    /// The client secret.
    client_secret: String,
    /// `subject_only`, `verified_email` or `confirm`.
    linking_policy: String,
}

/// What a provider sends back.
#[derive(Debug, Deserialize)]
struct CallbackQuery {
    /// The authorization code.
    code: String,
    /// The state of the sign-in.
    state: String,
}

/// The providers of a campus, for its sign-in page.
async fn providers(State(shared): State<Shared>, Path(campus): Path<String>) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let providers = shared.identity()?.providers(campus).await?;
    Ok(Json(providers).into_response())
}

/// Configures a provider.
async fn add_provider(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(campus): Path<String>,
    Body(body): Body<ProviderBody>,
) -> ApiResult<Response> {
    let identity = shared.identity()?;
    let campus: ScopeId = id(&campus)?;
    let linking_policy: LinkingPolicy = body.linking_policy.parse()?;
    let actor = caller.acting(identity, PROVIDER_MANAGE, campus).await?;
    let provider = identity
        .add_provider(
            actor,
            campus,
            ProviderConfig {
                issuer: body.issuer,
                client: body.client,
                client_secret: body.client_secret,
                linking_policy,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": provider }))).into_response())
}

/// Sends the browser to the provider, keeping the state of the sign-in in a cookie.
async fn start(State(shared): State<Shared>, Path(provider): Path<String>) -> ApiResult<Response> {
    let provider: ProviderId = id(&provider)?;
    let redirect = format!(
        "{}/auth/oidc/{provider}/callback",
        shared.settings()?.api_url.trim_end_matches('/')
    );
    let started = shared.identity()?.oidc_start(provider, &redirect).await?;
    Ok((
        StatusCode::SEE_OTHER,
        [
            (header::SET_COOKIE, shared.flow_cookie(&started.flow)?),
            (
                header::LOCATION,
                started
                    .authorize_url
                    .parse()
                    .map_err(|_| Error::internal("identity.oidc_url_invalid"))?,
            ),
        ],
    )
        .into_response())
}

/// Finishes the sign-in with what the provider sent back.
async fn callback(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> ApiResult<Response> {
    let flow = cookie(&headers, FLOW_COOKIE)
        .ok_or_else(|| Error::validation("identity.oidc_flow_invalid", vec![]))?;
    let outcome = shared
        .identity()?
        .oidc_callback(flow, &query.code, &query.state, &caller.client)
        .await?;
    let mut response = match outcome {
        OidcOutcome::SignedIn(outcome) => super::auth::signed_in(&shared, outcome)?,
        OidcOutcome::ConfirmLink { link } => {
            Json(json!({ "status": "confirm_link", "link": link })).into_response()
        }
    };
    response
        .headers_mut()
        .append(header::SET_COOKIE, shared.flow_cookie("")?);
    Ok(response)
}
