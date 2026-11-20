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

//! The events activities publishes: course modules created, changed, moved, deleted to the recycle bin,
//! restored from it, and removed for good.

use kampus_module::EventDeclaration;

/// A course module was added to a section.
pub const MODULE_CREATED: &str = "activities.module_created";

/// A course module changed.
pub const MODULE_UPDATED: &str = "activities.module_updated";

/// A course module moved to another place.
pub const MODULE_MOVED: &str = "activities.module_moved";

/// A course module went to the recycle bin.
pub const MODULE_DELETED: &str = "activities.module_deleted";

/// A course module came back from the recycle bin.
pub const MODULE_RESTORED: &str = "activities.module_restored";

/// A course module was removed for good when its time in the bin ended.
pub const MODULE_PURGED: &str = "activities.module_purged";

/// An exception of a course module was granted, changed or removed.
pub const EXCEPTION_CHANGED: &str = "activities.exception_changed";

/// A declaration of one of the events of a course module.
const fn declared(code: &'static str, summary: &'static str) -> EventDeclaration {
    EventDeclaration {
        code,
        version: 1,
        summary,
        fields: &["module", "section", "type"],
        audited: false,
        personal_fields: &[],
    }
}

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    declared(MODULE_CREATED, "A course module was added to a section"),
    declared(MODULE_UPDATED, "A course module changed"),
    declared(MODULE_MOVED, "A course module moved to another place"),
    declared(MODULE_DELETED, "A course module went to the recycle bin"),
    declared(MODULE_RESTORED, "A course module came back from the recycle bin"),
    declared(MODULE_PURGED, "A course module was removed for good"),
    declared(
        EXCEPTION_CHANGED,
        "An exception of a course module was granted, changed or removed",
    ),
];
