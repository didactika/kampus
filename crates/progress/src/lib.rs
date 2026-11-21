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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Progress of Kampus: whether a person has done what a course module asks. The state of a person on a course
//! module is derived from the rules of the course module and what the person did, re-evaluated when something
//! relevant changes; what teachers decide with an exception is never overwritten by the rules.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod observers;
pub mod permissions;
pub mod privacy;
pub mod service;
pub mod settings;
pub mod support;

use kampus_access::Access;
use kampus_activities::Activities;
use kampus_courses::Courses;
use kampus_db::Database;
use kampus_records::Records;

pub use model::{
    ActivityState, CourseCriteria, CourseProgress, Criterion, CriterionState, Exception, RuleKind, Source,
    State,
};
pub use module::MODULE;
pub use service::prerequisite::{PrerequisiteGate, Prerequisites};
pub use support::evaluate;

/// The progress of an installation.
#[derive(Clone)]
pub struct Progress {
    /// The database of the installation.
    db: Database,
    /// Access, for checks.
    access: Access,
    /// Courses, for who sees a course.
    courses: Courses,
    /// Activities, for the course modules progress is about.
    activities: Activities,
    /// Records, for the grades that some rules depend on.
    records: Records,
    /// The modules of the build, whose activity types may declare rules of their own.
    modules: &'static [&'static kampus_module::Module],
}

impl std::fmt::Debug for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Progress")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Progress {
    /// The progress of an installation.
    #[must_use]
    pub const fn new(
        db: Database,
        access: Access,
        courses: Courses,
        activities: Activities,
        records: Records,
    ) -> Self {
        Self {
            db,
            access,
            courses,
            activities,
            records,
            modules: &[],
        }
    }

    /// The same with the modules of the build, whose activity types may declare rules of their own.
    #[must_use]
    pub fn with_modules(self, modules: &'static [&'static kampus_module::Module]) -> Self {
        Self { modules, ..self }
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
