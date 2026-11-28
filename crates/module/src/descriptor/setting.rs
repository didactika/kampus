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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The settings a module declares: typed values kept per scope and resolved through the scope tree, each
//! with the levels where it may be set and the permission that sets it.

use crate::{ScopeKind, ValueKind};

/// A setting a module declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingDeclaration {
    /// The key, `<module>.<name>` (`identity.self_sign_up`).
    pub key: &'static str,
    /// What it sets.
    pub summary: &'static str,
    /// The group it shows in, among the settings of its module (`sign_up`, `sessions`): the administration
    /// gives each group of a module its own section, so a module with many settings keeps them apart.
    pub group: &'static str,
    /// Its type and default.
    pub kind: ValueKind,
    /// The kinds of scope where it may be set, such as the platform and the campus.
    pub levels: &'static [ScopeKind],
    /// The permission that sets it in a scope.
    pub permission: &'static str,
    /// Whether the roles that read it apply a change only at their next start.
    pub restart_required: bool,
}
