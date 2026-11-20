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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The permissions progress declares: seeing progress, marking the own, deciding it for others and
//! configuring what a course module requires.

use kampus_module::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// See the own progress.
pub const VIEW_OWN: &str = "progress.view_own";

/// See the progress of everyone.
pub const VIEW_ALL: &str = "progress.view_all";

/// Mark the own course modules done.
pub const MARK_OWN: &str = "progress.mark_own";

/// Decide the state of a person on a course module, with a reason.
pub const EXCEPTION: &str = "progress.exception";

/// Configure what a course module requires.
pub const MANAGE: &str = "progress.manage";

/// A permission of this module.
const fn declared(code: &'static str, kind: PermissionKind, risks: &'static [Risk]) -> PermissionDeclaration {
    PermissionDeclaration {
        code,
        kind,
        risks,
        lowest_kind: ScopeKind::Activity,
    }
}

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    declared(VIEW_OWN, PermissionKind::Read, &[]),
    declared(VIEW_ALL, PermissionKind::Read, &[Risk::PersonalData]),
    declared(MARK_OWN, PermissionKind::Write, &[]),
    declared(EXCEPTION, PermissionKind::Write, &[]),
    declared(MANAGE, PermissionKind::Write, &[]),
];
