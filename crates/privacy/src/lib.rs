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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Privacy of Kampus: people ask for an export of the personal data of an account, and every module answers
//! for its own data through the handler it declares, so privacy never depends on the modules it serves. A
//! request is split into one part per data category, answered by a job; the package is available to the
//! person who asked for a limited time. A campus also publishes its policies in versions, and people accept
//! the version in force.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;

use kampus_access::Access;
use kampus_db::Database;
use kampus_module::Module as Declared;

pub use model::{Hold, HoldId, Item, Policy, PolicyId, Request, RequestId, Retention};
pub use module::MODULE;

/// How long an exported package stays available.
pub const PACKAGE_LIFETIME: time::Duration = time::Duration::days(7);

/// How long a campus has to answer a request.
pub const ANSWER_WITHIN: time::Duration = time::Duration::days(30);

/// The privacy requests of an installation.
#[derive(Clone)]
pub struct Privacy {
    /// The database of the installation.
    db: Database,
    /// Access, for who may handle the requests of others.
    access: Access,
    /// The modules of the build, whose handlers answer.
    modules: &'static [&'static Declared],
}

impl std::fmt::Debug for Privacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Privacy")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Privacy {
    /// The privacy requests of an installation, which know no handlers until they know the modules.
    #[must_use]
    pub const fn new(db: Database, access: Access) -> Self {
        Self {
            db,
            access,
            modules: &[],
        }
    }

    /// The same with the modules of the build, whose handlers answer.
    #[must_use]
    pub fn with_modules(self, modules: &'static [&'static Declared]) -> Self {
        Self { modules, ..self }
    }

    /// The quoted name of the principals of access, to find the campus of an account.
    fn principals(&self) -> String {
        format!("{}.\"principal\"", self.db.prefix().schema("access"))
    }

    /// The quoted name of the accounts of identity, to find the ones deleted.
    fn accounts(&self) -> String {
        format!("{}.\"account\"", self.db.prefix().schema("identity"))
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
