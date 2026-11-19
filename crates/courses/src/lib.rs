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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Courses of Kampus: the tree of categories, courses with their lifecycle, nested sections ordered by
//! fractional keys, and groups and group sets. Every category, course and section is a scope of access,
//! and visibility is each node's own: what a learner sees is computed from the node and its ancestors.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod permissions;
pub mod ports;
pub mod service;
pub mod settings;
pub mod support;

use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{Clock, Timestamp};
use kampus_db::Database;

pub use model::{
    Category, CategoryFields, CategoryId, Course, CourseFields, CourseId, CourseState, Group, GroupFields,
    GroupId, GroupMode, GroupSet, GroupSetId, GroupVisibility, Member, MemberOrigin, Place, Section,
    SectionFields, SectionId, Visibility,
};
pub use module::MODULE;
pub use ports::participants::{CampusMembers, Participants};
pub use service::reference::RecordKind;

/// How long a scheduled deletion of a course waits.
pub const DELETION_GRACE: time::Duration = time::Duration::days(14);

/// The layout of a course that names none.
pub const DEFAULT_LAYOUT: &str = "topics";

/// The courses service.
#[derive(Clone)]
pub struct Courses {
    /// The database of the installation.
    db: Database,
    /// Access, for checks and scopes.
    access: Access,
    /// Where "now" comes from.
    clock: Arc<dyn Clock>,
    /// Who takes part in each course.
    participants: Arc<dyn Participants>,
    /// Customisation, for custom fields and features; without it every feature is on.
    customise: Option<kampus_customise::Customise>,
}

impl std::fmt::Debug for Courses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Courses")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Courses {
    /// Creates the service.
    #[must_use]
    pub fn new(
        db: Database,
        access: Access,
        clock: Arc<dyn Clock>,
        participants: Arc<dyn Participants>,
    ) -> Self {
        Self {
            db,
            access,
            clock,
            participants,
            customise: None,
        }
    }

    /// The service with customisation, which gives courses custom fields and lets scopes turn features
    /// off.
    #[must_use]
    pub fn with_customise(self, customise: kampus_customise::Customise) -> Self {
        Self {
            customise: Some(customise),
            ..self
        }
    }

    /// Fails unless a feature of this module is on in a scope.
    async fn require_feature(&self, code: &str, scope: kampus_access::ScopeId) -> kampus_common::Result<()> {
        match &self.customise {
            Some(customise) => customise.require_flag(code, scope).await,
            None => Ok(()),
        }
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
