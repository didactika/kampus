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

//! Moving content between stores: every blob is copied to another store and verified by its hash, so the
//! operator can switch the store the installation uses without stopping it, and then delete the old one.
//! Keys are the hashes, so the copy keeps them, and copying again skips what is already there.

use kampus_common::{Error, Result};
use kampus_storage::{Storage, StorageKey};
use sqlx::Row;

use crate::{Files, sql};

/// How many blobs a copy reads at once.
const PAGE: i64 = 500;

/// What a copy did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Copied {
    /// Blobs copied.
    pub copied: u64,
    /// Blobs the other store already had.
    pub present: u64,
}

impl Files {
    /// Copies every blob to another store, verifying each by its hash; blobs it already has with the same
    /// size are left as they are.
    ///
    /// # Errors
    ///
    /// `unavailable` when either store fails, and `conflict` (`files.blob_damaged`) when stored content
    /// does not match its hash; what was copied before stays copied, so a new run goes on.
    pub async fn copy_to(&self, target: &dyn Storage) -> Result<Copied> {
        let source = self.storage()?;
        let mut copied = Copied::default();
        let mut after = uuid::Uuid::nil();
        loop {
            let rows = sqlx::query(sql(format!(
                "select id, hash, size, storage_key from {} where id > $1 order by id limit $2",
                self.table("blob")
            )))
            .bind(after)
            .bind(PAGE)
            .fetch_all(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            let Some(last) = rows.last() else { break };
            after = last.get("id");
            for row in &rows {
                let key = StorageKey::new(row.get::<String, _>("storage_key"))?;
                let size = u64::try_from(row.get::<i64, _>("size")).unwrap_or(0);
                if target.head(&key).await?.is_some_and(|o| o.size == size) {
                    copied.present += 1;
                    continue;
                }
                let hash: Vec<u8> = row.get("hash");
                let hash: [u8; 32] = hash
                    .try_into()
                    .map_err(|_| Error::conflict("files.blob_damaged"))?;
                let data = source.get(&key).await?;
                kampus_storage::verify(&key, &data, &hash)
                    .map_err(|_| Error::conflict("files.blob_damaged").with_param("key", key.to_string()))?;
                target.put(&key, data, hash).await?;
                copied.copied += 1;
            }
        }
        Ok(copied)
    }
}
