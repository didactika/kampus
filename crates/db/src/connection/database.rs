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
// @package   kampus-db
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The connection pool of an installation, with its prefix, and the transactions service operations run in.

use std::time::Duration;

use kampus_common::{Error, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Prefix;

/// The longest wait for a connection of the pool before an operation fails as unavailable.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a connection may stay idle in the pool before it is closed.
const IDLE_TIMEOUT: Duration = Duration::from_mins(2);

/// How long a connection may stay open before it is replaced.
const MAX_LIFETIME: Duration = Duration::from_mins(30);

/// A transaction: every service operation is one.
pub type Transaction = sqlx::Transaction<'static, sqlx::Postgres>;

/// The optional features of a database adapter; modules ask for these, never for the dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "each flag is an independent capability"
)]
pub struct Capabilities {
    /// Row locks that skip locked rows (`FOR UPDATE SKIP LOCKED`), for job queues.
    pub skip_locked: bool,
    /// Inserts that resolve conflicts on a key (`ON CONFLICT`).
    pub upsert: bool,
    /// Locks held by a session or a transaction that are not rows (advisory locks).
    pub advisory_locks: bool,
    /// One schema per module; without it, tables carry the module in their name.
    pub schemas: bool,
}

/// Turns a database failure into the error model: violated keys and references are conflicts,
/// violated checks and missing values are validation errors naming the constraint or column, and
/// serialisation failures and deadlocks are unavailable, to be retried.
#[must_use]
pub fn classify(error: sqlx::Error) -> Error {
    let details = error.as_database_error().map(|db| {
        let code = db.code().map_or_else(String::new, std::borrow::Cow::into_owned);
        let constraint = db.constraint().unwrap_or_default().to_owned();
        let column = db
            .try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
            .and_then(|pg| pg.column())
            .unwrap_or_default()
            .to_owned();
        (code, constraint, column)
    });
    let Some((code, constraint, column)) = details else {
        return Error::unavailable("db.query").with_source(error);
    };
    match code.as_str() {
        "23505" => Error::conflict("db.unique_violation")
            .with_param("constraint", constraint)
            .with_source(error),
        "23503" => Error::conflict("db.foreign_key_violation")
            .with_param("constraint", constraint)
            .with_source(error),
        "23514" => Error::validation(
            "db.check_violation",
            vec![kampus_common::FieldError::new(constraint, "check")],
        )
        .with_source(error),
        "23502" => Error::validation(
            "db.not_null_violation",
            vec![kampus_common::FieldError::new(column, "required")],
        )
        .with_source(error),
        "40001" | "40P01" => Error::unavailable("db.retry").with_source(error),
        _ => Error::unavailable("db.query")
            .with_param("sqlstate", code)
            .with_source(error),
    }
}

/// The database of one installation.
#[derive(Debug, Clone)]
pub struct Database {
    /// The connection pool.
    pool: PgPool,
    /// The installation prefix of every physical name.
    prefix: Prefix,
}

impl Database {
    /// Connects to the database of an installation.
    ///
    /// The pool opens connections as they are needed, up to `pool_size`, and closes the ones left idle for
    /// `IDLE_TIMEOUT` or open for `MAX_LIFETIME`, so an idle process gives its connections back. Getting a
    /// connection waits at most `ACQUIRE_TIMEOUT`, retrying while the server has no free connection slot, so a
    /// database that is busy for a moment slows operations down instead of failing them.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the database cannot be reached.
    pub async fn connect(url: &str, prefix: Prefix, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size.max(1))
            .min_connections(0)
            .acquire_timeout(ACQUIRE_TIMEOUT)
            .idle_timeout(IDLE_TIMEOUT)
            .max_lifetime(MAX_LIFETIME)
            .connect(url)
            .await
            .map_err(|error| Error::unavailable("db.connect").with_source(error))?;
        Ok(Self { pool, prefix })
    }

    /// Wraps an existing pool, for tests and for adapters that build their own.
    #[must_use]
    pub const fn from_pool(pool: PgPool, prefix: Prefix) -> Self {
        Self { pool, prefix }
    }

    /// The connection pool.
    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// The installation prefix.
    #[must_use]
    pub const fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    /// The optional features of this database: PostgreSQL has them all.
    #[must_use]
    pub const fn capabilities(&self) -> Capabilities {
        Capabilities {
            skip_locked: true,
            upsert: true,
            advisory_locks: true,
            schemas: true,
        }
    }

    /// Starts a transaction.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when no connection can be obtained.
    pub async fn begin(&self) -> Result<Transaction> {
        self.pool
            .begin()
            .await
            .map_err(|error| Error::unavailable("db.begin").with_source(error))
    }

    /// Checks that the database answers.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when it does not.
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("select 1")
            .execute(&self.pool)
            .await
            .map(drop)
            .map_err(|error| Error::unavailable("db.ping").with_source(error))
    }

    /// Closes every connection, at shutdown.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}
