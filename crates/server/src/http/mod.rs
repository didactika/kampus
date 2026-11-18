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

//! The HTTP layer of every role: liveness and readiness (DEC-019), request identifiers on every request
//! and response, and problem details (RFC 9457) for errors (DEC-018).

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use kampus_common::{Error, PLATFORM_VERSION, RequestId};
use kampus_db::{Database, ModuleSteps, Upgrader};
use serde_json::json;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

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
}

/// Builds the router of a process.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .fallback(not_found)
        .with_state(state)
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
            let id = request.headers().get(REQUEST_ID).and_then(|v| v.to_str().ok()).unwrap_or_default();
            tracing::info_span!("request", method = %request.method(), path = %request.uri().path(), request_id = %id)
        }))
        .layer(SetRequestIdLayer::new(REQUEST_ID, KampusRequestId))
}

/// Gives every request an identifier, keeping a safe one sent by the caller or a proxy.
#[derive(Debug, Clone, Copy)]
struct KampusRequestId;

impl MakeRequestId for KampusRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<tower_http::request_id::RequestId> {
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
    if reasons.is_empty() {
        axum::Json(json!({ "status": "ready", "version": PLATFORM_VERSION })).into_response()
    } else {
        let body = json!({ "status": "not_ready", "version": PLATFORM_VERSION, "reasons": reasons });
        (StatusCode::SERVICE_UNAVAILABLE, axum::Json(body)).into_response()
    }
}

/// Answers routes that do not exist with problem details.
async fn not_found(headers: HeaderMap) -> Response {
    problem(&Error::not_found("server.route_not_found"), &headers)
}

/// Turns an error into problem details: kind, code, parameters, fields and the request id; internal
/// errors expose only the request id (DEC-018).
pub fn problem(error: &Error, headers: &HeaderMap) -> Response {
    let request_id = headers
        .get(REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let status =
        StatusCode::from_u16(error.kind().http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let internal = error.kind() == kampus_common::ErrorKind::Internal;
    let body = json!({
        "type": format!("https://kampus.didactika.org/problems/{}", error.kind()),
        "title": error.kind().as_str(),
        "status": status.as_u16(),
        "code": if internal { "internal" } else { error.code() },
        "params": if internal { json!({}) } else { json!(error.params().iter().cloned().collect::<std::collections::BTreeMap<_, _>>()) },
        "fields": if internal { json!([]) } else { json!(error.fields()) },
        "request_id": request_id,
    });
    let mut response = (status, axum::Json(body)).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}
