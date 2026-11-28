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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of settings: a resolved value with where it came from, and the record of a change.

use kampus_access::ScopeId;
use kampus_common::Timestamp;
use serde::Serialize;
use serde_json::Value;

/// Where a resolved value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum Origin {
    /// No scope sets it: the declared default.
    Default,
    /// Set in this scope, the nearest one or one that locks it.
    Scope {
        /// The scope.
        scope: ScopeId,
        /// Whether that scope locks it, so no scope below may change it.
        locked: bool,
    },
}

/// The value of a setting in a scope.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Resolved {
    /// The key.
    pub key: String,
    /// Its value there.
    pub value: Value,
    /// Where the value came from.
    pub origin: Origin,
}

/// A change of a setting in a scope.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct SettingChange {
    /// The key.
    pub key: String,
    /// The scope.
    pub scope: ScopeId,
    /// Who changed it; the operator when absent.
    pub actor: Option<uuid::Uuid>,
    /// The value before; unset when absent.
    pub before: Option<Value>,
    /// The value after; unset when absent.
    pub after: Option<Value>,
    /// Whether it was locked after the change.
    pub locked: bool,
    /// When.
    pub at: Timestamp,
}

/// A declared setting, as stored.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Definition {
    /// The key.
    pub key: String,
    /// The module that declares it.
    pub module: String,
    /// What it sets.
    pub summary: String,
    /// Its default.
    pub default_value: Value,
    /// The kinds of scope where it may be set.
    pub levels: Vec<String>,
    /// The permission that sets it.
    pub permission: String,
    /// Whether a change applies only at the next start.
    pub restart_required: bool,
}
