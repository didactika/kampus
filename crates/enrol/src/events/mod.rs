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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events enrolment publishes, so other modules react to enrolments without reading its tables.

use kampus_module::EventDeclaration;

/// Someone was enrolled in a course.
pub const ENROLMENT_CREATED: &str = "enrol.enrolment_created";
/// The status or period of an enrolment changed.
pub const ENROLMENT_UPDATED: &str = "enrol.enrolment_updated";
/// An enrolment was removed; records and progress stay.
pub const ENROLMENT_REMOVED: &str = "enrol.enrolment_removed";

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: ENROLMENT_CREATED,
        version: 1,
        summary: "Someone was enrolled in a course",
        fields: &["enrolment", "course", "account", "instance"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: ENROLMENT_UPDATED,
        version: 1,
        summary: "The status or period of an enrolment changed",
        fields: &["enrolment", "course", "account", "status"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: ENROLMENT_REMOVED,
        version: 1,
        summary: "An enrolment was removed",
        fields: &["enrolment", "course", "account"],
        audited: true,
        personal_fields: &[],
    },
];
