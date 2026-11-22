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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Blocks: a person stops another from writing to them, and stops writing to them too; nobody is told who
//! blocked whom.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result};

use super::person;
use crate::{Notify, sql};

impl Notify {
    /// Blocks an account for the actor.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor, `validation` (`notify.block_invalid`) for oneself, `not_found` for
    /// an unknown account.
    pub async fn block(&self, actor: &Actor, account: PrincipalId) -> Result<()> {
        let me = person(actor)?;
        if me == account {
            return Err(Error::validation(
                "notify.block_invalid",
                vec![FieldError::new("account", "other")],
            ));
        }
        sqlx::query(sql(format!(
            "insert into {} (account_id, blocked_id) values ($1, $2) on conflict do nothing",
            self.table("block")
        )))
        .bind(me.as_uuid())
        .bind(account.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(|e| match kampus_db::classify(e) {
            error if error.code() == "db.foreign_key_violation" => {
                Error::not_found("notify.account_not_found")
            }
            error => error,
        })?;
        Ok(())
    }

    /// Removes a block of the actor.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn unblock(&self, actor: &Actor, account: PrincipalId) -> Result<()> {
        let me = person(actor)?;
        sqlx::query(sql(format!(
            "delete from {} where account_id = $1 and blocked_id = $2",
            self.table("block")
        )))
        .bind(me.as_uuid())
        .bind(account.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    }

    /// The accounts the actor blocked.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn blocked(&self, actor: &Actor) -> Result<Vec<PrincipalId>> {
        let me = person(actor)?;
        let found: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select blocked_id from {} where account_id = $1 order by created_at",
            self.table("block")
        )))
        .bind(me.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(found.into_iter().map(PrincipalId::from_uuid).collect())
    }
}
