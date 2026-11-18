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

//! What this module contributes to the platform (DEC-015): its permissions and the role templates every
//! campus starts from. Other modules add their own permissions and values to the templates.

use crate::model::{PermissionDeclaration, PermissionKind, Risk, RoleTemplateDeclaration, ScopeKind, Value};

/// Grant a role in a scope (RN-013).
pub const GRANT_CREATE: &str = "access.grant.create";
/// Remove a manual grant (RN-016).
pub const GRANT_DELETE: &str = "access.grant.delete";
/// Override a role's permission in a scope (RN-014).
pub const OVERRIDE_UPDATE: &str = "access.override.update";
/// Override a permission with risks (RN-014).
pub const OVERRIDE_RISKY: &str = "access.override.risky";
/// Create and change roles.
pub const ROLE_MANAGE: &str = "access.role.manage";
/// Freeze and unfreeze a scope (RN-011).
pub const SCOPE_FREEZE: &str = "access.scope.freeze";
/// See a scope as another role (RN-018).
pub const VIEW_AS: &str = "access.view_as";
/// Read who holds which roles and why a check is decided.
pub const REVIEW: &str = "access.review";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: GRANT_CREATE,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation],
        lowest_kind: ScopeKind::Section,
    },
    PermissionDeclaration {
        code: GRANT_DELETE,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation],
        lowest_kind: ScopeKind::Section,
    },
    PermissionDeclaration {
        code: OVERRIDE_UPDATE,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: OVERRIDE_RISKY,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: ROLE_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation, Risk::Configuration],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: SCOPE_FREEZE,
        kind: PermissionKind::Write,
        risks: &[Risk::DataLoss],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: VIEW_AS,
        kind: PermissionKind::Read,
        risks: &[],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: REVIEW,
        kind: PermissionKind::Read,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Activity,
    },
];

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

/// The template of the role visitors read with (RN-009).
pub const GUEST_TEMPLATE: &str = "guest";
