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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The places where users keeps files: the profile picture of each person, which whoever signed in sees.

use kampus_module::{FileAreaDeclaration, FileRead, ReadFuture};

use crate::permissions::PROFILE_UPDATE_OWN;

/// The area of profile pictures.
pub const PICTURE_AREA: &str = "users.picture";

/// The file areas of this module.
pub const FILE_AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: PICTURE_AREA,
    summary: "The profile picture of a person",
    max_size: 5 * 1024 * 1024,
    allowed_types: &["image/png", "image/jpeg", "image/gif", "image/webp"],
    upload_permission: PROFILE_UPDATE_OWN,
    readable: picture_readable,
}];

/// A profile picture is seen by whoever signed in, as their name is.
fn picture_readable(read: FileRead<'_>) -> ReadFuture<'_> {
    Box::pin(async move { Ok(read.principal.is_some()) })
}
