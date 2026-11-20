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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a module offers to files: the areas where it keeps files, each with its limits, the permission to
//! upload into it, and how to tell whether someone may read one of its files, which is when they may read
//! the record that owns it.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use kampus_common::Result;

use crate::ServiceContext;

/// Whether someone may read a file.
pub type ReadFuture<'r> = Pin<Box<dyn Future<Output = Result<bool>> + Send + 'r>>;

/// A question to the module that owns a file: may this person read it?
#[derive(Debug, Clone, Copy)]
pub struct FileRead<'r> {
    /// The file.
    pub file: uuid::Uuid,
    /// The principal asking; none for a visitor.
    pub principal: Option<uuid::Uuid>,
    /// The services of the process.
    pub services: &'r ServiceContext,
}

/// A place where a module keeps files.
#[derive(Clone, Copy)]
pub struct FileAreaDeclaration {
    /// `<module>.<area>`, unique in the build.
    pub code: &'static str,
    /// What it holds.
    pub summary: &'static str,
    /// The largest file, in bytes.
    pub max_size: u64,
    /// The media types it accepts, such as `image/png`, or a family such as `image/*`.
    pub allowed_types: &'static [&'static str],
    /// The permission to upload into it, in the scope the upload is for.
    pub upload_permission: &'static str,
    /// Whether someone may read one of its files: when they may read the record that owns it.
    pub readable: for<'r> fn(FileRead<'r>) -> ReadFuture<'r>,
}

impl fmt::Debug for FileAreaDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileAreaDeclaration")
            .field("code", &self.code)
            .field("max_size", &self.max_size)
            .finish_non_exhaustive()
    }
}
