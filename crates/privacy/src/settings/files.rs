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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The file area privacy keeps: the packages of exports, readable only by the person who asked for them
//! while the request has not expired.

use kampus_module::{FileAreaDeclaration, FileRead, ReadFuture};

use crate::permissions::REQUEST_MANAGE;
use crate::{PACKAGE_AREA, Privacy};

/// The largest package, in bytes.
const MAX_PACKAGE: u64 = 512 * 1024 * 1024;

/// The file areas of this module.
pub const AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: PACKAGE_AREA,
    summary: "The package of an export of personal data",
    max_size: MAX_PACKAGE,
    allowed_types: &["application/json"],
    upload_permission: REQUEST_MANAGE,
    readable,
}];

/// Whether someone may read a package: the person who asked for it, until it expires.
fn readable(read: FileRead<'_>) -> ReadFuture<'_> {
    Box::pin(async move {
        read.services
            .service::<Privacy>()?
            .package_readable(read.file, read.principal)
            .await
    })
}
