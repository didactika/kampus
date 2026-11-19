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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The permissions enrolment declares; the values its role templates give them are in [`templates`].

use kampus_access::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// Add, change and delete the method instances of a course.
pub const INSTANCE_MANAGE: &str = "enrol.instance.manage";
/// Enrol people, change and remove their enrolments.
pub const ENROLMENT_MANAGE: &str = "enrol.enrolment.manage";
/// See the participants of a course and their enrolments.
pub const ENROLMENT_READ: &str = "enrol.enrolment.read";
/// Approve or reject requests to join.
pub const REQUEST_DECIDE: &str = "enrol.request.decide";
/// Enrol oneself through a self instance.
pub const SELF: &str = "enrol.self";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: INSTANCE_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::TrustEscalation],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: ENROLMENT_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData, Risk::TrustEscalation],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: ENROLMENT_READ,
        kind: PermissionKind::Read,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: REQUEST_DECIDE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Course,
    },
    PermissionDeclaration {
        code: SELF,
        kind: PermissionKind::Write,
        risks: &[],
        lowest_kind: ScopeKind::Course,
    },
];
