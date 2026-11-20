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
//! module changed, and a teacher decided or removed a state.

use kampus_module::EventDeclaration;

/// The state of a person on a course module changed.
pub const ACTIVITY_CHANGED: &str = "progress.activity_changed";

/// The rules of a course module changed.
pub const RULES_CHANGED: &str = "progress.rules_changed";

/// A teacher decided the state of a person, or removed the decision.
pub const EXCEPTION_CHANGED: &str = "progress.exception_changed";

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
];
