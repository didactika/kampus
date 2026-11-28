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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Settings of Kampus: modules declare typed settings with a default, the levels of the scope tree where
//! each may be set and the permission that sets it; values are kept per scope and resolved from the nearest
//! scope that sets them, unless a scope above locks them, and every change is recorded and published.

pub mod commands;
pub mod db;
pub mod events;
pub mod http;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;

use kampus_access::Access;
use kampus_db::Database;

pub use model::{Definition, Origin, Resolved, SettingChange};
pub use module::MODULE;
pub use service::scope::ScopeSetting;

/// The settings of an installation.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The database of the installation.
    db: Database,
    /// Access, for the scope tree and the permission of each setting.
    access: Access,
}

impl Settings {
    /// The settings of an installation.
    #[must_use]
    pub const fn new(db: Database, access: Access) -> Self {
        Self { db, access }
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
