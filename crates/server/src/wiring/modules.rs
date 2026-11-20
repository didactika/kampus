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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The composition of modules: the server reads the descriptor of every module of the build
//! from the generated registry, in the order of the module table of the architecture. `kampus-db` adds
//! its own steps first.

use std::sync::LazyLock;

use kampus_db::ModuleSteps;

use crate::wiring::registry::MODULES;

/// The upgrade steps of every module of this build, in dependency order.
pub static SCHEMAS: LazyLock<Vec<ModuleSteps>> =
    LazyLock::new(|| MODULES.iter().map(|m| m.schema()).collect());

/// Brings what the modules of this build declare into the database at start-up: permissions and role
/// templates, with the values of every module for each template merged, event types and listeners, settings, and job
/// types and schedules.
///
/// # Errors
///
/// The errors of [`kampus_access::Access::sync`] and [`kampus_tasks::Tasks::sync_declarations`].
pub async fn sync_declarations(db: &kampus_db::Database) -> kampus_common::Result<()> {
    use kampus_common::Clock;
    kampus_access::Access::new(db.clone(), std::sync::Arc::new(kampus_common::SystemClock))
        .sync_modules(MODULES)
        .await?;
    kampus_events::Events::new(db.clone())
        .sync_declarations(MODULES)
        .await?;
    kampus_settings::Settings::new(
        db.clone(),
        kampus_access::Access::new(db.clone(), std::sync::Arc::new(kampus_common::SystemClock)),
    )
    .sync_declarations(MODULES)
    .await?;
    kampus_customise::Customise::new(
        db.clone(),
        kampus_access::Access::new(db.clone(), std::sync::Arc::new(kampus_common::SystemClock)),
    )
    .sync_declarations(MODULES)
    .await?;
    kampus_files::Files::new(
        db.clone(),
        kampus_access::Access::new(db.clone(), std::sync::Arc::new(kampus_common::SystemClock)),
        None,
        [0; 32],
        String::new(),
    )
    .sync_declarations(MODULES)
    .await?;
    kampus_tasks::Tasks::new(db.clone())
        .sync_declarations(MODULES, kampus_common::SystemClock.now())
        .await
}

/// The oldest release still supported: columns may be dropped only when it no longer reads them.
pub const OLDEST_SUPPORTED: &str = kampus_common::PLATFORM.upgrade_from;
