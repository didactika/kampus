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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The permissions users declares; the values its role templates give them are in [`templates`].

use kampus_access::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// Edit one's own profile and preferences.
pub const PROFILE_UPDATE_OWN: &str = "users.profile.update_own";
/// Edit the profile and preferences of anyone in the campus.
pub const PROFILE_UPDATE_ANY: &str = "users.profile.update_any";
/// Create system groups and manage their members.
pub const GROUP_MANAGE: &str = "users.system_group.manage";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: PROFILE_UPDATE_OWN,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: PROFILE_UPDATE_ANY,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: GROUP_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Category,
    },
];
