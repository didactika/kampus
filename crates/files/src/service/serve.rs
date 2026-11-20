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

use crate::support::sign;
use crate::{File, FileId, Files, LINK_LIFETIME, SignedUrl, sql};

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
        let expires = time::OffsetDateTime::now_utc() + LINK_LIFETIME;
        let expires_at = Timestamp::from_offset_date_time(expires);
        let unix = expires.unix_timestamp();
        let signature = sign::signature(&self.signing_key, found.id.as_uuid(), unix);
        Ok(SignedUrl {
            url: format!(
                "{}/{}/content/{}?expires={unix}&signature={signature}",
                self.api_url.trim_end_matches('/'),
                crate::module::NAME,
                found.id
            ),
            expires_at,
        })
    }

    /// The content of a file behind a link that is valid and has not expired.
    ///
    /// # Errors
    ///
    /// `not_found` (`files.link_invalid`) for a link that was changed or expired, or a file that is gone;
    /// `unavailable` when storage fails.
    pub async fn content(&self, id: FileId, expires: i64, signature: &str) -> Result<(File, Bytes)> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        if expires < now || !sign::verify(&self.signing_key, id.as_uuid(), expires, signature) {
            return Err(Error::not_found("files.link_invalid"));
        }
        let found = self
            .stored(id)
            .await
            .map_err(|_| Error::not_found("files.link_invalid"))?;
        let key = self.key_of(id).await?;
        let data = self.storage()?.get(&key).await?;
        Ok((found, data))
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
}
