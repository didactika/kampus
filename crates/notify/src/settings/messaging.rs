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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The settings of messaging in a campus: who may write to whom, and how long messages are kept.

use kampus_access::ScopeId;
use kampus_common::Result;
use kampus_module::{ScopeKind, SettingDeclaration, ValueKind};
use kampus_settings::Settings;

use crate::Notify;
use crate::permissions::DEFAULTS_MANAGE;

/// Who may write to whom in a campus: people who take part in a course together, anyone of the campus, or
/// nobody; whoever holds `notify.message.anyone` writes to anyone of the campus.
pub const MESSAGING: &str = "notify.messaging";

/// How many days messages are kept; none are removed for their age with 0.
pub const RETENTION: &str = "notify.message_retention_days";

/// The options of [`MESSAGING`].
pub const MESSAGING_OPTIONS: &[&str] = &["course", "campus", "nobody"];

/// The settings of this module.
pub const SETTINGS: &[SettingDeclaration] = &[
    SettingDeclaration {
        key: MESSAGING,
        summary: "Who may write to whom: people who take part in a course together, anyone of the campus, or nobody",
        group: "messaging",
        kind: ValueKind::Choice {
            options: MESSAGING_OPTIONS,
            default: "course",
        },
        levels: &[ScopeKind::Campus],
        permission: DEFAULTS_MANAGE,
        restart_required: false,
    },
    SettingDeclaration {
        key: RETENTION,
        summary: "How many days messages are kept before they are removed; 0 keeps them until every member deletes them",
        group: "messaging",
        kind: ValueKind::Integer {
            min: 0,
            max: 3650,
            default: 0,
        },
        levels: &[ScopeKind::Campus],
        permission: DEFAULTS_MANAGE,
        restart_required: false,
    },
];

impl Notify {
    /// Who may write to whom in a campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown campus.
    pub async fn messaging(&self, campus: ScopeId) -> Result<String> {
        let resolved = Settings::new(self.db.clone(), self.access())
            .resolve(MESSAGING, campus)
            .await?;
        Ok(resolved.value.as_str().unwrap_or("course").to_owned())
    }

    /// How many days a campus keeps messages; 0 keeps them.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown campus.
    pub async fn retention_days(&self, campus: ScopeId) -> Result<i64> {
        let resolved = Settings::new(self.db.clone(), self.access())
            .resolve(RETENTION, campus)
            .await?;
        Ok(resolved.value.as_i64().unwrap_or(0))
    }
}
