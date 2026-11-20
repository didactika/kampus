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

//! Purging: uploads that expired, files nobody attached within a day, and content that stayed in the trash
//! past its retention, which is deleted from storage.

use kampus_common::Result;
use kampus_storage::StorageKey;
use sqlx::Row;

use crate::{FileId, Files, sql};

/// What a purge removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Purged {
    /// Uploads that expired.
    pub uploads: u64,
    /// Files nobody attached.
    pub files: u64,
    /// Contents deleted from storage.
    pub blobs: u64,
}

impl Files {
    /// Removes expired uploads, files nobody attached within a day, and content kept in the trash past its
    /// retention.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database or storage fails; what was removed before stays removed.
    pub async fn purge(&self) -> Result<Purged> {
        let uploads = sqlx::query(sql(format!(
            "delete from {} where expires_at < now() and file_id is null",
            self.table("upload")
        )))
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        let mut purged = Purged {
            uploads,
            ..Purged::default()
        };
        let stale: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select id from {} where not attached and created_at < now() - interval '1 day'",
            self.table("file")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        for id in stale {
            let mut tx = self.db.begin().await?;
            self.detach_in(&mut tx, FileId::from_uuid(id)).await?;
            tx.commit().await.map_err(kampus_db::classify)?;
            purged.files += 1;
        }
        let expired = sqlx::query(sql(format!(
            "select b.id, b.storage_key from {} t join {} b on b.id = t.blob_id
             where t.since < now() - interval '7 days' and b.ref_count = 0",
            self.table("trash"),
            self.table("blob")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        for row in &expired {
            let id: uuid::Uuid = row.get("id");
            let deleted = sqlx::query(sql(format!(
                "delete from {} where id = $1 and ref_count = 0",
                self.table("blob")
            )))
            .bind(id)
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
            if deleted > 0 {
                self.storage()?
                    .delete(&StorageKey::new(row.get::<String, _>("storage_key"))?)
                    .await?;
                purged.blobs += 1;
            }
        }
        Ok(purged)
    }
}
