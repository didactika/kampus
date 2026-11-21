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

//! Files a module makes itself, such as the package of an export: no upload by a person, but the same checks
//! of the area (size and types), the same scan and storage by hash, and the same quotas once attached.

use bytes::Bytes;
use kampus_access::{PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde_json::{Value, json};
use sqlx::Row;

use crate::support::sniff;
use crate::{File, FileId, Files, sql};

impl Files {
    /// Stores content a module produced, with the media type the module knows it has, as a file of an area,
    /// attached at once and owned by a person, who is counted for it in the quotas.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown area, `validation` when the content does not fit the area, `unavailable`
    /// when the database or the store fails.
    pub async fn store_for(
        &self,
        area: &str,
        scope: ScopeId,
        owner: PrincipalId,
        name: &str,
        (content, media_type): (Bytes, &str),
    ) -> Result<File> {
        let row = sqlx::query(sql(format!(
            "select max_size, allowed_types from {} where code = $1 and retired_at is null",
            self.table("area")
        )))
        .bind(area)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.area_unknown").with_param("area", area.to_owned()))?;
        let max_size: i64 = row.get("max_size");
        if i64::try_from(content.len()).unwrap_or(i64::MAX) > max_size {
            return Err(
                Error::validation("files.too_large", vec![FieldError::new("content", "max")])
                    .with_param("max_size", max_size.to_string()),
            );
        }
        let allowed: Vec<String> =
            serde_json::from_value(row.get::<Value, _>("allowed_types")).unwrap_or_default();
        if !sniff::allowed(&allowed, media_type) {
            return Err(Error::validation(
                "files.type_not_allowed",
                vec![FieldError::new("content", "type")],
            )
            .with_param("media_type", media_type.to_owned()));
        }
        let status = self.scan(&content).await?;
        let blob = self.store(content, media_type, status).await?;
        let id = FileId::generate();
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "with used as (update {blobs} set ref_count = ref_count + 1 where id = $2 returning media_type, size)
             insert into {files} as f (id, blob_id, area_code, scope_id, name, media_type, size, uploader)
             select $1, $2, $3, $4, $5, used.media_type, used.size, $6 from used",
            blobs = self.table("blob"),
            files = self.table("file")
        )))
        .bind(id.as_uuid())
        .bind(blob)
        .bind(area)
        .bind(scope.as_uuid())
        .bind(name)
        .bind(owner.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "delete from {} where blob_id = $1",
            self.table("trash")
        )))
        .bind(blob)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let attached = self.attach_in(&mut tx, id, area, Some(owner.as_uuid())).await?;
        let payload = json!({
            "file": attached.id, "area": attached.area, "size": attached.size, "media_type": attached.media_type,
        });
        self.publish_in(&mut tx, crate::events::UPLOADED, &attached, payload)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(attached)
    }

    /// The content of a file, for the module that made it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown file, `unavailable` when the store fails.
    pub async fn read_content(&self, id: FileId) -> Result<Bytes> {
        let key = self.key_of(id).await?;
        self.storage()?.get(&key).await
    }

    /// Deletes a file its module made, once the module has no use for it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown file.
    pub async fn remove(&self, id: FileId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        self.detach_in(&mut tx, id).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
