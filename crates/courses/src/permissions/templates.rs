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

//! The values courses gives in the default role templates, version 1.

use kampus_access::{RoleTemplateDeclaration, Value};

use super::{
    CATEGORY_MANAGE, COURSE_ARCHIVE, COURSE_CREATE, COURSE_DELETE, COURSE_UPDATE, GROUPS_ACCESS_ALL,
    GROUPS_MANAGE, HIDDEN_VIEW, SECTION_MANAGE,
};

/// Every permission of this module, allowed: what an administrator holds.
const ALL_ALLOWED: &[(&str, Value)] = &[
    (CATEGORY_MANAGE, Value::Allow),
    (COURSE_CREATE, Value::Allow),
    (COURSE_UPDATE, Value::Allow),
    (COURSE_ARCHIVE, Value::Allow),
    (COURSE_DELETE, Value::Allow),
    (HIDDEN_VIEW, Value::Allow),
    (SECTION_MANAGE, Value::Allow),
    (GROUPS_MANAGE, Value::Allow),
    (GROUPS_ACCESS_ALL, Value::Allow),
];

/// The values courses gives in the default role templates; templates it leaves out give none of its
/// permissions.
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
        code: "course_creator",
        version: 1,
        values: &[(COURSE_CREATE, Value::Allow)],
    },
    RoleTemplateDeclaration {
        code: "teacher",
        version: 1,
        values: &[
            (COURSE_UPDATE, Value::Allow),
            (COURSE_ARCHIVE, Value::Allow),
            (HIDDEN_VIEW, Value::Allow),
            (SECTION_MANAGE, Value::Allow),
            (GROUPS_MANAGE, Value::Allow),
            (GROUPS_ACCESS_ALL, Value::Allow),
        ],
    },
    RoleTemplateDeclaration {
        code: "non_editing_teacher",
        version: 1,
        values: &[(HIDDEN_VIEW, Value::Allow), (GROUPS_ACCESS_ALL, Value::Allow)],
    },
    RoleTemplateDeclaration {
        code: "section_leader",
        version: 1,
        values: &[(HIDDEN_VIEW, Value::Allow), (SECTION_MANAGE, Value::Allow)],
    },
];
