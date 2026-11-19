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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What modules declare: their event types, retired when no module declares them any more, and their
//! listeners, which keep whether an operator turned them off.

use kampus_common::{Error, Result};
use kampus_module::Module;
use serde_json::json;

use crate::{Events, sql};

impl Events {
    /// Brings the event types and listeners the modules of the build declare into the database.
    ///
    /// # Errors
    ///
    /// `validation` (`events.listener_event_unknown`) for a listener of an event no module declares.
    pub async fn sync_declarations(&self, modules: &[&Module]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut codes = Vec::new();
        for module in modules {
            for event in module.events {
                codes.push(event.code.to_owned());
                sqlx::query(sql(format!(
                    "insert into {} (code, version, module, summary, fields, audited, personal_fields)
                     values ($1, $2, $3, $4, $5, $6, $7)
                     on conflict (code) do update set version = excluded.version, module = excluded.module,
                     summary = excluded.summary, fields = excluded.fields, audited = excluded.audited,
                     personal_fields = excluded.personal_fields, retired_at = null",
                    self.table("event_type")
                )))
                .bind(event.code)
                .bind(i32::from(event.version))
                .bind(module.name)
                .bind(event.summary)
                .bind(json!(event.fields))
                .bind(event.audited)
                .bind(json!(event.personal_fields))
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where not (code = any($1)) and retired_at is null",
            self.table("event_type")
        )))
        .bind(&codes)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut kept = Vec::new();
        for module in modules {
            for listener in module.listeners {
                if !codes.iter().any(|c| c == listener.event) {
                    return Err(Error::validation("events.listener_event_unknown", vec![])
                        .with_param("event", listener.event)
                        .with_param("listener", listener.name));
                }
                let id: uuid::Uuid = sqlx::query_scalar(sql(format!(
                    "insert into {} (id, module, type_code, name) values ($1, $2, $3, $4)
                     on conflict (module, type_code, name) do update set retired_at = null returning id",
                    self.table("listener")
                )))
                .bind(uuid::Uuid::now_v7())
                .bind(module.name)
                .bind(listener.event)
                .bind(listener.name)
                .fetch_one(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
                kept.push(id);
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where not (id = any($1)) and retired_at is null",
            self.table("listener")
        )))
        .bind(&kept)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
