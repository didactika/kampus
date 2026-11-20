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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Profile pictures: a file of the `users.picture` area linked to its person through this module's own
//! table. Setting one attaches the new file and detaches the old one in the same transaction.

use kampus_access::Actor;
use kampus_common::{Error, Result};
use kampus_files::{FileId, Files};
use kampus_identity::AccountId;

use crate::settings::files::PICTURE_AREA;
use crate::{Users, sql};

impl Users {
    /// Files, or an error in a build without them.
    fn files(&self) -> Result<&Files> {
        self.files
            .as_ref()
            .ok_or_else(|| Error::internal("users.files_missing"))
    }

    /// The account of the person acting.
    fn own_account(actor: &Actor) -> Result<AccountId> {
        match actor {
            Actor::Principal(p) => Ok(*p),
            _ => Err(Error::permission_denied("users.not_a_person")),
        }
    }

    /// The profile picture of a person, if they have one.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn picture(&self, account: AccountId) -> Result<Option<FileId>> {
        let file: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select file_id from {} where account_id = $1",
            self.table("profile_picture")
        )))
        .bind(account.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(file.map(FileId::from_uuid))
    }

    /// Makes a file the person uploaded into the pictures area their profile picture, replacing the one
    /// they had.
    ///
    /// # Errors
    ///
    /// `permission_denied` for someone who is not a person, `not_found` for a file of someone else,
    /// `validation` (`files.area_mismatch`) for a file of another area.
    pub async fn set_picture(&self, actor: &Actor, file: FileId) -> Result<()> {
        let account = Self::own_account(actor)?;
        let files = self.files()?;
        let mut tx = self.db.begin().await?;
        let old: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select file_id from {} where account_id = $1 for update",
            self.table("profile_picture")
        )))
        .bind(account.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        files
            .attach_in(&mut tx, file, PICTURE_AREA, Some(account.as_uuid()))
            .await?;
        if let Some(old) = old {
            files.detach_in(&mut tx, FileId::from_uuid(old)).await?;
        }
        sqlx::query(sql(format!(
            "insert into {} (account_id, file_id) values ($1, $2)
             on conflict (account_id) do update set file_id = excluded.file_id",
            self.table("profile_picture")
        )))
        .bind(account.as_uuid())
        .bind(file.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Removes the profile picture of the person acting, which deletes its file.
    ///
    /// # Errors
    ///
    /// `permission_denied` for someone who is not a person, `not_found` (`users.picture_not_found`) when
    /// they have none.
    pub async fn remove_picture(&self, actor: &Actor) -> Result<()> {
        let account = Self::own_account(actor)?;
        let file = self
            .picture(account)
            .await?
            .ok_or_else(|| Error::not_found("users.picture_not_found"))?;
        let mut tx = self.db.begin().await?;
        self.files()?.detach_in(&mut tx, file).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
