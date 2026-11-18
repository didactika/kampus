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

/// A transaction: every service operation is one (DEC-016).
pub type Transaction = sqlx::Transaction<'static, sqlx::Postgres>;

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
    /// # Errors
    ///
    /// An `unavailable` error when the database cannot be reached.
    pub async fn connect(url: &str, prefix: Prefix, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size.max(1))
            .acquire_timeout(Duration::from_secs(10))
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
