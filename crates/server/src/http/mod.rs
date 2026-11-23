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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP layer of every role: liveness and readiness, request identifiers on every request
//! and response, problem details (RFC 9457) for errors, and, for the roles that serve the API, the routes of
//! every module behind the campus of the request and its authentication.

pub mod campus;
pub mod client;
pub mod metrics;
pub mod no_store;
pub mod openapi;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use kampus_common::{Error, PLATFORM_VERSION, RequestId};
use kampus_db::{Database, ModuleSteps, Upgrader};
use kampus_module::http::{REQUEST_ID as CURRENT_REQUEST, problem};
use serde_json::json;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use crate::wiring::services::Services;

/// The path every module route is served under.
pub const API_PREFIX: &str = "/api/v1";

/// The header that carries the request identifier.
pub const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// What the handlers share.
#[derive(Debug, Clone)]
pub struct AppState {
    /// The database of the installation.
    pub db: Database,
    /// The upgrade steps of this build, to tell whether the database is current.
    pub modules: &'static [ModuleSteps],
    /// The other ports of the process.
    pub ports: crate::wiring::ports::Ports,
    /// The proxies whose `X-Forwarded-For` tells the address of people.
    pub trusted_proxies: std::sync::Arc<client::TrustedProxies>,
}

/// The API of the modules: their services and how the campus of a request is found.
#[derive(Debug, Clone)]
pub struct Api {
    /// The services of the modules.
    pub services: Services,
    /// Finds the campus of each request.
    pub campuses: campus::Campuses,
}

impl Api {
    /// The routes of every module, behind the campus of the request and its authentication.
    fn router(&self) -> Router {
        let routes = self.services.routes();
        let document = openapi::document(
            &routes,
            API_PREFIX,
            crate::wiring::registry::MODULES,
            kampus_common::PLATFORM_VERSION,
        );
        routes
            .route(
                "/openapi.json",
                get(move || {
                    let document = document.clone();
                    async move { axum::Json(document) }
                }),
            )
            .layer(middleware::from_fn_with_state(
                self.services.identity.clone(),
                kampus_identity::http::authenticate,
            ))
            .layer(middleware::from_fn_with_state(
                self.campuses.clone(),
                campus::resolve,
            ))
            .layer(middleware::from_fn(no_store::no_store))
    }
}

/// Builds the router of a process; `api` adds the routes of the modules.
pub fn router(state: AppState, api: Option<&Api>) -> Router {
    let trusted = state.trusted_proxies.clone();
    let mut router = Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/metrics", get(metrics::render))
        .with_state(state);
    if let Some(api) = api {
        router = router.nest(API_PREFIX, api.router());
    }
    router
        .fallback(not_found)
        .layer(middleware::from_fn(metrics::measure))
        .layer(middleware::from_fn_with_state(trusted, client::resolve))
        .layer(middleware::from_fn(scoped))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
            let id = request.headers().get(REQUEST_ID).and_then(|v| v.to_str().ok()).unwrap_or_default();
            tracing::info_span!("request", method = %request.method(), path = %request.uri().path(), request_id = %id)
        }))
        .layer(SetRequestIdLayer::new(REQUEST_ID, KampusRequestId))
}

/// Serves a request with its id at hand, so problem details carry it.
async fn scoped(request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get(REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    CURRENT_REQUEST.scope(id, next.run(request)).await
}

/// Gives every request an identifier, keeping a safe one sent by the caller or a proxy.
#[derive(Debug, Clone, Copy)]
struct KampusRequestId;

impl MakeRequestId for KampusRequestId {
    fn make_request_id<B>(
        &mut self,
        _request: &axum::http::Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        HeaderValue::from_str(RequestId::generate().as_str())
            .ok()
            .map(tower_http::request_id::RequestId::new)
    }
}

/// The process is running.
async fn live() -> Response {
    axum::Json(json!({ "status": "live", "version": PLATFORM_VERSION })).into_response()
}
/// The process can serve: the database answers and its upgrade steps match this build, and the storage and
/// the cache answer.
async fn ready(State(state): State<AppState>) -> Response {
    let mut reasons = Vec::new();
    match state.db.ping().await {
        Err(error) => reasons.push(error.to_string()),
        Ok(()) => match Upgrader::new(&state.db, state.modules, PLATFORM_VERSION)
            .plan()
            .await
        {
            Ok(plan) if plan.is_current() => {}
            Ok(plan) => {
                if !plan.installed {
                    reasons.push("the installation does not exist".to_owned());
                }
                reasons.extend(plan.pending.iter().map(|step| format!("pending step {step}")));
                reasons.extend(plan.problems.iter().map(ToString::to_string));
            }
            Err(error) => reasons.push(error.to_string()),
        },
    }
    if let Some(storage) = &state.ports.storage
        && let Err(error) = storage.check().await
    {
        reasons.push(format!("storage: {error}"));
    }
    if let Err(error) = state.ports.cache.check().await {
        reasons.push(format!("cache: {error}"));
    }
    if let Some(scanner) = &state.ports.scanner
        && let Err(error) = scanner.check().await
    {
        reasons.push(format!("scanner: {error}"));
    }
    if reasons.is_empty() {
        axum::Json(json!({ "status": "ready", "version": PLATFORM_VERSION })).into_response()
    } else {
        let body = json!({ "status": "not_ready", "version": PLATFORM_VERSION, "reasons": reasons });
        (StatusCode::SERVICE_UNAVAILABLE, axum::Json(body)).into_response()
    }
}

/// Answers routes that do not exist with problem details.
async fn not_found() -> Response {
    problem(&Error::not_found("server.route_not_found"))
}
