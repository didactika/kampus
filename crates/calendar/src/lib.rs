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
// @package   kampus-calendar
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Calendar of Kampus: the entries people create for a campus, a category, a course or themselves, and the
//! calendar of a course, which also shows the dates of its activities. Those dates are never copied: they
//! are read from the activities each time, with the exceptions of the person who asks applied.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod permissions;
pub mod privacy;
pub mod service;
pub mod settings;
pub mod support;

use kampus_access::Access;
use kampus_activities::Activities;
use kampus_courses::Courses;
use kampus_db::Database;

pub use model::{
    Entry, EntryFields, EntryId, Feed, FeedId, Item, ItemKind, OccurrenceChange, Subscription, SubscriptionId,
};
pub use module::MODULE;

/// The calendar of an installation.
#[derive(Clone)]
pub struct Calendar {
    /// The database of the installation.
    db: Database,
    /// Access, for checks.
    access: Access,
    /// Courses, for who sees a course.
    courses: Courses,
    /// Activities, for the dates of course modules.
    activities: Activities,
    /// Whether subscriptions may read calendars at private network addresses, which only tests allow.
    private_addresses: bool,
}

impl std::fmt::Debug for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Calendar")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Calendar {
    /// The calendar of an installation.
    #[must_use]
    pub const fn new(db: Database, access: Access, courses: Courses, activities: Activities) -> Self {
        Self {
            db,
            access,
            courses,
            activities,
            private_addresses: false,
        }
    }

    /// The same allowing subscriptions to calendars at private network addresses, for tests that serve one
    /// themselves.
    #[must_use]
    pub fn allowing_private_addresses(self) -> Self {
        Self {
            private_addresses: true,
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
