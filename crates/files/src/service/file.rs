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

//! Files and their owners: a file is readable exactly when its owner is, which the module that declares
//! its area answers; until a module attaches it, only its uploader reads it. Modules attach and detach
//! files inside the transaction that links them to their records, and the count of the uses of the content
//! moves in that same transaction.

use kampus_common::{Error, FieldError, Result};
use kampus_module::{FileRead, ServiceContext};
use serde_json::json;
use sqlx::{PgConnection, Row};

use super::{COLUMNS, file};
use crate::{File, FileId, Files, sql};

impl Files {
    /// A file as stored, whoever asks.
    pub(crate) async fn stored(&self, id: FileId) -> Result<File> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} f where f.id = $1",
            self.table("file")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.file_not_found"))?;
        Ok(file(&row))
    }

    /// Whether a principal may read a file: its uploader until it is attached, and then whoever the
    /// module of its area says may read its owner.
    ///
    /// # Errors
    ///
    /// Those of the module that answers.
    pub async fn readable(
        &self,
        principal: Option<uuid::Uuid>,
        file: &File,
        services: &ServiceContext,
    ) -> Result<bool> {
        if !file.attached {
            return Ok(principal.is_some() && principal == file.uploader);
        }
        match self.area_declaration(&file.area) {
            Some(area) => {
                (area.readable)(FileRead {
                    file: file.id.as_uuid(),
                    principal,
                    services,
                })
                .await
            }
            None => Ok(false),
        }
    }

    /// A file the principal may read.
    ///
    /// # Errors
    ///
    /// `not_found` (`files.file_not_found`) for an unknown file or one the principal may not read, which
    /// answer the same.
    pub async fn file(
        &self,
        principal: Option<uuid::Uuid>,
        id: FileId,
        services: &ServiceContext,
    ) -> Result<File> {
        let found = self.stored(id).await?;
        if self.readable(principal, &found, services).await? {
            Ok(found)
        } else {
            Err(Error::not_found("files.file_not_found"))
        }
    }

    /// Attaches a file of an area to the record of the calling module that owns it, in the transaction that
    /// links them; only the uploader's own files, when one is given.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown file or one of someone else, `validation` (`files.area_mismatch`) for a
    /// file of another area, `conflict` (`files.already_attached`).
    pub async fn attach_in(
        &self,
        conn: &mut PgConnection,
        id: FileId,
        area: &str,
        uploader: Option<uuid::Uuid>,
    ) -> Result<File> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} f where f.id = $1 for update",
            self.table("file")
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.file_not_found"))?;
        let found = file(&row);
        if uploader.is_some() && uploader != found.uploader {
            return Err(Error::not_found("files.file_not_found"));
        }
        if found.area != area {
            return Err(
                Error::validation("files.area_mismatch", vec![FieldError::new("file", "area")])
                    .with_param("area", area.to_owned()),
            );
        }
        if found.attached {
            return Err(Error::conflict("files.already_attached"));
        }
        sqlx::query(sql(format!(
            "update {} set attached = true where id = $1",
            self.table("file")
        )))
        .bind(id.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let attached = File {
            attached: true,
            ..found
        };
        let payload = json!({ "file": attached.id, "area": attached.area });
        self.publish_in(conn, crate::events::ATTACHED, &attached, payload)
            .await?;
        Ok(attached)
    }

    /// Detaches a file from its owner, which deletes it, in the transaction that unlinks them; content no
    /// file uses any more goes to the trash.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown file.
    pub async fn detach_in(&self, conn: &mut PgConnection, id: FileId) -> Result<()> {
        let row = sqlx::query(sql(format!(
            "delete from {} f where f.id = $1 returning {COLUMNS}, f.blob_id",
            self.table("file")
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.file_not_found"))?;
        let removed = file(&row);
        let blob: uuid::Uuid = row.get("blob_id");
        let left: i32 = sqlx::query_scalar(sql(format!(
            "update {} set ref_count = ref_count - 1 where id = $1 returning ref_count",
            self.table("blob")
        )))
        .bind(blob)
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        if left == 0 {
            sqlx::query(sql(format!(
                "insert into {} (blob_id) values ($1) on conflict do nothing",
                self.table("trash")
            )))
            .bind(blob)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        let payload = json!({ "file": removed.id, "area": removed.area });
        self.publish_in(conn, crate::events::DETACHED, &removed, payload)
            .await
    }
}
