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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Records of Kampus: the grade records of a course, organised in categories, and the grades people get on
//! them. Every change to a grade is an immutable entry; the current grade is derived from the entries by
//! idempotent upserts, and everything a person may see is decided in one place.

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
    Band, Category, CategoryFields, CategoryId, Entry, EntryId, Grade, GradeChange, GradeType, GradeWrite,
    LetterSet, LetterSetId, Method, Record, RecordFields, RecordId, RecordKind, Scale, ScaleId, Source, Tree,
};
pub use module::MODULE;
pub use service::grade::{Detail, Report, ReportRow};
pub use service::recalc::Recalculated;
pub use service::record::ActivityGrade;
pub use support::{aggregation, letters};

/// The records of an installation.
#[derive(Clone)]
pub struct Records {
    /// The database of the installation.
    db: Database,
    /// Access, for checks.
    access: Access,
    /// Courses, for the course a record belongs to and who sees it.
    courses: Courses,
    /// Activities, for the course of a course module.
    activities: Activities,
}

impl std::fmt::Debug for Records {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Records")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Records {
    /// The records of an installation.
    #[must_use]
    pub const fn new(db: Database, access: Access, courses: Courses, activities: Activities) -> Self {
        Self {
            db,
            access,
            courses,
            activities,
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
