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

//! The types of notification modules declare, brought into the database when the installation starts.

use kampus_common::Result;
use kampus_module::Module;
use serde_json::json;

use crate::{Notify, sql};

impl Notify {
    /// Brings the types the modules of the build declare into the database; those no module declares any
    /// more are retired, and their notifications stay.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sync_declarations(&self, modules: &[&Module]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut codes = Vec::new();
        for module in modules {
            for declared in module.notification_types {
                codes.push(declared.code.to_owned());
                sqlx::query(sql(format!(
                    "insert into {} (code, module, summary, channels, required) values ($1, $2, $3, $4, $5)
                     on conflict (code) do update set module = excluded.module, summary = excluded.summary,
                         channels = excluded.channels, required = excluded.required, retired_at = null",
                    self.table("notification_type")
                )))
                .bind(declared.code)
                .bind(module.name)
                .bind(declared.summary)
                .bind(json!(declared.channels))
                .bind(declared.required)
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where retired_at is null and not (code = any($1))",
            self.table("notification_type")
        )))
        .bind(&codes)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
