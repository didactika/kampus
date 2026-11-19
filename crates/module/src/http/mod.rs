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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What modules need to offer HTTP routes: the services the server composed, errors as problem details
//! (RFC 9457) with the request id, and the conditional requests of `If-Match`.

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use kampus_common::{Error, ErrorKind, FieldError};
use serde_json::json;

tokio::task_local! {
    /// The id of the request being served, set by the server around every request.
    pub static REQUEST_ID: String;
}

/// An error answered as problem details.
#[derive(Debug)]
pub struct ApiError(pub Error);

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        problem(&self.0)
    }
}

/// The result of a handler.
pub type ApiResult<T> = Result<T, ApiError>;

/// A JSON request body; a body that does not parse is a `validation` problem (`module.body_invalid`).
#[derive(Debug, Clone)]
pub struct Body<T>(pub T);

impl<S: Send + Sync, T: serde::de::DeserializeOwned> axum::extract::FromRequest<S> for Body<T> {
    type Rejection = ApiError;

    async fn from_request(request: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        axum::Json::<T>::from_request(request, state)
            .await
            .map(|json| Self(json.0))
            .map_err(|rejection| {
                ApiError(
                    Error::validation("module.body_invalid", vec![])
                        .with_param("reason", rejection.body_text()),
                )
            })
    }
}

/// An identifier read from a path segment.
///
/// # Errors
///
/// `not_found` (`module.id_invalid`) when the segment is not an identifier, as for an unknown one.
pub fn id<T: std::str::FromStr>(segment: &str) -> Result<T, ApiError> {
    segment
        .parse()
        .map_err(|_| ApiError(Error::not_found("module.id_invalid")))
}

/// Problem details of an error: kind, code, parameters, fields and the request id; internal errors expose
/// only the request id.
#[must_use]
pub fn problem(error: &Error) -> Response {
    let request_id = REQUEST_ID.try_with(Clone::clone).unwrap_or_default();
    let status =
        StatusCode::from_u16(error.kind().http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let internal = error.kind() == ErrorKind::Internal;
    if internal {
        tracing::error!(error = %error, request_id, "internal error");
    }
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

/// The version an `If-Match` header asks for, `"<version>"` or `W/"<version>"`: every update names the
/// version it was made on.
///
/// # Errors
///
/// `precondition_failed` (`module.if_match_required`) without the header, `validation` on `If-Match` when
/// it is not a version of Kampus.
pub fn if_match(headers: &HeaderMap) -> Result<i32, Error> {
    let Some(value) = headers.get(axum::http::header::IF_MATCH) else {
        return Err(Error::new(
            ErrorKind::PreconditionFailed,
            "module.if_match_required",
        ));
    };
    value
        .to_str()
        .ok()
        .map(|v| v.trim().trim_start_matches("W/").trim_matches('"'))
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| {
            Error::validation(
                "module.if_match_invalid",
                vec![FieldError::new("If-Match", "version")],
            )
        })
}

/// A response with the `ETag` of a version.
#[must_use]
pub fn with_etag(response: impl IntoResponse, version: i32) -> Response {
    let mut response = response.into_response();
    if let Ok(value) = HeaderValue::from_str(&format!("\"{version}\"")) {
        response.headers_mut().insert(axum::http::header::ETAG, value);
    }
    response
}
