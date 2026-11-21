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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Deletes the accounts whose grace period ended: their personal data is erased and the
//! row stays as a tombstone with a pseudonym, so academic records keep a valid reference. Each module
//! erases its own data through its privacy handler; this job erases what identity holds.

use kampus_common::Result;
use sqlx::PgConnection;

use crate::support::secret::random_hex;
use crate::{AccountId, AccountStatus, Identity, sql};

/// How many accounts one run deletes, so a run stays short.
const BATCH: i64 = 100;

/// The tables of identity that hold an account's personal data, erased with it.
const PERSONAL: &[&str] = &[
    "password",
    "second_factor",
    "passkey",
    "recovery_code",
    "one_time_token",
    "session",
    "api_token",
    "external_identity",
    "sign_in_attempt",
];

impl Identity {
    /// Deletes the accounts due, at most a batch per run, and returns how many it deleted. Rows locked by
    /// another run are skipped, so runs can overlap.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; the accounts deleted before stay deleted.
    pub async fn run_due_deletions(&self) -> Result<usize> {
        let mut tx = self.db.begin().await?;
        let due: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select id from {} where status = 'suspended' and deletion_due_at <= $1
             order by deletion_due_at limit $2 for update skip locked",
            self.table("account")
        )))
        .bind(self.now().as_offset_date_time())
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for id in &due {
            self.erase_in(&mut tx, AccountId::from_uuid(*id)).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(due.len())
    }

    /// Erases the personal data of an account now and leaves its tombstone, without waiting for the grace
    /// period: the deletion was approved through a privacy request.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn erase_account(&self, id: AccountId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        self.erase_in(&mut tx, id).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Erases an account's personal data and leaves its tombstone.
    async fn erase_in(&self, conn: &mut PgConnection, id: AccountId) -> Result<()> {
        for table in PERSONAL {
            sqlx::query(sql(format!(
                "delete from {} where account_id = $1",
                self.table(table)
            )))
            .bind(id.as_uuid())
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        let pseudonym = format!("deleted-{}", random_hex::<6>()?);
        let now = self.now().as_offset_date_time();
        sqlx::query(sql(format!(
            "update {} set username = null, email = null, email_verified_at = null, last_sign_in_at = null,
             status = 'deleted', status_reason = null, status_changed_at = $2, status_changed_by = null,
             pseudonym = $3, deletion_due_at = null, deleted_at = $2, version = version + 1 where id = $1",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .bind(now)
        .bind(pseudonym)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set reason = null where account_id = $1",
            self.table("account_transition")
        )))
        .bind(id.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        self.record_transition_in(
            conn,
            id,
            Some(AccountStatus::Suspended),
            AccountStatus::Deleted,
            Some("grace period ended"),
            None,
        )
        .await?;
        self.access.set_principal_active_in(conn, id, false).await
    }
}
