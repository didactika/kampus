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

//! The permissions courses declares; the values its role templates give them are in [`templates`].

use kampus_access::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// Create, edit, move, reorder and delete categories.
pub const CATEGORY_MANAGE: &str = "courses.category.manage";
/// Create courses in a category.
pub const COURSE_CREATE: &str = "courses.course.create";
/// Edit a course, publish it and take it back to draft.
pub const COURSE_UPDATE: &str = "courses.course.update";
/// Archive a course and bring it back.
pub const COURSE_ARCHIVE: &str = "courses.course.archive";
/// Schedule and cancel the deletion of a course; its owners may do it too.
pub const COURSE_DELETE: &str = "courses.course.delete";
/// See hidden categories, courses and sections, marked as hidden.
pub const HIDDEN_VIEW: &str = "courses.hidden.view";
/// Add, edit, move, hide and delete sections.
pub const SECTION_MANAGE: &str = "courses.section.manage";
/// Create groups and group sets and manage their members.
pub const GROUPS_MANAGE: &str = "courses.groups.manage";
/// See and work with every group, even in separate group mode.
pub const GROUPS_ACCESS_ALL: &str = "courses.groups.access_all";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: CATEGORY_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration, Risk::DataLoss],
        lowest_kind: ScopeKind::Category,
    },
    PermissionDeclaration {
        code: COURSE_CREATE,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Category,
    },
    PermissionDeclaration {
        code: COURSE_UPDATE,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: COURSE_ARCHIVE,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: COURSE_DELETE,
        kind: PermissionKind::Write,
        risks: &[Risk::DataLoss],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: HIDDEN_VIEW,
        kind: PermissionKind::Read,
        risks: &[],
        lowest_kind: ScopeKind::Section,
    },
    PermissionDeclaration {
        code: SECTION_MANAGE,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Section,
    },
    PermissionDeclaration {
        code: GROUPS_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: GROUPS_ACCESS_ALL,
        kind: PermissionKind::Read,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Section,
    },
];
