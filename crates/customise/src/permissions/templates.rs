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

//! The values customisation gives in the default role templates, version 1.

use kampus_access::{RoleTemplateDeclaration, Value};

use super::{
    FIELD_EDIT, FIELD_MANAGE, FIELD_VIEW, FLAG_MANAGE, LAYOUT_MANAGE, MENU_MANAGE, TERM_MANAGE, THEME_MANAGE,
};

/// Every permission of this module, allowed: what an administrator holds.
const ALL_ALLOWED: &[(&str, Value)] = &[
    (TERM_MANAGE, Value::Allow),
    (FLAG_MANAGE, Value::Allow),
    (FIELD_MANAGE, Value::Allow),
    (FIELD_VIEW, Value::Allow),
    (FIELD_EDIT, Value::Allow),
    (LAYOUT_MANAGE, Value::Allow),
    (MENU_MANAGE, Value::Allow),
    (THEME_MANAGE, Value::Allow),
];

/// The values customisation gives in the default role templates; templates it leaves out give none of
/// its permissions.
pub const TEMPLATES: &[RoleTemplateDeclaration] = &[
    RoleTemplateDeclaration {
        code: "platform_administrator",
        version: 1,
        values: ALL_ALLOWED,
    },
    RoleTemplateDeclaration {
        code: "campus_administrator",
        version: 1,
        values: ALL_ALLOWED,
    },
    RoleTemplateDeclaration {
        code: "manager",
        version: 1,
        values: ALL_ALLOWED,
    },
    RoleTemplateDeclaration {
        code: "teacher",
        version: 1,
        values: &[
            (TERM_MANAGE, Value::Allow),
            (FIELD_VIEW, Value::Allow),
            (FIELD_EDIT, Value::Allow),
            (LAYOUT_MANAGE, Value::Allow),
        ],
    },
    RoleTemplateDeclaration {
        code: "non_editing_teacher",
        version: 1,
        values: &[(FIELD_VIEW, Value::Allow)],
    },
];
