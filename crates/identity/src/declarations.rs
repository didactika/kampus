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

//! The permissions identity declares (DEC-015) and the values its role templates give them.

use kampus_access::{PermissionDeclaration, PermissionKind, Risk, RoleTemplateDeclaration, ScopeKind, Value};

/// Invite people to a campus and create service accounts (UC-009).
pub const ACCOUNT_INVITE: &str = "identity.account.invite";
/// Suspend and reactivate accounts (RN-023).
pub const ACCOUNT_SUSPEND: &str = "identity.account.suspend";
/// Schedule and cancel the deletion of accounts (RN-034, RN-035).
pub const ACCOUNT_DELETE: &str = "identity.account.delete";
/// Read the accounts of a campus.
pub const ACCOUNT_READ: &str = "identity.account.read";
/// Configure the identity providers of a campus.
pub const PROVIDER_MANAGE: &str = "identity.provider.manage";
/// Create and revoke one's own API tokens (RN-033).
pub const TOKEN_MANAGE_OWN: &str = "identity.token.manage_own";
/// Create and revoke the API tokens of any account, such as service accounts.
pub const TOKEN_MANAGE_ANY: &str = "identity.token.manage_any";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: ACCOUNT_INVITE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData, Risk::Spam],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: ACCOUNT_SUSPEND,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: ACCOUNT_DELETE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData, Risk::DataLoss],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: ACCOUNT_READ,
        kind: PermissionKind::Read,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: PROVIDER_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation, Risk::Configuration],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: TOKEN_MANAGE_OWN,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: TOKEN_MANAGE_ANY,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation],
        lowest_kind: ScopeKind::Campus,
    },
];

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
