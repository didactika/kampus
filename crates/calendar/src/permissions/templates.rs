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

//! The values calendar gives in the default role templates, version 1: whoever takes part in a course sees
//! its calendar, whoever teaches adds to it.

use kampus_module::{RoleTemplateDeclaration, Value};

use super::{MANAGE, VIEW};

/// Seeing and managing.
const BOTH: &[(&str, Value)] = &[(VIEW, Value::Allow), (MANAGE, Value::Allow)];

/// Seeing only.
const SEEING: &[(&str, Value)] = &[(VIEW, Value::Allow)];

/// The values of this module in the role templates.
pub const TEMPLATES: &[RoleTemplateDeclaration] = &[
    RoleTemplateDeclaration {
        code: "platform_administrator",
        version: 1,
        values: BOTH,
    },
    RoleTemplateDeclaration {
        code: "campus_administrator",
        version: 1,
        values: BOTH,
    },
    RoleTemplateDeclaration {
        code: "manager",
        version: 1,
        values: BOTH,
    },
    RoleTemplateDeclaration {
        code: "teacher",
        version: 1,
        values: BOTH,
    },
    RoleTemplateDeclaration {
        code: "non_editing_teacher",
        version: 1,
        values: SEEING,
    },
    RoleTemplateDeclaration {
        code: "learner",
        version: 1,
        values: SEEING,
    },
];
