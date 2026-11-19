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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Customisation of Kampus: what a campus changes without code, kept as data per scope and resolved from
//! the nearest scope up, with locks: the words of the domain, feature flags, and typed custom fields of the
//! entities modules declare.

pub mod db;
pub mod events;
pub mod http;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;

use kampus_access::Access;
use kampus_db::Database;

pub use model::{
    Choice, Field, FieldEntry, FieldId, FieldSpec, FieldType, FieldValueId, FieldVisibility, Flag, Origin,
    Term,
};
pub use module::MODULE;

/// The customisation of an installation.
#[derive(Debug, Clone)]
pub struct Customise {
    /// The database of the installation.
    db: Database,
    /// Access, for the scope tree and the checks.
    access: Access,
}

impl Customise {
    /// The customisation of an installation.
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
