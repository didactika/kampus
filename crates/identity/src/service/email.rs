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

//! Changing the email of an account: a sensitive operation that needs a recent strong authentication;
//! the new address is used only once its owner opens the link sent to it, and the current address is
//! told.

use kampus_common::{Error, FieldError, Result};

use crate::service::account::normalise_email;
use crate::service::one_time::Purpose;
use crate::{AccountId, Identity, Message, Notice, Session, sql};

impl Identity {
    /// Asks to change the session's email: the new address receives a link, the current one is told.
    ///
    /// # Errors
    ///
    /// `permission_denied` (`identity.step_up_required`) without a recent strong authentication,
    /// `validation` for an invalid email, `conflict` (`identity.email_taken`) when another account of the
    /// campus has it.
    pub async fn change_email(&self, session: &Session, email: &str) -> Result<()> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let email = normalise_email(email)?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, session.account).await?;
        if account.email.as_deref() == Some(email.as_str()) {
            return Err(Error::validation(
                "identity.email_unchanged",
                vec![FieldError::new("email", "different")],
            ));
        }
        let in_use: bool = sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where campus_id = $1 and email = $2)",
            self.table("account")
        )))
        .bind(account.campus.as_uuid())
        .bind(&email)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if in_use {
            return Err(Error::conflict("identity.email_taken"));
        }
        sqlx::query(sql(format!(
            "update {} set pending_email = $2 where id = $1",
            self.table("account")
        )))
        .bind(account.id.as_uuid())
        .bind(&email)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let token = self
            .issue_in(
                &mut tx,
                account.id,
                Purpose::EmailChange,
                policy.confirmation_lifetime,
            )
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.deliver(Message {
            campus: account.campus,
            account: account.id,
            email,
            notice: Notice::ConfirmNewEmail { token },
        })
        .await;
        self.tell_owner(&account, Notice::EmailChangeRequested).await;
        Ok(())
    }

    /// Confirms a new email with the link sent to it. A username that was the old email follows it.
    ///
    /// # Errors
    ///
    /// `validation` (`identity.token_invalid`) for an unknown, used or expired link, `conflict` when the
    /// address was taken meanwhile.
    pub async fn confirm_email_change(&self, token: &str) -> Result<AccountId> {
        let mut tx = self.db.begin().await?;
        let id = self.consume_in(&mut tx, Purpose::EmailChange, token).await?;
        let account = self.lock_account_in(&mut tx, id).await?;
        let Some(email) = account.pending_email.clone() else {
            return Err(crate::service::one_time::invalid_token());
        };
        let follows = account.username == account.email;
        sqlx::query(sql(format!(
            "update {} set email = $2, username = case when $3 then $2 else username end, email_verified_at = $4,
             pending_email = null, version = version + 1 where id = $1",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .bind(&email)
        .bind(follows)
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(|error| {
            if error.code() == "db.unique_violation" {
                Error::conflict("identity.email_taken")
            } else {
                error
            }
        })?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }
}
