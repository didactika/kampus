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

//! The HTTP layer against PostgreSQL: liveness, readiness before and after the upgrade, request ids and
//! problem details.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a failing setup must stop the test"
)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use kampus_common::PLATFORM_VERSION;
use kampus_db::{Database, Prefix, Upgrader};
use kampus_server::http::{AppState, router};
use kampus_server::wiring::modules::SCHEMAS;
use kampus_server::wiring::ports::Ports;
use tower::ServiceExt;

/// Connects to a fresh installation with a prefix of its own.
async fn installation() -> Database {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("h{}_", &simple[simple.len() - 6..]).parse().unwrap();
    Database::connect(&url, prefix, 2).await.unwrap()
}

/// Sends a request and returns the status, the headers and the JSON body.
async fn call(
    db: &Database,
    uri: &str,
    request_id: Option<&str>,
) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
    let mut request = Request::get(uri);
    if let Some(id) = request_id {
        request = request.header("x-request-id", id);
    }
    let files = tempfile::tempdir().unwrap();
    let ports = Ports::for_tests(files.path().to_str().unwrap()).await.unwrap();
    let response = router(AppState {
        db: db.clone(),
        modules: SCHEMAS.as_slice(),
        ports,
    })
    .oneshot(request.body(Body::empty()).unwrap())
    .await
    .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, headers, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn liveness_answers_with_the_version() {
    let db = installation().await;
    let (status, headers, body) = call(&db, "/health/live", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["version"], PLATFORM_VERSION);
    assert!(headers.contains_key("x-request-id"));
}

#[tokio::test]
async fn readiness_waits_for_the_upgrade() {
    let db = installation().await;
    let (status, _, body) = call(&db, "/health/ready", None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], "not_ready");

    Upgrader::new(&db, &SCHEMAS, PLATFORM_VERSION)
        .apply(true)
        .await
        .unwrap();
    let (status, _, body) = call(&db, "/health/ready", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ready");

    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "drop schema if exists {} cascade",
        db.prefix().schema("db")
    )))
    .execute(db.pool())
    .await
    .unwrap();
}

#[tokio::test]
async fn unknown_routes_answer_problem_details_with_the_request_id() {
    let db = installation().await;
    let (status, headers, body) = call(&db, "/nowhere", Some("trace-42")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(headers["content-type"], "application/problem+json");
    assert_eq!(headers["x-request-id"], "trace-42");
    assert_eq!(body["code"], "server.route_not_found");
    assert_eq!(body["request_id"], "trace-42");
}
