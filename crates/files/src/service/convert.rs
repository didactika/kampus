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

//! Conversions: content derived from a file, such as a thumbnail, made by a job the first time someone
//! asks for it and kept as content of its own; the original is never changed. A conversion belongs to the
//! content, so every file with that content shares it.

use std::io::Cursor;

use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_module::ServiceContext;
use serde::Serialize;
use serde_json::json;
use sqlx::Row;

use super::serve::Part;
use crate::db::steps::CONVERSION_KINDS;
use crate::{FileId, Files, sql};

/// The largest side of a thumbnail, in pixels.
const THUMBNAIL_SIDE: u32 = 256;

/// The largest image a conversion decodes, in pixels per side.
const MAX_SIDE: u32 = 12_000;

/// The most memory decoding an image may take, in bytes.
const MAX_DECODED: u64 = 256 * 1024 * 1024;

/// Where a conversion of a file stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Conversion {
    /// What it converts into.
    pub kind: String,
    /// `queued`, `done` or `failed`.
    pub state: String,
    /// Why it failed.
    pub error: Option<String>,
    /// The media type of the result, once done.
    pub media_type: Option<String>,
    /// A link that reads the result for a few minutes, once done.
    pub url: Option<String>,
    /// When the link stops working.
    pub expires_at: Option<Timestamp>,
}

/// Whether content of a media type converts into a kind.
fn convertible(kind: &str, media_type: &str) -> bool {
    kind == "thumbnail"
        && matches!(
            media_type,
            "image/png" | "image/jpeg" | "image/gif" | "image/webp"
        )
}

/// A thumbnail of an image, as PNG, never larger than the image; decoding stays within limits.
fn thumbnail(content: &[u8]) -> std::result::Result<Vec<u8>, String> {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_DECODED);
    let mut reader = image::ImageReader::new(Cursor::new(content))
        .with_guessed_format()
        .map_err(|_| "unreadable image".to_owned())?;
    reader.limits(limits);
    let decoded = reader.decode().map_err(|e| format!("unreadable image: {e}"))?;
    let small = decoded.thumbnail(THUMBNAIL_SIDE, THUMBNAIL_SIDE);
    let mut out = Cursor::new(Vec::new());
    small
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| format!("unwritable thumbnail: {e}"))?;
    Ok(out.into_inner())
}

impl Files {
    /// A conversion of a file the principal may read: queued the first time it is asked for, and with a
    /// link to its result once done.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown file, one the principal may not read, or an unknown kind; `validation`
    /// (`files.conversion_unsupported`) for content that does not convert into the kind.
    pub async fn conversion(
        &self,
        principal: Option<uuid::Uuid>,
        id: FileId,
        kind: &str,
        services: &ServiceContext,
    ) -> Result<Conversion> {
        let found = self.file(principal, id, services).await?;
        let kind = CONVERSION_KINDS
            .iter()
            .find(|k| **k == kind)
            .ok_or_else(|| Error::not_found("files.conversion_unknown"))?;
        if !convertible(kind, &found.media_type) {
            return Err(Error::validation(
                "files.conversion_unsupported",
                vec![FieldError::new("kind", "media_type")],
            )
            .with_param("media_type", found.media_type));
        }
        let mut tx = self.db.begin().await?;
        let queued: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "insert into {} (id, source_blob_id, kind) select $1, f.blob_id, $3 from {} f where f.id = $2
             on conflict (source_blob_id, kind) do nothing returning id",
            self.table("conversion"),
            self.table("file")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(id.as_uuid())
        .bind(kind)
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(conversion) = queued {
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(
                    &mut tx,
                    "files.convert",
                    &json!({ "conversion": conversion }),
                    None,
                )
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        let row = sqlx::query(sql(format!(
            "select c.state, c.error, r.media_type from {} f join {} c on c.source_blob_id = f.blob_id
             left join {} r on r.id = c.result_blob_id where f.id = $1 and c.kind = $2",
            self.table("file"),
            self.table("conversion"),
            self.table("blob")
        )))
        .bind(id.as_uuid())
        .bind(kind)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let state: String = row.get("state");
        let link = (state == "done").then(|| self.link(id, Part::Conversion(kind)));
        Ok(Conversion {
            kind: (*kind).to_owned(),
            state,
            error: row.get("error"),
            media_type: row.get("media_type"),
            url: link.as_ref().map(|l| l.url.clone()),
            expires_at: link.map(|l| l.expires_at),
        })
    }

    /// Makes a queued conversion: its result is stored once by its hash and counted as a use of it. A
    /// failure is recorded and not retried, since the same content fails the same way.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database or storage fails; the job retries then.
    pub async fn convert(&self, conversion: uuid::Uuid) -> Result<()> {
        let Some(row) = sqlx::query(sql(format!(
            "select c.kind, c.state, b.storage_key, b.scan_status from {} c join {} b on b.id = c.source_blob_id
             where c.id = $1",
            self.table("conversion"),
            self.table("blob")
        )))
        .bind(conversion)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        else {
            return Ok(());
        };
        if row.get::<String, _>("state") != "queued" {
            return Ok(());
        }
        let key = kampus_storage::StorageKey::new(row.get::<String, _>("storage_key"))?;
        let content = self.storage()?.get(&key).await?;
        let result = match thumbnail(&content) {
            Ok(result) => result,
            Err(reason) => {
                sqlx::query(sql(format!(
                    "update {} set state = 'failed', error = $2 where id = $1",
                    self.table("conversion")
                )))
                .bind(conversion)
                .bind(reason)
                .execute(self.db.pool())
                .await
                .map_err(kampus_db::classify)?;
                return Ok(());
            }
        };
        let status: String = row.get("scan_status");
        let status = if status == "clean" { "clean" } else { "unscanned" };
        let blob = self
            .store(bytes::Bytes::from(result), "image/png", status)
            .await?;
        let mut tx = self.db.begin().await?;
        let done = sqlx::query(sql(format!(
            "update {} set state = 'done', result_blob_id = $2 where id = $1 and state = 'queued'",
            self.table("conversion")
        )))
        .bind(conversion)
        .bind(blob)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if done > 0 {
            self.use_blob_in(&mut tx, blob).await?;
            let mut event = kampus_events::Publish::new(
                crate::events::CONVERTED,
                Some(conversion),
                json!({ "conversion": conversion, "kind": row.get::<String, _>("kind"), "blob": blob }),
            );
            event.summary = Some(format!("conversion {conversion} done"));
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, event)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }
}
