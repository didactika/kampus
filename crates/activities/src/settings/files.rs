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

//! The places where activities keeps files: the file of a material.

use kampus_access::{Actor, PrincipalId};
use kampus_module::{FileAreaDeclaration, FileRead, ReadFuture};

use crate::Activities;
use crate::permissions::MODULE_MANAGE;

/// The area of the files of materials.
pub const MATERIAL_AREA: &str = "activities.material";

/// The file areas of this module.
pub const FILE_AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: MATERIAL_AREA,
    summary: "The file of a material",
    max_size: 100 * 1024 * 1024,
    allowed_types: &["*/*"],
    upload_permission: MODULE_MANAGE,
    readable,
}];

/// A file of a material is read by whoever sees the course module that owns it.
fn readable(read: FileRead<'_>) -> ReadFuture<'_> {
    Box::pin(async move {
        let Some(principal) = read.principal else {
            return Ok(false);
        };
        let activities = read.services.service::<Activities>()?;
        activities
            .file_readable(&Actor::Principal(PrincipalId::from_uuid(principal)), read.file)
            .await
    })
}
