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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events courses publishes.

use kampus_module::EventDeclaration;

/// A course was created.
pub const COURSE_CREATED: &str = "courses.course_created";
/// A course moved to another state of its lifecycle.
pub const COURSE_STATE_CHANGED: &str = "courses.course_state_changed";
/// A course was deleted with everything it owns.
pub const COURSE_DELETED: &str = "courses.course_deleted";

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: COURSE_CREATED,
        version: 1,
        summary: "A course was created",
        fields: &["course", "category"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: COURSE_STATE_CHANGED,
        version: 1,
        summary: "A course moved to another state of its lifecycle",
        fields: &["course", "from", "to"],
        audited: true,
        personal_fields: &[],
    },
    EventDeclaration {
        code: COURSE_DELETED,
        version: 1,
        summary: "A course was deleted with everything it owns",
        fields: &["course"],
        audited: true,
        personal_fields: &[],
    },
];
