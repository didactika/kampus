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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The permissions records declares: seeing grades, grading, overriding, locking, hiding, configuring the
//! records of a course and exporting them.

use kampus_module::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// See the own grades and the structure of the records of a course.
pub const VIEW_OWN: &str = "records.view_own";

/// See the grades of everyone in a course.
pub const VIEW_ALL: &str = "records.view_all";

/// See grades and records that are hidden.
pub const VIEW_HIDDEN: &str = "records.view_hidden";

/// Set the grades of people.
pub const GRADE: &str = "records.grade";

/// Override a grade, and remove the override.
pub const OVERRIDE: &str = "records.override";

/// Lock and unlock records and grades.
pub const LOCK: &str = "records.lock";

/// Hide records and grades until a date, and show them again.
pub const HIDE: &str = "records.hide";

/// Configure the categories and records of a course.
pub const MANAGE: &str = "records.manage";

/// Export the grades of a course.
pub const EXPORT: &str = "records.export";

/// A permission of this module.
const fn declared(code: &'static str, kind: PermissionKind, risks: &'static [Risk]) -> PermissionDeclaration {
    PermissionDeclaration {
        code,
        kind,
        risks,
        lowest_kind: ScopeKind::Course,
    }
}

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    declared(VIEW_OWN, PermissionKind::Read, &[]),
    declared(VIEW_ALL, PermissionKind::Read, &[Risk::PersonalData]),
    declared(VIEW_HIDDEN, PermissionKind::Read, &[Risk::PersonalData]),
    declared(GRADE, PermissionKind::Write, &[]),
    declared(OVERRIDE, PermissionKind::Write, &[]),
    declared(LOCK, PermissionKind::Write, &[]),
    declared(HIDE, PermissionKind::Write, &[]),
    declared(MANAGE, PermissionKind::Write, &[Risk::DataLoss]),
    declared(EXPORT, PermissionKind::Read, &[Risk::PersonalData]),
];
