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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Events of Kampus: every significant change publishes a declared event, written to the outbox in the
//! change's transaction and delivered after commit to the listeners of every module, at least once, in
//! order for each object, with retries and a dead-letter state; audited events keep an append-only entry
//! in the audit log.

pub mod commands;
pub mod db;
pub mod events;
pub mod jobs;
pub mod module;
pub mod permissions;
pub mod service;

use kampus_db::Database;

pub use events::ANY;
pub use module::MODULE;
pub use service::audit::{AuditEntry, Purged};
pub use service::delivery::Delivery;
pub use service::publish::{MAX_DEPTH, Publish};
pub use service::runtime::Dispatcher;

/// The events of an installation.
#[derive(Debug, Clone)]
pub struct Events {
    /// The database of the installation.
    db: Database,
}

impl Events {
    /// The events of an installation.
    #[must_use]
    pub const fn new(db: Database) -> Self {
        Self { db }
    }

    /// The quoted name of a table of this module.
    fn table(&self, name: &str) -> String {
        format!("{}.\"{name}\"", self.db.prefix().schema(module::NAME))
    }
}

/// Wraps SQL built by this module as audited: only quoted, prefixed table names are interpolated, and
/// every value is bound.
fn sql(text: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(text)
}
