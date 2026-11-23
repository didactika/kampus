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

//! Metrics of every role, in the Prometheus text format at `/metrics`: the HTTP requests by method, route template
//! and status with their duration, the jobs by type and outcome with their duration (recorded by the task runner),
//! the connections of the database pool and the version of the build. Routes are labelled by their template, never
//! by the path asked, so the series stay few however many records there are.

use std::sync::OnceLock;
use std::time::Instant;

use axum::extract::{MatchedPath, Request, State};
use axum::http::header::CONTENT_TYPE;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

use super::AppState;

/// The bounds of the duration histograms, in seconds: from 5 ms to 10 s.
const BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.2, 0.3, 0.5, 1.0, 2.5, 5.0, 10.0];

/// The recorder of the process, installed once.
static HANDLE: OnceLock<Option<PrometheusHandle>> = OnceLock::new();

/// The recorder of the process; `None` when another recorder was installed first.
fn handle() -> Option<&'static PrometheusHandle> {
    HANDLE
        .get_or_init(|| {
            let recorder = PrometheusBuilder::new()
                .set_buckets_for_metric(Matcher::Suffix("_duration_seconds".to_owned()), BUCKETS)
                .ok()?
                .build_recorder();
            let handle = recorder.handle();
            metrics::set_global_recorder(recorder).ok()?;
            metrics::gauge!("kampus_build_info", "version" => kampus_common::PLATFORM_VERSION).set(1.0);
            Some(handle)
        })
        .as_ref()
}

/// Installs the recorder of the process, so the metrics recorded from now on are kept.
pub fn install() {
    let _ = handle();
}

/// Records the method, route template, status and duration of every request.
pub async fn measure(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "unmatched".to_owned(), |path| path.as_str().to_owned());
    let response = next.run(request).await;
    let status = response.status().as_u16().to_string();
    metrics::counter!("kampus_http_requests_total", "method" => method.clone(), "route" => route.clone(), "status" => status)
        .increment(1);
    metrics::histogram!("kampus_http_request_duration_seconds", "method" => method, "route" => route)
        .record(started.elapsed().as_secs_f64());
    response
}

/// The metrics of the process, with the state of the database pool read now.
pub async fn render(State(state): State<AppState>) -> Response {
    let pool = state.db.pool();
    metrics::gauge!("kampus_db_connections").set(f64::from(pool.size()));
    metrics::gauge!("kampus_db_connections_idle")
        .set(f64::from(u32::try_from(pool.num_idle()).unwrap_or(u32::MAX)));
    let body = handle().map_or_else(String::new, PrometheusHandle::render);
    ([(CONTENT_TYPE, "text/plain; version=0.0.4")], body).into_response()
}
