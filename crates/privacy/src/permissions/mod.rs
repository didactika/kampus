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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The permissions privacy declares: asking for the data of other people, seeing their requests, and
//! publishing the policies of the campus.

use kampus_module::{PermissionDeclaration, PermissionKind, Risk, ScopeKind};

pub mod templates;

pub use templates::TEMPLATES;

/// Ask for the data of other people, and see and follow their requests.
pub const REQUEST_MANAGE: &str = "privacy.request.manage";

/// Approve or reject the deletion of personal data, and see the requests waiting.
pub const REQUEST_APPROVE: &str = "privacy.request.approve";

/// Place and release legal holds, which stop deletion.
pub const HOLD_MANAGE: &str = "privacy.hold.manage";

/// Set how long each kind of personal data is kept after an account is deleted.
pub const RETENTION_MANAGE: &str = "privacy.retention.manage";

/// Publish new versions of the policies of the campus.
pub const POLICY_MANAGE: &str = "privacy.policy.manage";

/// The permissions of this module.
pub const PERMISSIONS: &[PermissionDeclaration] = &[
    PermissionDeclaration {
        code: REQUEST_MANAGE,
        kind: PermissionKind::Read,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: REQUEST_APPROVE,
        kind: PermissionKind::Write,
        risks: &[Risk::DataLoss, Risk::PersonalData],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: HOLD_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::PersonalData],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: RETENTION_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::DataLoss],
        lowest_kind: ScopeKind::Campus,
    },
    PermissionDeclaration {
        code: POLICY_MANAGE,
        kind: PermissionKind::Write,
        risks: &[Risk::Configuration],
        lowest_kind: ScopeKind::Campus,
    },
];
