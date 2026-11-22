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

//! The entities of files: a file, a named use of stored content in an area; an upload on its way to
//! becoming one; and a signed link to read one.

use kampus_access::ScopeId;
use kampus_common::Timestamp;
use serde::Serialize;

kampus_common::define_id! {
    /// Identifies a file.
    pub struct FileId;
}

kampus_common::define_id! {
    /// Identifies an upload.
    pub struct UploadId;
}

/// A file: content stored once, used under a name in an area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct File {
    /// The file.
    pub id: FileId,
    /// The area it belongs to.
    pub area: String,
    /// The scope it was uploaded for.
    pub scope: ScopeId,
    /// Its name.
    pub name: String,
    /// Its media type, read from its content.
    pub media_type: String,
    /// Its size in bytes.
    pub size: i64,
    /// Who uploaded it, while their account exists.
    pub uploader: Option<uuid::Uuid>,
    /// Whether a record owns it; until then only its uploader reads it.
    pub attached: bool,
    /// When it was uploaded.
    pub created_at: Timestamp,
}

/// An upload: permission to send one file into an area, for a while.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Upload {
    /// The upload.
    pub id: UploadId,
    /// The area.
    pub area: String,
    /// The scope it is for.
    pub scope: ScopeId,
    /// The name of the file.
    pub name: String,
    /// The largest content it accepts, in bytes.
    pub max_size: i64,
    /// Whether the content arrived.
    pub received: bool,
    /// The file it became, once completed.
    pub file: Option<FileId>,
    /// When it stops accepting content.
    pub expires_at: Timestamp,
}

/// A link that reads one file for a few minutes, without signing in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct SignedUrl {
    /// The link.
    pub url: String,
    /// When it stops working.
    pub expires_at: Timestamp,
}
