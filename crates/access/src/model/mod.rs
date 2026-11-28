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

//! The concepts of access control: identifiers, scope kinds, permission values, risks, declarations and
//! the actor of a check.

mod acting;

pub use acting::{Acting, Carried};
pub use kampus_module::{
    PermissionDeclaration, PermissionKind, Risk, RoleTemplateDeclaration, ScopeKind, Value,
};

kampus_common::define_id! {
    /// Identifies a scope, a node of the scope tree.
    pub struct ScopeId;
}

kampus_common::define_id! {
    /// Identifies a principal: anything that can hold grants.
    pub struct PrincipalId;
}

kampus_common::define_id! {
    /// Identifies a role of a campus.
    pub struct RoleId;
}

kampus_common::define_id! {
    /// Identifies a grant of a role to a principal in a scope.
    pub struct GrantId;
}

kampus_common::define_id! {
    /// Identifies a "view as" session.
    pub struct ViewAsId;
}

/// Who a check is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A principal with its grants.
    Principal(PrincipalId),
    /// Someone not signed in, in a campus: only what the campus's guest role reads.
    Visitor(ScopeId),
    /// A principal seeing a scope as another role, read-only.
    ViewAs(PrincipalId, ViewAsId),
}

/// The kind of an allowance between roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowanceKind {
    /// May grant the target role.
    Grant,
    /// May override the target role.
    Override,
    /// May view as the target role.
    ViewAs,
}

impl AllowanceKind {
    /// The name of the allowance in the database.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Override => "override",
            Self::ViewAs => "view_as",
        }
    }
}
