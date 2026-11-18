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

//! The commands of the binary: serve a role, upgrade the installation, or check it.

use kampus_common::{Error, PLATFORM_VERSION, Result};
use kampus_db::{Database, UpgradePlan, Upgrader};
use tokio::net::TcpListener;

use crate::config::{Config, LogFormat};
use crate::http::{AppState, router};
use crate::modules::SCHEMAS;

/// Starts logging as the configuration asks: JSON lines by default, with the request id in each span.
pub fn init_logging(config: &Config) {
    let filter = tracing_subscriber::EnvFilter::try_new(&config.log_level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    let result = match config.log_format {
        LogFormat::Json => builder.json().with_current_span(true).try_init(),
        LogFormat::Text => builder.try_init(),
    };
    drop(result);
}

/// Connects to the database of the installation.
async fn connect(config: &Config) -> Result<Database> {
    Database::connect(
        &config.database_url,
        config.database_prefix.clone(),
        config.database_pool_size,
    )
    .await
}

/// Serves a role until the process receives a stop signal.
///
/// The database must match this build; when it does not, pending steps are applied if the configuration
/// allows it, and otherwise the process refuses to start.
///
/// # Errors
///
/// A `conflict` error when the database does not match this build, and an `unavailable` error when the
/// database or the listener cannot be reached.
pub async fn serve(config: &Config) -> Result<()> {
    let db = connect(config).await?;
    let upgrader = Upgrader::new(&db, &SCHEMAS, PLATFORM_VERSION);
    let plan = upgrader.plan().await?;
    if !plan.is_current() {
        if config.upgrade_on_start && plan.problems.is_empty() {
            let report = upgrader.apply(false).await?;
            tracing::info!(
                applied = report.applied.len(),
                installed = report.installed,
                "upgrade on start finished"
            );
        } else {
            return Err(not_current(&plan));
        }
    }
    crate::modules::sync_declarations(&db).await?;

    let listener = TcpListener::bind(config.http_address).await.map_err(|error| {
        Error::unavailable("server.bind")
            .with_param("address", config.http_address.to_string())
            .with_source(error)
    })?;
    tracing::info!(role = %config.role, address = %config.http_address, version = PLATFORM_VERSION, "kampus started");
    let ports = crate::ports::Ports::open(config).await?;
    let app = router(AppState {
        db: db.clone(),
        modules: SCHEMAS.as_slice(),
        ports,
    });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| Error::unavailable("server.http").with_source(error))?;
    db.close().await;
    tracing::info!("kampus stopped");
    Ok(())
}

/// Applies every pending upgrade step, from whichever release the installation is on.
///
/// # Errors
///
/// A `conflict` error when a problem stops the upgrade, and an `unavailable` error when a step fails.
pub async fn upgrade(config: &Config, new_installation: bool) -> Result<Vec<String>> {
    let db = connect(config).await?;
    let report = Upgrader::new(&db, &SCHEMAS, PLATFORM_VERSION)
        .apply(new_installation)
        .await?;
    db.close().await;
    Ok(report.applied.iter().map(ToString::to_string).collect())
}

/// Reports whether the installation matches this build, without changing anything.
///
/// # Errors
///
/// An `unavailable` error when the database cannot be read.
pub async fn check(config: &Config) -> Result<UpgradePlan> {
    let db = connect(config).await?;
    let plan = Upgrader::new(&db, &SCHEMAS, PLATFORM_VERSION).plan().await;
    db.close().await;
    plan
}

/// The error of a database that does not match this build.
fn not_current(plan: &UpgradePlan) -> Error {
    let mut reasons: Vec<String> = plan.problems.iter().map(ToString::to_string).collect();
    if !plan.installed {
        reasons.push("the installation does not exist: run `kampus-server upgrade`".to_owned());
    } else if !plan.pending.is_empty() {
        reasons.push(format!(
            "{} pending upgrade step(s): run `kampus-server upgrade`",
            plan.pending.len()
        ));
    }
    Error::conflict("server.schema_not_current").with_param("reasons", reasons.join("; "))
}

/// Waits for Ctrl+C or, on Unix, SIGTERM, so in-flight requests finish before the process exits.
async fn shutdown_signal() {
    let interrupt = async {
        drop(tokio::signal::ctrl_c().await);
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
    tracing::info!("stop signal received");
}
