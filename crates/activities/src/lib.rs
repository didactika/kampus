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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Activities of Kampus: the course modules, activities and materials, that people find in the sections of
//! a course. Modules provide types; this module places course modules of them, in one transaction with
//! the data their type keeps, gives each a scope of its own under its section, and keeps what was deleted
//! in a recycle bin until it expires.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;
pub mod settings;

use kampus_access::Access;
use kampus_courses::Courses;
use kampus_db::Database;
use kampus_files::Files;
use kampus_module::{ActivityTypeDeclaration, Module as Declared};

pub use model::{ActivityType, BinItem, Module, ModuleFields, ModuleId, ModuleState};
pub use module::MODULE;
pub use service::exception::{Exception, ExceptionId, Target, Values};

/// How long a course module stays in the recycle bin.
pub const BIN_RETENTION: time::Duration = time::Duration::days(30);

/// The activities of an installation.
#[derive(Clone)]
pub struct Activities {
    /// The database of the installation.
    db: Database,
    /// Access, for scopes and checks.
    access: Access,
    /// Courses, for sections and what people see.
    courses: Courses,
    /// Files, for the files of materials.
    files: Files,
    /// The modules of the build, for the types they provide.
    modules: &'static [&'static Declared],
}

impl std::fmt::Debug for Activities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Activities")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Activities {
    /// The activities of an installation, which know no types until they know the modules.
    #[must_use]
    pub const fn new(db: Database, access: Access, courses: Courses, files: Files) -> Self {
        Self {
            db,
            access,
            courses,
            files,
            modules: &[],
        }
    }

    /// The activities with the modules of the build, whose types they register.
    #[must_use]
    pub fn with_modules(self, modules: &'static [&'static Declared]) -> Self {
        Self { modules, ..self }
    }

    /// The type a module of the build declares with a code.
    fn type_declaration(&self, code: &str) -> Option<&'static ActivityTypeDeclaration> {
        self.modules
            .iter()
            .flat_map(|m| m.activity_types.iter())
            .find(|t| t.code == code)
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
