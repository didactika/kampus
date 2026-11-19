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

//! What modules declare: their settings, retired when no module declares them any more; the values set
//! for them stay with their definition.

use kampus_common::{Error, FieldError, Result};
use kampus_module::Module;
use serde_json::json;

use crate::{Settings, sql};

impl Settings {
    /// Brings the settings the modules of the build declare into the database.
    ///
    /// # Errors
    ///
    /// `validation` (`settings.declaration_invalid`) for a declaration without levels or whose default is
    /// not of its kind.
    pub async fn sync_declarations(&self, modules: &[&Module]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut keys = Vec::new();
        for module in modules {
            for setting in module.settings {
                let default = setting.kind.default_value();
                if setting.levels.is_empty() || !setting.kind.accepts(&default) {
                    return Err(Error::validation(
                        "settings.declaration_invalid",
                        vec![FieldError::new("setting", "declaration")],
                    )
                    .with_param("key", setting.key));
                }
                keys.push(setting.key.to_owned());
                let levels: Vec<&str> = setting.levels.iter().map(|l| l.as_str()).collect();
                sqlx::query(sql(format!(
                    "insert into {} (key, module, summary, kind, default_value, levels, permission, restart_required)
                     values ($1, $2, $3, $4, $5, $6, $7, $8)
                     on conflict (key) do update set module = excluded.module, summary = excluded.summary,
                     kind = excluded.kind, default_value = excluded.default_value, levels = excluded.levels,
                     permission = excluded.permission, restart_required = excluded.restart_required, retired_at = null",
                    self.table("definition")
                )))
                .bind(setting.key)
                .bind(module.name)
                .bind(setting.summary)
                .bind(setting.kind.stored())
                .bind(&default)
                .bind(json!(levels))
                .bind(setting.permission)
                .bind(setting.restart_required)
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where not (key = any($1)) and retired_at is null",
            self.table("definition")
        )))
        .bind(&keys)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
