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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Access control of Kampus: the scope tree with sections, principals,
//! roles made from templates, grants, overrides, permission checks with their explanation, and "view as".
//!
//! There is no bypass: administrators are principals with roles, checked like everyone else.

pub mod db;
pub mod http;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;

use std::sync::Arc;

use kampus_common::{Clock, Timestamp};
use kampus_db::Database;

pub use model::{
    Acting, Actor, AllowanceKind, Carried, GrantId, PermissionDeclaration, PermissionKind, PrincipalId, Risk,
    RoleId, RoleTemplateDeclaration, ScopeId, ScopeKind, Value, ViewAsId,
};
pub use module::MODULE;
pub use service::check::{Decision, GrantEvaluation};
pub use service::review::{GrantView, OverrideView, PermissionInfo, RoleView};
pub use service::scope::PathScope;

/// The access service.
#[derive(Clone)]
pub struct Access {
    /// The database of the installation.
    db: Database,
    /// Where "now" comes from.
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Access")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Access {
    /// Creates the service on an installation's database.
    #[must_use]
    pub fn new(db: Database, clock: Arc<dyn Clock>) -> Self {
        Self { db, clock }
    }

    /// The quoted name of a table of this module.
    fn table(&self, name: &str) -> String {
        format!("{}.\"{name}\"", self.db.prefix().schema(module::NAME))
    }

    /// The current instant.
    fn now(&self) -> Timestamp {
        self.clock.now()
    }
}

/// Wraps SQL built by this module as audited: only quoted, prefixed table names are interpolated, and
/// every value is bound.
fn sql(text: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(text)
}
