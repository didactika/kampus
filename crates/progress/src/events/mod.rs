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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events progress publishes: the state of a person on a course module changed, the rules of a course
//! module changed, a teacher decided or removed a state, a course was completed or its completion revoked,
//! what a course requires or requires first changed, and someone of a role approved a person.

use kampus_module::EventDeclaration;

/// The state of a person on a course module changed.
pub const ACTIVITY_CHANGED: &str = "progress.activity_changed";

/// The rules of a course module changed.
pub const RULES_CHANGED: &str = "progress.rules_changed";

/// A teacher decided the state of a person, or removed the decision.
pub const EXCEPTION_CHANGED: &str = "progress.exception_changed";

/// A person completed a course.
pub const COURSE_COMPLETED: &str = "progress.course_completed";

/// The completion of a course by a person was revoked, because a criterion stopped being met.
pub const COURSE_REVOKED: &str = "progress.course_revoked";

/// What a course requires changed.
pub const CRITERIA_CHANGED: &str = "progress.criteria_changed";

/// Someone of a role approved a person for a criterion.
pub const APPROVED: &str = "progress.approved";

/// The courses a course requires first changed.
pub const PREREQUISITES_CHANGED: &str = "progress.prerequisites_changed";

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: ACTIVITY_CHANGED,
        version: 1,
        summary: "The state of a person on a course module changed",
        fields: &["module", "course", "account", "state"],
        audited: false,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: RULES_CHANGED,
        version: 1,
        summary: "The rules of a course module changed",
        fields: &["module", "course"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: EXCEPTION_CHANGED,
        version: 1,
        summary: "A teacher decided the state of a person on a course module, or removed the decision",
        fields: &["module", "course", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: COURSE_COMPLETED,
        version: 1,
        summary: "A person completed a course",
        fields: &["course", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: COURSE_REVOKED,
        version: 1,
        summary: "The completion of a course by a person was revoked",
        fields: &["course", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: CRITERIA_CHANGED,
        version: 1,
        summary: "What a course requires changed",
        fields: &["course"],
        audited: true,
        personal_fields: &[],
    },
    EventDeclaration {
        code: APPROVED,
        version: 1,
        summary: "Someone of a role approved a person for a criterion of a course",
        fields: &["course", "criterion", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: PREREQUISITES_CHANGED,
        version: 1,
        summary: "The courses a course requires first changed",
        fields: &["course"],
        audited: true,
        personal_fields: &[],
    },
];
