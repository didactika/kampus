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

//! The types of material activities provides itself: a page of text, and a file.

use kampus_module::{ActivityKind, ActivitySetting, ActivityTypeDeclaration, ValueKind};

/// The type of a page of text.
pub const PAGE: &str = "activities.page";

/// The type of a downloadable file.
pub const FILE: &str = "activities.file";

/// The types of this module.
pub const TYPES: &[ActivityTypeDeclaration] = &[
    ActivityTypeDeclaration {
        code: PAGE,
        kind: ActivityKind::Material,
        summary: "A page of text",
        features: &["copy"],
        settings: &[ActivitySetting {
            key: "content",
            kind: ValueKind::Text {
                max: 20_000,
                default: "",
            },
        }],
    },
    ActivityTypeDeclaration {
        code: FILE,
        kind: ActivityKind::Material,
        summary: "A file to download",
        features: &["copy", "files"],
        settings: &[],
    },
];
