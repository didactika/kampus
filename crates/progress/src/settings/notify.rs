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

//! The settings progress declares: whether the people of a course are told about their progress there.

use kampus_access::ScopeId;
use kampus_common::Result;
use kampus_module::{ScopeKind, SettingDeclaration, ValueKind};
use kampus_settings::Settings;

use crate::Progress;
use crate::permissions::MANAGE;

/// The key of whether people are told about their progress.
pub const NOTIFY: &str = "progress.notify";

/// The settings of this module.
pub const SETTINGS: &[SettingDeclaration] = &[SettingDeclaration {
    key: NOTIFY,
    summary: "Whether the people of a course are told when they complete it or a teacher decides their progress",
    group: "completion",
    kind: ValueKind::Boolean { default: true },
    levels: &[ScopeKind::Campus, ScopeKind::Category, ScopeKind::Course],
    permission: MANAGE,
    restart_required: false,
}];

impl Progress {
    /// Whether the people of a course, or of the course above a scope, are told about their progress.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn notifies(&self, scope: ScopeId) -> Result<bool> {
        let resolved = Settings::new(self.db.clone(), self.access.clone())
            .resolve(NOTIFY, scope)
            .await?;
        Ok(resolved.value.as_bool().unwrap_or(true))
    }
}
