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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Enrolment of Kampus: people take part in a course through enrolments that method instances of the
//! course create, by hand, by themselves with a key or an approval, or from a system group. An
//! enrolment is participation; what the person may do comes from the grants it owns in access.

pub mod commands;
pub mod db;
pub mod http;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;

use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{Clock, Timestamp};
use kampus_courses::Courses;
use kampus_db::Database;
use kampus_identity::{Identity, PasswordHasher};
use kampus_users::Users;

pub use model::{
    Enrolment, EnrolmentChange, EnrolmentId, Expired, ExpiryAction, Instance, InstanceFields, InstanceId,
    Joined, Method, Period, Request, RequestId, RequestState, Status, Synced,
};
pub use module::MODULE;
pub use service::participation::Participation;

/// The enrolment service.
#[derive(Clone)]
pub struct Enrol {
    /// The database of the installation.
    db: Database,
    /// Access, for checks and the grants enrolments own.
    access: Access,
    /// Courses, for their scopes and groups.
    courses: Courses,
    /// Identity, for the campus of an account.
    identity: Identity,
    /// Users, for the members of system groups.
    users: Users,
    /// Where "now" comes from.
    clock: Arc<dyn Clock>,
    /// Hashes self-enrolment keys.
    hasher: PasswordHasher,
}

impl std::fmt::Debug for Enrol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enrol")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Enrol {
    /// Creates the service.
    #[must_use]
    pub fn new(
        db: Database,
        access: Access,
        courses: Courses,
        identity: Identity,
        users: Users,
        clock: Arc<dyn Clock>,
        hasher: PasswordHasher,
    ) -> Self {
        Self {
            db,
            access,
            courses,
            identity,
            users,
            clock,
            hasher,
        }
    }

    /// The quoted name of a table of this module.
    fn table(&self, name: &str) -> String {
        table(&self.db, name)
    }

    /// The current instant.
    fn now(&self) -> Timestamp {
        self.clock.now()
    }
}

/// The quoted name of a table of this module in an installation.
fn table(db: &Database, name: &str) -> String {
    format!("{}.\"{name}\"", db.prefix().schema(module::NAME))
}

/// Wraps SQL built by this module as audited: only quoted, prefixed table names are interpolated, and
/// every value is bound.
fn sql(text: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(text)
}
