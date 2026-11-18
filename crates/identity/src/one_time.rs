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

//! One-time tokens (RN-026): random, stored only as hashes, expiring and used once. Issuing a token
//! replaces the unused tokens of the same purpose, so only the latest link works.

use kampus_common::{Error, FieldError, Result};
use sqlx::{PgConnection, Row};

use crate::secret::{new_token, token_hash};
use crate::{AccountId, Identity, later, sql};

/// What a one-time token is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// Confirms the email of a new account.
    ConfirmEmail,
    /// Accepts an invitation and sets the password.
    Invitation,
    /// Sets a password asked by a manager.
    SetPassword,
    /// Recovers access (RN-030).
    Recovery,
    /// A code sent by email as a second factor.
    EmailCode,
    /// A sign-in that waits for its second factor.
    SecondFactor,
}

impl Purpose {
    /// The purpose as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConfirmEmail => "confirm_email",
            Self::Invitation => "invitation",
            Self::SetPassword => "set_password",
            Self::Recovery => "recovery",
            Self::EmailCode => "email_code",
            Self::SecondFactor => "second_factor",
        }
    }
}

/// The error of a token that is unknown, used, expired or of another purpose: one answer for all.
pub(crate) fn invalid_token() -> Error {
    Error::validation("identity.token_invalid", vec![FieldError::new("token", "valid")])
}

impl Identity {
    /// Issues a token for an account and returns it; only its hash is stored.
    pub(crate) async fn issue_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
        purpose: Purpose,
        lifetime: time::Duration,
    ) -> Result<String> {
        let table = self.table("one_time_token");
        sqlx::query(sql(format!(
            "delete from {table} where account_id = $1 and purpose = $2 and used_at is null"
        )))
        .bind(account.as_uuid())
        .bind(purpose.as_str())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let token = new_token()?;
        let now = self.now();
        sqlx::query(sql(format!(
            "insert into {table} (id, account_id, purpose, hash, created_at, expires_at) values ($1, $2, $3, $4, $5, $6)"
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(account.as_uuid())
        .bind(purpose.as_str())
        .bind(token_hash(&token))
        .bind(now.as_offset_date_time())
        .bind(later(now, lifetime).as_offset_date_time())
        .execute(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(token)
    }

    /// Uses a token once and returns its account.
    ///
    /// # Errors
    ///
    /// `validation` (`identity.token_invalid`) when the token is unknown, used, expired or of another
    /// purpose.
    pub(crate) async fn consume_in(
        &self,
        conn: &mut PgConnection,
        purpose: Purpose,
        token: &str,
    ) -> Result<AccountId> {
        let now = self.now().as_offset_date_time();
        let row = sqlx::query(sql(format!(
            "update {} set used_at = $3 where hash = $1 and purpose = $2 and used_at is null and expires_at > $3
             returning account_id",
            self.table("one_time_token")
        )))
        .bind(token_hash(token))
        .bind(purpose.as_str())
        .bind(now)
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(invalid_token)?;
        Ok(AccountId::from_uuid(row.get("account_id")))
    }
}
