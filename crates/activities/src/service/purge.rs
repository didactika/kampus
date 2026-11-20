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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Purging: course modules whose time in the recycle bin ended are removed for good with their files; one
//! that cannot be removed keeps its place in the bin with the reason, so teachers see it, and the next run
//! tries again.

use kampus_common::Result;
use kampus_files::FileId;
use sqlx::Row;

use crate::{Activities, ModuleId, sql};

/// What a purge did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Purged {
    /// Course modules removed for good.
    pub removed: u64,
    /// Course modules that could not be removed, and wait with their reason.
    pub failed: u64,
}

impl Activities {
    /// Removes the course modules whose time in the recycle bin ended.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails while listing; a course module that fails is recorded, not
    /// returned.
    pub async fn purge(&self) -> Result<Purged> {
        let due: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select course_module_id from {} where expires_at < now() order by expires_at limit 200",
            self.table("bin_item")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut purged = Purged::default();
        for id in due {
            match self.remove_for_good(ModuleId::from_uuid(id)).await {
                Ok(()) => purged.removed += 1,
                Err(error) => {
                    purged.failed += 1;
                    sqlx::query(sql(format!(
                        "update {} set error = $2 where course_module_id = $1",
                        self.table("bin_item")
                    )))
                    .bind(id)
                    .bind(error.code())
                    .execute(self.db.pool())
                    .await
                    .map_err(kampus_db::classify)?;
                }
            }
        }
        Ok(purged)
    }

    /// Removes a course module with its scope and its file, in one transaction.
    async fn remove_for_good(&self, id: ModuleId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let module = self.stored_in(&mut tx, id, true).await?;
        let file: Option<uuid::Uuid> = sqlx::query(sql(format!(
            "select file_id from {} where course_module_id = $1",
            self.table("module_file")
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .map(|r| r.get("file_id"));
        if let Some(file) = file {
            self.files.detach_in(&mut tx, FileId::from_uuid(file)).await?;
        }
        self.publish_in(&mut tx, crate::events::MODULE_PURGED, &module, None)
            .await?;
        self.access.delete_scope_in(&mut tx, module.scope).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
