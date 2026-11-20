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

//! Serving: after the check that the requester may read a file, a link that reads it for a few minutes,
//! which the API serves behind a signature of the file and the moment the link stops working, with the
//! headers that keep content from running. Links signed by the store itself wait for a setting of its
//! public address, since they would skip those headers and point inside the deployment.

use bytes::Bytes;
use kampus_common::{Error, Result, Timestamp};
use kampus_module::ServiceContext;
use kampus_storage::StorageKey;
use sqlx::Row;

use crate::support::sign;
use crate::{File, FileId, Files, LINK_LIFETIME, SignedUrl, sql};

/// What a link reads: the original, or a conversion of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part<'a> {
    /// The file as uploaded.
    Original,
    /// A conversion of it, such as `thumbnail`.
    Conversion(&'a str),
}

impl Part<'_> {
    /// The name the signature binds; empty for the original.
    const fn name(&self) -> &str {
        match self {
            Self::Original => "",
            Self::Conversion(kind) => kind,
        }
    }
}

impl Files {
    /// A link that reads a file for a few minutes, for a principal who may read it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown file or one the principal may not read, `unavailable` when storage fails.
    pub async fn signed_url(
        &self,
        principal: Option<uuid::Uuid>,
        id: FileId,
        services: &ServiceContext,
    ) -> Result<SignedUrl> {
        let found = self.file(principal, id, services).await?;
        Ok(self.link(found.id, Part::Original))
    }

    /// A link to a part of a file, signed for a few minutes.
    pub(crate) fn link(&self, id: FileId, part: Part<'_>) -> SignedUrl {
        let expires = time::OffsetDateTime::now_utc() + LINK_LIFETIME;
        let unix = expires.unix_timestamp();
        let signature = sign::signature(&self.signing_key, id.as_uuid(), part.name(), unix);
        let kind = match part {
            Part::Original => String::new(),
            Part::Conversion(kind) => format!("&kind={kind}"),
        };
        SignedUrl {
            url: format!(
                "{}/{}/content/{id}?expires={unix}&signature={signature}{kind}",
                self.api_url.trim_end_matches('/'),
                crate::module::NAME,
            ),
            expires_at: Timestamp::from_offset_date_time(expires),
        }
    }

    /// The content behind a link that is valid and has not expired: the file, its media type and its
    /// content, or those of the conversion the link names.
    ///
    /// # Errors
    ///
    /// `not_found` (`files.link_invalid`) for a link that was changed or expired, or a file or conversion
    /// that is gone; `unavailable` when storage fails.
    pub async fn content(
        &self,
        id: FileId,
        part: Part<'_>,
        expires: i64,
        signature: &str,
    ) -> Result<(File, String, Bytes)> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        if expires < now || !sign::verify(&self.signing_key, id.as_uuid(), part.name(), expires, signature) {
            return Err(Error::not_found("files.link_invalid"));
        }
        let invalid = |_| Error::not_found("files.link_invalid");
        let found = self.stored(id).await.map_err(invalid)?;
        let (key, media_type) = match part {
            Part::Original => (self.key_of(id).await?, found.media_type.clone()),
            Part::Conversion(kind) => self.converted_key(id, kind).await.map_err(invalid)?,
        };
        let data = self.storage()?.get(&key).await?;
        Ok((found, media_type, data))
    }

    /// The storage key of the content of a file.
    async fn key_of(&self, id: FileId) -> Result<StorageKey> {
        let key: String = sqlx::query_scalar(sql(format!(
            "select b.storage_key from {} f join {} b on b.id = f.blob_id where f.id = $1",
            self.table("file"),
            self.table("blob")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.file_not_found"))?;
        StorageKey::new(key)
    }

    /// The storage key and media type of a finished conversion of a file.
    async fn converted_key(&self, id: FileId, kind: &str) -> Result<(StorageKey, String)> {
        let row = sqlx::query(sql(format!(
            "select r.storage_key, r.media_type from {} f join {} c on c.source_blob_id = f.blob_id
             join {} r on r.id = c.result_blob_id where f.id = $1 and c.kind = $2 and c.state = 'done'",
            self.table("file"),
            self.table("conversion"),
            self.table("blob")
        )))
        .bind(id.as_uuid())
        .bind(kind)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("files.conversion_not_found"))?;
        Ok((
            StorageKey::new(row.get::<String, _>("storage_key"))?,
            row.get("media_type"),
        ))
    }
}
