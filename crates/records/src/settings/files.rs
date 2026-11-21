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

//! The file area records keeps: the exports of grades, readable only by the person who asked for them while
//! they have not expired.

use kampus_module::{FileAreaDeclaration, FileRead, ReadFuture};

use crate::Records;
use crate::permissions::EXPORT;
use crate::service::export::EXPORT_AREA;

/// The largest export, in bytes.
const MAX_EXPORT: u64 = 256 * 1024 * 1024;

/// The file areas of this module.
pub const AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: EXPORT_AREA,
    summary: "An export of the grades of a course",
    max_size: MAX_EXPORT,
    allowed_types: &["text/csv"],
    upload_permission: EXPORT,
    readable,
}];

/// Whether someone may read an export: the person who asked for it, until it expires.
fn readable(read: FileRead<'_>) -> ReadFuture<'_> {
    Box::pin(async move {
        read.services
            .service::<Records>()?
            .export_readable(read.file, read.principal)
            .await
    })
}
