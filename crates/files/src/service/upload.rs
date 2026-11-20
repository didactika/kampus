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

//! Uploads: the one pipeline every file goes through. An upload is opened for an area and a scope with the
//! permission the area asks for, its content is checked against the size and the types the area accepts
//! (read from the content, not from what the sender says) and stored once by its hash, and completing it
//! makes the file, which waits for a module to attach it to its owner.

use bytes::Bytes;
use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_storage::StorageKey;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::{COLUMNS, file, principal};
use crate::support::sniff;
use crate::{File, FileId, Files, Upload, UploadId, sql};

/// The longest file name, in characters.
const MAX_NAME: usize = 255;

/// The columns of an upload.
const UPLOAD_COLUMNS: &str =
    "id, area_code, scope_id, uploader, name, declared_type, max_size, blob_id, file_id, expires_at";

/// An upload from its row.
fn upload(row: &PgRow) -> Upload {
    Upload {
        id: UploadId::from_uuid(row.get("id")),
        area: row.get("area_code"),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        name: row.get("name"),
        max_size: row.get("max_size"),
        received: row.get::<Option<uuid::Uuid>, _>("blob_id").is_some(),
        file: row.get::<Option<uuid::Uuid>, _>("file_id").map(FileId::from_uuid),
        expires_at: Timestamp::from_offset_date_time(row.get("expires_at")),
    }
}

/// Whether a name can be a file name: not empty, not too long, on one line and without a path.
fn file_name(name: &str) -> Option<&str> {
    let name = name.trim();
    let fits = !name.is_empty()
        && name.chars().count() <= MAX_NAME
        && !name.chars().any(|c| c.is_control() || c == '/' || c == '\\');
    fits.then_some(name)
}

impl Files {
    /// Opens an upload of one file into an area, for a scope; needs the permission the area asks for there.
    ///
    /// # Errors
    ///
    /// `not_found` (`files.area_unknown`) for an area no module declares, `permission_denied` without the
    /// permission or for a visitor, `validation` (`files.upload_invalid`) for a name that is not one or a
    /// size above what the area accepts, `conflict` (`files.quota_exceeded`) when it would not fit a quota.
    pub async fn open_upload(
        &self,
        actor: &Actor,
        (area, scope): (&str, ScopeId),
        name: &str,
        size: i64,
        declared_type: Option<&str>,
    ) -> Result<Upload> {
        let row = sqlx::query(sql(format!(
            "select max_size, upload_permission from {} where code = $1 and retired_at is null",
            self.table("area")
        )))
        .bind(area)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.area_unknown").with_param("area", area.to_owned()))?;
        let permission: String = row.get("upload_permission");
        self.access.require(actor, &permission, scope).await?;
        let uploader = principal(actor).ok_or_else(|| Error::permission_denied("files.visitor"))?;
        let max_size: i64 = row.get("max_size");
        let mut problems = Vec::new();
        let name = file_name(name);
        if name.is_none() {
            problems.push(FieldError::new("name", "file_name"));
        }
        if size <= 0 || size > max_size {
            problems.push(FieldError::new("size", "max"));
        }
        let Some(name) = name.filter(|_| problems.is_empty()) else {
            return Err(Error::validation("files.upload_invalid", problems)
                .with_param("max_size", max_size.to_string()));
        };
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.require_room(&mut conn, scope, Some(uploader), size).await?;
        drop(conn);
        let row = sqlx::query(sql(format!(
            "insert into {} (id, area_code, scope_id, uploader, name, declared_type, max_size, expires_at)
             values ($1, $2, $3, $4, $5, $6, $7, now() + interval '1 hour') returning {UPLOAD_COLUMNS}",
            self.table("upload")
        )))
        .bind(UploadId::generate().as_uuid())
        .bind(area)
        .bind(scope.as_uuid())
        .bind(uploader)
        .bind(name)
        .bind(declared_type)
        .bind(size)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(upload(&row))
    }

    /// An upload of the actor's that still accepts work.
    async fn own_upload(&self, actor: &Actor, id: UploadId) -> Result<PgRow> {
        let row = sqlx::query(sql(format!(
            "select {UPLOAD_COLUMNS}, expires_at < now() as expired from {} where id = $1",
            self.table("upload")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .filter(|r| principal(actor) == Some(r.get("uploader")))
        .ok_or_else(|| Error::not_found("files.upload_not_found"))?;
        if row.get::<Option<uuid::Uuid>, _>("file_id").is_some() {
            return Err(Error::conflict("files.upload_completed"));
        }
        if row.get("expired") {
            return Err(Error::conflict("files.upload_expired"));
        }
        Ok(row)
    }

    /// Receives the content of an upload: checked against its size and the types of its area, read from
    /// the content, and stored once by its hash.
    ///
    /// # Errors
    ///
    /// `not_found` for an upload that is not the actor's, `conflict` (`files.upload_expired`,
    /// `files.upload_completed`), `validation` (`files.too_large`, `files.type_not_allowed`), and
    /// `unavailable` when storage fails.
    pub async fn send_upload(&self, actor: &Actor, id: UploadId, content: Bytes) -> Result<Upload> {
        let row = self.own_upload(actor, id).await?;
        let max_size: i64 = row.get("max_size");
        if i64::try_from(content.len()).unwrap_or(i64::MAX) > max_size {
            return Err(
                Error::validation("files.too_large", vec![FieldError::new("content", "max")])
                    .with_param("max_size", max_size.to_string()),
            );
        }
        let declared: Option<String> = row.get("declared_type");
        let media_type = sniff::media_type(&content, declared.as_deref());
        let allowed: Value = sqlx::query_scalar(sql(format!(
            "select allowed_types from {} where code = $1",
            self.table("area")
        )))
        .bind(row.get::<String, _>("area_code"))
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let allowed: Vec<String> = serde_json::from_value(allowed).unwrap_or_default();
        if !sniff::allowed(&allowed, &media_type) {
            return Err(Error::validation(
                "files.type_not_allowed",
                vec![FieldError::new("content", "type")],
            )
            .with_param("media_type", media_type));
        }
        let status = self.scan(&content).await?;
        let blob = self.store(content, &media_type, status).await?;
        let row = sqlx::query(sql(format!(
            "update {} set blob_id = $2 where id = $1 returning {UPLOAD_COLUMNS}",
            self.table("upload")
        )))
        .bind(id.as_uuid())
        .bind(blob)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(upload(&row))
    }

    /// The scan status of content: scanned by the scanner when one is configured, every time, since its
    /// signatures change; `unscanned` without one.
    async fn scan(&self, content: &Bytes) -> Result<&'static str> {
        let Some(scanner) = &self.scanner else {
            return Ok("unscanned");
        };
        match scanner.scan(content.clone()).await? {
            kampus_scan::Verdict::Clean => Ok("clean"),
            kampus_scan::Verdict::Infected(name) => Err(Error::validation(
                "files.infected",
                vec![FieldError::new("content", "infected")],
            )
            .with_param("signature", name)),
        }
    }

    /// The blob of some content: the one stored with its hash, whose scan status a fresh scan updates, or a
    /// new one in storage, which waits in the trash until a file uses it.
    async fn store(&self, content: Bytes, media_type: &str, status: &str) -> Result<uuid::Uuid> {
        let hash = kampus_storage::sha256(&content);
        let existing: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "update {} set scan_status = case when $2 = 'clean' then $2 else scan_status end
             where hash = $1 returning id",
            self.table("blob")
        )))
        .bind(hash.as_slice())
        .bind(status)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if let Some(blob) = existing {
            return Ok(blob);
        }
        let key = StorageKey::for_hash(&hash);
        let size = i64::try_from(content.len()).unwrap_or(i64::MAX);
        self.storage()?.put(&key, content, hash).await?;
        let mut tx = self.db.begin().await?;
        let blob: uuid::Uuid = sqlx::query_scalar(sql(format!(
            "insert into {t} (id, hash, size, media_type, storage_key, scan_status) values ($1, $2, $3, $4, $5, $6)
             on conflict (hash) do update set hash = excluded.hash returning id",
            t = self.table("blob")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(hash.as_slice())
        .bind(size)
        .bind(media_type)
        .bind(key.as_str())
        .bind(status)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {} (blob_id) select $1 where not exists (select 1 from {} where id = $1 and ref_count > 0)
             on conflict do nothing",
            self.table("trash"),
            self.table("blob")
        )))
        .bind(blob)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(blob)
    }

    /// Completes an upload whose content arrived: the file is made, with its content counted as used, and
    /// waits for a module to attach it to its owner.
    ///
    /// # Errors
    ///
    /// `not_found` for an upload that is not the actor's, `conflict` (`files.upload_empty`,
    /// `files.upload_expired`, `files.upload_completed`).
    pub async fn complete_upload(&self, actor: &Actor, id: UploadId) -> Result<File> {
        let row = self.own_upload(actor, id).await?;
        let blob: uuid::Uuid = row
            .get::<Option<uuid::Uuid>, _>("blob_id")
            .ok_or_else(|| Error::conflict("files.upload_empty"))?;
        let mut tx = self.db.begin().await?;
        let created = sqlx::query(sql(format!(
            "with used as (update {blobs} set ref_count = ref_count + 1 where id = $2 returning media_type, size)
             insert into {files} as f (id, blob_id, area_code, scope_id, name, media_type, size, uploader)
             select $1, $2, $3, $4, $5, used.media_type, used.size, $6 from used returning {COLUMNS}",
            blobs = self.table("blob"),
            files = self.table("file")
        )))
        .bind(FileId::generate().as_uuid())
        .bind(blob)
        .bind(row.get::<String, _>("area_code"))
        .bind(row.get::<uuid::Uuid, _>("scope_id"))
        .bind(row.get::<String, _>("name"))
        .bind(row.get::<uuid::Uuid, _>("uploader"))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let created = file(&created);
        sqlx::query(sql(format!(
            "delete from {} where blob_id = $1",
            self.table("trash")
        )))
        .bind(blob)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set file_id = $2 where id = $1",
            self.table("upload")
        )))
        .bind(id.as_uuid())
        .bind(created.id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let payload = json!({
            "file": created.id, "area": created.area, "size": created.size, "media_type": created.media_type,
        });
        self.publish_in(&mut tx, crate::events::UPLOADED, &created, payload)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }
}
