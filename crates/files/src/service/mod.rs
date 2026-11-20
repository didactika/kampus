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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The service API of files, one file per area, and what they share: reading files from their rows, the
//! principal behind an actor, publishing, and the areas modules declare.

pub mod file;
pub mod purge;
pub mod quota;
pub mod serve;
pub mod upload;

use kampus_access::Actor;
use kampus_common::{Result, Timestamp};
use kampus_module::Module;
use serde_json::{Value, json};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use crate::{File, FileId, Files, sql};

/// The columns of a file.
pub(crate) const COLUMNS: &str =
    "f.id, f.area_code, f.scope_id, f.name, f.media_type, f.size, f.uploader, f.attached, f.created_at";

/// A file from its row.
pub(crate) fn file(row: &PgRow) -> File {
    File {
        id: FileId::from_uuid(row.get("id")),
        area: row.get("area_code"),
        scope: kampus_access::ScopeId::from_uuid(row.get("scope_id")),
        name: row.get("name"),
        media_type: row.get("media_type"),
        size: row.get("size"),
        uploader: row.get("uploader"),
        attached: row.get("attached"),
        created_at: Timestamp::from_offset_date_time(row.get("created_at")),
    }
}

/// The principal behind an actor; none for a visitor.
pub(crate) const fn principal(actor: &Actor) -> Option<uuid::Uuid> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(p.as_uuid()),
        Actor::Visitor(_) => None,
    }
}

impl Files {
    /// Publishes an event about a file, in the transaction of the change.
    pub(crate) async fn publish_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        file: &File,
        payload: Value,
    ) -> Result<()> {
        let mut event = kampus_events::Publish::new(code, Some(file.id.as_uuid()), payload);
        event.scope = Some(file.scope.as_uuid());
        event.actor = file.uploader;
        event.summary = Some(format!("{code} {}", file.id));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }

    /// Brings the areas the modules of the build declare into the database; those no module declares any
    /// more are retired, and their files stay.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sync_declarations(&self, modules: &[&Module]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut codes = Vec::new();
        for module in modules {
            for area in module.file_areas {
                codes.push(area.code.to_owned());
                sqlx::query(sql(format!(
                    "insert into {} (code, module, summary, max_size, allowed_types, upload_permission)
                     values ($1, $2, $3, $4, $5, $6)
                     on conflict (code) do update set module = excluded.module, summary = excluded.summary,
                     max_size = excluded.max_size, allowed_types = excluded.allowed_types,
                     upload_permission = excluded.upload_permission, retired_at = null",
                    self.table("area")
                )))
                .bind(area.code)
                .bind(module.name)
                .bind(area.summary)
                .bind(i64::try_from(area.max_size).unwrap_or(i64::MAX))
                .bind(json!(area.allowed_types))
                .bind(area.upload_permission)
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where not (code = any($1)) and retired_at is null",
            self.table("area")
        )))
        .bind(&codes)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
