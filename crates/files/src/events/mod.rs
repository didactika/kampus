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

//! The events files publishes: a file uploaded, attached to its owner, and detached from it.

use kampus_module::EventDeclaration;

/// A file finished uploading and waits to be attached.
pub const UPLOADED: &str = "files.uploaded";

/// A module attached a file to the record that owns it.
pub const ATTACHED: &str = "files.attached";

/// A module detached a file from its owner, which deletes it.
pub const DETACHED: &str = "files.detached";

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: UPLOADED,
        version: 1,
        summary: "A file finished uploading into an area and waits to be attached",
        fields: &["file", "area", "size", "media_type"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: ATTACHED,
        version: 1,
        summary: "A file was attached to the record that owns it",
        fields: &["file", "area"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: DETACHED,
        version: 1,
        summary: "A file was detached from its owner and deleted",
        fields: &["file", "area"],
        audited: false,
        personal_fields: &[],
    },
];
