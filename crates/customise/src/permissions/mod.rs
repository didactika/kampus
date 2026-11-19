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

//! The permissions customisation declares; the values its role templates give them are in [`templates`].

use kampus_access::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// Rename the words of the domain in a scope.
pub const TERM_MANAGE: &str = "customise.term.manage";
/// Turn features on or off in a scope.
pub const FLAG_MANAGE: &str = "customise.flag.manage";
/// Add and remove custom fields in a scope.
pub const FIELD_MANAGE: &str = "customise.field.manage";
/// See the values of private custom fields.
pub const FIELD_VIEW: &str = "customise.field.view";
/// Change the values of custom fields.
pub const FIELD_EDIT: &str = "customise.field.edit";
/// Arrange the widgets of the pages of a scope.
pub const LAYOUT_MANAGE: &str = "customise.layout.manage";
/// Change the navigation menus of a scope.
pub const MENU_MANAGE: &str = "customise.menu.manage";
/// Change the theme of a scope.
pub const THEME_MANAGE: &str = "customise.theme.manage";
/// Write, change and remove the rules of a scope.
pub const RULE_MANAGE: &str = "customise.rule.manage";
/// Export the customisations of a scope and import packages into it.
pub const PACKAGE_MANAGE: &str = "customise.package.manage";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: TERM_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Section,
    },
    PermissionDeclaration {
        code: FLAG_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Section,
    },
    PermissionDeclaration {
        code: FIELD_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration, Risk::DataLoss],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: FIELD_VIEW,
        kind: PermissionKind::Read,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: FIELD_EDIT,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Activity,
    },
    PermissionDeclaration {
        code: LAYOUT_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: MENU_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: THEME_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: RULE_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: PACKAGE_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Course,
    },
];
