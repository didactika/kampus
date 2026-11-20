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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Files of Kampus: content stored once by its SHA-256 in the storage port, used as named files that
//! belong to records of other modules through their own link tables. Every upload goes through one
//! pipeline, the media type is read from the content, a file is readable exactly when its owner is, and
//! it is served through signed links of a few minutes.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod permissions;
pub mod service;
pub mod support;

use std::sync::Arc;

use kampus_access::Access;
use kampus_db::Database;
use kampus_module::{FileAreaDeclaration, Module};
use kampus_storage::Storage;

pub use model::{File, FileId, SignedUrl, Upload, UploadId};
pub use module::MODULE;

/// How long an upload accepts content.
pub const UPLOAD_LIFETIME: time::Duration = time::Duration::hours(1);

/// How long a signed link reads a file.
pub const LINK_LIFETIME: time::Duration = time::Duration::minutes(5);

/// How long a blob no file uses stays before it is deleted from storage.
pub const TRASH_RETENTION: time::Duration = time::Duration::days(7);

/// How long a file nobody attached stays before it is deleted.
pub const UNATTACHED_RETENTION: time::Duration = time::Duration::days(1);

/// The files of an installation.
#[derive(Clone)]
pub struct Files {
    /// The database of the installation.
    db: Database,
    /// Access, for the checks of uploads.
    access: Access,
    /// Where content is stored; none in roles that do not serve files.
    storage: Option<Arc<dyn Storage>>,
    /// The key that signs links.
    signing_key: [u8; 32],
    /// The address of the API, which links point to.
    api_url: String,
    /// The modules of the build, for their areas.
    modules: &'static [&'static Module],
}

impl std::fmt::Debug for Files {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Files")
            .field("db", &self.db)
            .field("api_url", &self.api_url)
            .finish_non_exhaustive()
    }
}

impl Files {
    /// The files of an installation.
    #[must_use]
    pub fn new(
        db: Database,
        access: Access,
        storage: Option<Arc<dyn Storage>>,
        signing_key: [u8; 32],
        api_url: String,
    ) -> Self {
        Self {
            db,
            access,
            storage,
            signing_key,
            api_url,
            modules: &[],
        }
    }

    /// The files with the modules of the build, whose areas they read.
    #[must_use]
    pub fn with_modules(self, modules: &'static [&'static Module]) -> Self {
        Self { modules, ..self }
    }

    /// The area a module of the build declares with a code.
    fn area_declaration(&self, code: &str) -> Option<&'static FileAreaDeclaration> {
        self.modules
            .iter()
            .flat_map(|m| m.file_areas.iter())
            .find(|a| a.code == code)
    }

    /// The storage, or an error in a process without one.
    fn storage(&self) -> kampus_common::Result<&Arc<dyn Storage>> {
        self.storage
            .as_ref()
            .ok_or_else(|| kampus_common::Error::unavailable("files.storage_missing"))
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
