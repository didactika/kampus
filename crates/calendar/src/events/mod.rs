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
// @package   kampus-calendar
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events calendar publishes: entries created, changed and deleted.

use kampus_module::EventDeclaration;

/// An entry was created.
pub const ENTRY_CREATED: &str = "calendar.entry_created";

/// An entry changed.
pub const ENTRY_UPDATED: &str = "calendar.entry_updated";

/// An entry was deleted.
pub const ENTRY_DELETED: &str = "calendar.entry_deleted";

/// A declaration of an event about an entry.
const fn declared(code: &'static str, summary: &'static str) -> EventDeclaration {
    EventDeclaration {
        code,
        version: 1,
        summary,
        fields: &["entry"],
        audited: false,
        personal_fields: &[],
    }
}

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    declared(ENTRY_CREATED, "A calendar entry was created"),
    declared(ENTRY_UPDATED, "A calendar entry changed"),
    declared(ENTRY_DELETED, "A calendar entry was deleted"),
];
