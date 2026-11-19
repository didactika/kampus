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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events customisation publishes: its changes, so the modules and nodes that keep a resolved value
//! refresh it, and the failures of rules.

use kampus_module::EventDeclaration;

/// A customisation changed in a scope.
pub const CHANGED: &str = "customise.changed";

/// A rule failed to run its actions for an event.
pub const RULE_FAILED: &str = "customise.rule_failed";

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: CHANGED,
        version: 1,
        summary: "A customisation changed in a scope",
        fields: &["kind", "object", "scope"],
        audited: true,
        personal_fields: &[],
    },
    EventDeclaration {
        code: RULE_FAILED,
        version: 1,
        summary: "A rule failed to run its actions for an event",
        fields: &["rule", "event", "error"],
        audited: true,
        personal_fields: &[],
    },
];
