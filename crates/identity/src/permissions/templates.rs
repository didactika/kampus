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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The values identity gives in the default role templates, version 1, and the template of the role
//! every active account holds.

use kampus_access::{RoleTemplateDeclaration, Value};

use super::{
    ACCOUNT_DELETE, ACCOUNT_INVITE, ACCOUNT_READ, ACCOUNT_SUSPEND, PROVIDER_MANAGE, TOKEN_MANAGE_ANY,
    TOKEN_MANAGE_OWN,
};

/// Every permission of this module, allowed: what an administrator holds.
const ALL_ALLOWED: &[(&str, Value)] = &[
    (ACCOUNT_INVITE, Value::Allow),
    (ACCOUNT_SUSPEND, Value::Allow),
    (ACCOUNT_DELETE, Value::Allow),
    (ACCOUNT_READ, Value::Allow),
    (PROVIDER_MANAGE, Value::Allow),
    (TOKEN_MANAGE_OWN, Value::Allow),
    (TOKEN_MANAGE_ANY, Value::Allow),
];

/// The values identity gives in the default role templates, version 1; templates it leaves out give
/// none of its permissions.
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
            (ACCOUNT_INVITE, Value::Allow),
            (ACCOUNT_SUSPEND, Value::Allow),
            (ACCOUNT_READ, Value::Allow),
            (TOKEN_MANAGE_OWN, Value::Allow),
        ],
    },
    RoleTemplateDeclaration {
        code: "authenticated_user",
        version: 1,
        values: &[(TOKEN_MANAGE_OWN, Value::Allow)],
    },
];

/// The template of the role every active account holds in its campus.
pub const MEMBER_TEMPLATE: &str = "authenticated_user";
