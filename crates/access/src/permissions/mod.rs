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

//! The permissions access declares (DEC-015, RN-021); the values its role templates give them are in
//! [`templates`].
//! campus starts from. Other modules add their own permissions and values to the templates.

use crate::model::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::{GUEST_TEMPLATE, TEMPLATES};

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
