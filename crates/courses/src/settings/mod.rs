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

//! The settings courses declares: the group mode, resolved through course, section and activity, so a
//! section can work in groups while the rest of the course does not.

use kampus_access::ScopeId;
use kampus_common::Result;
use kampus_module::{ScopeKind, SettingDeclaration, ValueKind};
use kampus_settings::Settings;

use crate::permissions::GROUPS_MANAGE;
use crate::{Courses, GroupMode};

/// The key of the group mode.
pub const GROUP_MODE: &str = "courses.group_mode";

/// The settings of this module.
pub const SETTINGS: &[SettingDeclaration] = &[SettingDeclaration {
    key: GROUP_MODE,
    summary: "Whether people work in groups: none, separate (each group on its own) or visible",
    kind: ValueKind::Choice {
        options: &["none", "separate", "visible"],
        default: "none",
    },
    levels: &[ScopeKind::Course, ScopeKind::Section, ScopeKind::Activity],
    permission: GROUPS_MANAGE,
    restart_required: false,
}];

impl Courses {
    /// The group mode in a course, a section or an activity, as its settings resolve it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn group_mode(&self, scope: ScopeId) -> Result<GroupMode> {
        let resolved = Settings::new(self.db.clone(), self.access.clone())
            .resolve(GROUP_MODE, scope)
            .await?;
        Ok(match resolved.value.as_str() {
            Some("separate") => GroupMode::Separate,
            Some("visible") => GroupMode::Visible,
            _ => GroupMode::None,
        })
    }
}
