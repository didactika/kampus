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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The role templates of access, version 1: the default roles of the domain model and the values they
//! give to the permissions of access.

use super::{
    GRANT_CREATE, GRANT_DELETE, OVERRIDE_RISKY, OVERRIDE_UPDATE, REVIEW, ROLE_MANAGE, SCOPE_FREEZE, VIEW_AS,
};
use crate::model::{RoleTemplateDeclaration, Value};

/// Every permission of this module, allowed: what an administrator holds.
const ALL_ALLOWED: &[(&str, Value)] = &[
    (GRANT_CREATE, Value::Allow),
    (GRANT_DELETE, Value::Allow),
    (OVERRIDE_UPDATE, Value::Allow),
    (OVERRIDE_RISKY, Value::Allow),
    (ROLE_MANAGE, Value::Allow),
    (SCOPE_FREEZE, Value::Allow),
    (VIEW_AS, Value::Allow),
    (REVIEW, Value::Allow),
];

/// The role templates of this module, version 1 (the default roles of the domain model).
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
        values: &[
            (GRANT_CREATE, Value::Allow),
            (GRANT_DELETE, Value::Allow),
            (OVERRIDE_UPDATE, Value::Allow),
            (VIEW_AS, Value::Allow),
            (REVIEW, Value::Allow),
        ],
    },
    RoleTemplateDeclaration {
        code: "course_creator",
        version: 1,
        values: &[],
    },
    RoleTemplateDeclaration {
        code: "teacher",
        version: 1,
        values: &[
            (GRANT_CREATE, Value::Allow),
            (VIEW_AS, Value::Allow),
            (REVIEW, Value::Allow),
        ],
    },
    RoleTemplateDeclaration {
        code: "non_editing_teacher",
        version: 1,
        values: &[(REVIEW, Value::Allow)],
    },
    RoleTemplateDeclaration {
        code: "section_leader",
        version: 1,
        values: &[(GRANT_CREATE, Value::Allow), (REVIEW, Value::Allow)],
    },
    RoleTemplateDeclaration {
        code: "learner",
        version: 1,
        values: &[],
    },
    RoleTemplateDeclaration {
        code: "mentor",
        version: 1,
        values: &[],
    },
    RoleTemplateDeclaration {
        code: "authenticated_user",
        version: 1,
        values: &[],
    },
    RoleTemplateDeclaration {
        code: "guest",
        version: 1,
        values: &[],
    },
];

/// The template of the role visitors read with.
pub const GUEST_TEMPLATE: &str = "guest";
