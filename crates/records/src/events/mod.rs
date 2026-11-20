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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events records publishes: grade records created, changed and deleted, grades changed, overridden
//! and locked.

use kampus_module::EventDeclaration;

/// A grade record was added to a course.
pub const RECORD_CREATED: &str = "records.record_created";

/// A grade record changed.
pub const RECORD_UPDATED: &str = "records.record_updated";

/// A grade record was deleted.
pub const RECORD_DELETED: &str = "records.record_deleted";

/// The grade of a person on a record changed.
pub const GRADE_CHANGED: &str = "records.grade_changed";

/// The grade of a person was overridden, or its override removed.
pub const OVERRIDDEN: &str = "records.overridden";

/// A record or a grade was locked or unlocked.
pub const LOCKED: &str = "records.locked";

/// A total of a person changed, because the grades it adds up changed.
pub const TOTAL_CHANGED: &str = "records.total_changed";

/// A declaration of an event about a grade record.
const fn about_record(code: &'static str, summary: &'static str) -> EventDeclaration {
    EventDeclaration {
        code,
        version: 1,
        summary,
        fields: &["record", "course"],
        audited: false,
        personal_fields: &[],
    }
}

/// A declaration of an event about the grade of a person.
const fn about_grade(code: &'static str, summary: &'static str, audited: bool) -> EventDeclaration {
    EventDeclaration {
        code,
        version: 1,
        summary,
        fields: &["record", "course", "account"],
        audited,
        personal_fields: &["account"],
    }
}

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    about_record(RECORD_CREATED, "A grade record was added to a course"),
    about_record(RECORD_UPDATED, "A grade record changed"),
    about_record(RECORD_DELETED, "A grade record was deleted"),
    about_grade(GRADE_CHANGED, "The grade of a person on a record changed", false),
    about_grade(
        OVERRIDDEN,
        "The grade of a person was overridden, or its override removed",
        true,
    ),
    about_grade(
        TOTAL_CHANGED,
        "A total of a person changed with the grades it adds up",
        false,
    ),
    EventDeclaration {
        code: LOCKED,
        version: 1,
        summary: "A record or a grade was locked or unlocked",
        fields: &["record", "course"],
        audited: true,
        personal_fields: &["account"],
    },
];
