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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Users of Kampus: the profile of the person behind each account, the preferences modules declare, and
//! system groups of a campus or a category, whose members are added by hand, by a rule or by a sync.

pub mod db;
pub mod events;
pub mod http;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;
pub mod settings;

use kampus_access::Access;
use kampus_db::Database;
use kampus_identity::Identity;
use kampus_module::PreferenceDeclaration;

pub use model::{
    GroupId, GroupKind, GroupRule, Member, Origin, Profile, ProfileFields, RuleField, SystemGroup,
};
pub use module::MODULE;

/// The users service.
#[derive(Clone)]
pub struct Users {
    /// The database of the installation.
    db: Database,
    /// Access, for checks and scopes.
    access: Access,
    /// Identity, for the campus of an account.
    identity: Identity,
    /// The preferences the modules of the build declare.
    preferences: Vec<PreferenceDeclaration>,
    /// Files, for profile pictures; without it there are none.
    files: Option<kampus_files::Files>,
}

impl std::fmt::Debug for Users {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Users")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Users {
    /// Creates the service with the preferences the modules of the build declare.
    #[must_use]
    pub fn new(
        db: Database,
        access: Access,
        identity: Identity,
        preferences: Vec<PreferenceDeclaration>,
    ) -> Self {
        Self {
            db,
            access,
            identity,
            preferences,
            files: None,
        }
    }

    /// The service with files, which keeps profile pictures.
    #[must_use]
    pub fn with_files(self, files: kampus_files::Files) -> Self {
        Self {
            files: Some(files),
            ..self
        }
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
