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

//! The HTTP routes of identity and the authentication of every request: the session cookie or an API token
//! becomes the caller of the request, which every module reads. A request that changes something with the
//! cookie must carry the `X-Kampus` header, which other sites cannot send.

mod accounts;
mod auth;
mod me;
mod oidc;

use std::net::SocketAddr;

use axum::Router;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use kampus_access::{Acting, Actor, ScopeId};
use kampus_common::{Error, Result, Timestamp};
use kampus_module::ServiceContext;
use kampus_module::http::ApiError;

use crate::service::api_token::TokenAuth;
use crate::service::session::Client;
use crate::{Identity, Session};

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[kampus_module::Operation]] = &[
    auth::OPERATIONS,
    me::OPERATIONS,
    accounts::OPERATIONS,
    oidc::OPERATIONS,
];

/// The cookie that carries the session.
pub const SESSION_COOKIE: &str = "kampus_session";

/// The cookie that keeps the state of a sign-in at an identity provider.
pub const FLOW_COOKIE: &str = "kampus_oidc";

/// The header a cookie-authenticated request that changes something carries.
pub const CSRF_HEADER: &str = "x-kampus";

/// How long the flow cookie of a provider sign-in lasts, in seconds.
const FLOW_MAX_AGE: i64 = 10 * 60;

/// The campus a request is for, set by the server before authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentCampus(pub ScopeId);

/// What the routes of identity need from the configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpSettings {
    /// The public address of the site; cookies are secure when it is HTTPS.
    pub site_url: String,
    /// The public address the routes are served under, for the links of providers.
    pub api_url: String,
}

impl HttpSettings {
    /// Whether cookies are sent only over HTTPS.
    fn secure(&self) -> bool {
        self.site_url.starts_with("https://")
    }
}

/// Who makes a request.
#[derive(Debug, Clone)]
pub struct Caller {
    /// The actor, a visitor of the campus when nobody signed in.
    pub actor: Actor,
    /// The campus of the request.
    pub campus: ScopeId,
    /// The session, when the request carries one.
    pub session: Option<Session>,
    /// The API token, when the request carries one.
    pub token: Option<TokenAuth>,
    /// Where the request comes from.
    pub client: Client,
}

impl Caller {
    /// The session of a signed-in person; API tokens do not manage credentials.
    ///
    /// # Errors
    ///
    /// `unauthenticated` (`identity.sign_in_required`) without a session.
    pub fn session(&self) -> Result<&Session> {
        self.session
            .as_ref()
            .ok_or_else(|| Error::unauthenticated("identity.sign_in_required"))
    }

    /// The actor, when the request is authenticated by a session or a token.
    ///
    /// # Errors
    ///
    /// `unauthenticated` (`identity.sign_in_required`) for a visitor.
    pub fn signed_in(&self) -> Result<&Actor> {
        if matches!(self.actor, Actor::Visitor(_)) {
            Err(Error::unauthenticated("identity.sign_in_required"))
        } else {
            Ok(&self.actor)
        }
    }

    /// The account of a signed-in caller.
    ///
    /// # Errors
    ///
    /// `unauthenticated` (`identity.sign_in_required`) for a visitor.
    pub fn account(&self) -> Result<crate::AccountId> {
        match self.actor {
            Actor::Principal(account) => Ok(account),
            _ => Err(Error::unauthenticated("identity.sign_in_required")),
        }
    }

    /// What the request acts with, as the modules that cannot see a caller read it: the actor and what its
    /// API token carries.
    #[must_use]
    pub fn as_acting(&self) -> Acting {
        Acting {
            actor: self.actor,
            carried: self.token.as_ref().map(TokenAuth::carried),
        }
    }

    /// The actor for an operation that needs a permission in a scope: an API token acts only within what
    /// it carries, while a session acts as its account and the service checks the permission.
    ///
    /// # Errors
    ///
    /// `permission_denied` (`access.token_scope`) when the token does not carry the permission there.
    pub async fn acting(&self, identity: &Identity, permission: &str, scope: ScopeId) -> Result<&Actor> {
        if self.token.is_some() {
            identity
                .access
                .acting(&self.as_acting(), permission, scope)
                .await?;
        }
        Ok(&self.actor)
    }
}

/// The value of a cookie.
pub(crate) fn cookie<'h>(headers: &'h HeaderMap, name: &str) -> Option<&'h str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

/// A `Set-Cookie` header: HTTP only, same site, secure over HTTPS; an empty value removes the cookie.
fn set_cookie(settings: &HttpSettings, name: &str, value: &str, max_age: i64) -> Result<HeaderValue> {
    let secure = if settings.secure() { "; Secure" } else { "" };
    let age = if value.is_empty() { 0 } else { max_age };
    HeaderValue::from_str(&format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={age}{secure}"
    ))
    .map_err(|_| Error::internal("identity.cookie_invalid"))
}

/// Authenticates a request: the session cookie or a `Bearer` API token becomes its [`Caller`]; without
/// either, the caller is a visitor of the campus. An invalid token is refused; an expired cookie is a
/// visitor. Changes made with the cookie need the `X-Kampus` header.
pub async fn authenticate(State(identity): State<Identity>, mut request: Request, next: Next) -> Response {
    let campus = request.extensions().get::<CurrentCampus>().map(|c| c.0);
    let client = client_of(&request);
    let changes = !matches!(*request.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    match caller(&identity, campus, client, request.headers().clone(), changes).await {
        Ok(caller) => {
            request.extensions_mut().insert(caller.as_acting());
            request.extensions_mut().insert(caller);
            next.run(request).await
        }
        Err(error) => ApiError(error).into_response(),
    }
}

/// Where a request comes from.
fn client_of(request: &Request) -> Client {
    Client {
        name: request
            .headers()
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.chars().take(500).collect()),
        address: request
            .extensions()
            .get::<kampus_module::http::ClientAddress>()
            .map(|c| c.0)
            .or_else(|| {
                request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|c| c.0.ip())
            })
            .map(|address| address.to_string()),
    }
}

/// The caller of a request.
async fn caller(
    identity: &Identity,
    campus: Option<ScopeId>,
    client: Client,
    headers: HeaderMap,
    changes: bool,
) -> Result<Caller> {
    let campus = campus.ok_or_else(|| Error::not_found("identity.campus_unknown"))?;
    if let Some(bearer) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        let token = identity.authenticate_token(bearer.trim()).await?;
        return Ok(Caller {
            actor: Actor::Principal(token.account),
            campus,
            session: None,
            token: Some(token),
            client,
        });
    }
    if let Some(value) = cookie(&headers, SESSION_COOKIE)
        && let Ok(session) = identity.authenticate(value).await
    {
        if changes && !headers.contains_key(CSRF_HEADER) {
            return Err(Error::permission_denied("identity.csrf_header_missing"));
        }
        return Ok(Caller {
            actor: Actor::Principal(session.account),
            campus,
            session: Some(session),
            token: None,
            client,
        });
    }
    Ok(Caller {
        actor: Actor::Visitor(campus),
        campus,
        session: None,
        token: None,
        client,
    })
}

/// The routes of identity.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        identity: context.service::<Identity>().ok(),
        settings: context.service::<HttpSettings>().ok(),
    };
    Router::new()
        .merge(auth::routes())
        .merge(me::routes())
        .merge(accounts::routes())
        .merge(oidc::routes())
        .with_state(shared)
}

/// What the handlers of identity share.
#[derive(Debug, Clone)]
pub(crate) struct Shared {
    /// The service.
    identity: Option<Identity>,
    /// The settings of the routes.
    settings: Option<HttpSettings>,
}

impl Shared {
    /// The service.
    pub(crate) fn identity(&self) -> Result<&Identity> {
        self.identity
            .as_ref()
            .ok_or_else(|| Error::internal("identity.service_missing"))
    }

    /// The settings of the routes.
    pub(crate) fn settings(&self) -> Result<&HttpSettings> {
        self.settings
            .as_ref()
            .ok_or_else(|| Error::internal("identity.settings_missing"))
    }

    /// The session cookie of a new session.
    pub(crate) fn session_cookie(&self, token: &str, expires_at: Timestamp) -> Result<HeaderValue> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let age = (expires_at.as_offset_date_time().unix_timestamp() - now).max(0);
        set_cookie(self.settings()?, SESSION_COOKIE, token, age)
    }

    /// The removal of the session cookie.
    pub(crate) fn cleared_session(&self) -> Result<HeaderValue> {
        set_cookie(self.settings()?, SESSION_COOKIE, "", 0)
    }

    /// The flow cookie of a provider sign-in, or its removal with an empty flow.
    pub(crate) fn flow_cookie(&self, flow: &str) -> Result<HeaderValue> {
        set_cookie(self.settings()?, FLOW_COOKIE, flow, FLOW_MAX_AGE)
    }
}
