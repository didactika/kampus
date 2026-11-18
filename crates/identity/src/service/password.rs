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

//! Passwords (RN-025): chosen by the person, checked against the campus policy at every change, hashed
//! with Argon2id; never generated, sent or shown. Changing one revokes every session and token (RN-031).

use kampus_common::{Error, FieldError, Result};
use sqlx::PgConnection;

use crate::{Account, AccountKind, CampusPolicy, Identity, Message, Notice, Session, sql};

/// The longest password accepted, so hashing cannot be used to exhaust the server.
pub const MAX_LENGTH: usize = 1024;

/// Checks a new password against the policy and the account's own names.
///
/// # Errors
///
/// `validation` with every rule the password breaks.
pub fn validate(policy: &CampusPolicy, account: &Account, password: &str) -> Result<()> {
    let mut fields = Vec::new();
    let length = password.chars().count();
    if length < policy.password_min_length {
        fields.push(FieldError::new("password", "min_length"));
    }
    if length > MAX_LENGTH {
        fields.push(FieldError::new("password", "max_length"));
    }
    let lower = password.to_lowercase();
    let names = [
        account.username.as_deref(),
        account.email.as_deref().and_then(|email| email.split('@').next()),
    ];
    if names
        .into_iter()
        .flatten()
        .any(|name| name.chars().count() >= 3 && lower.contains(name))
    {
        fields.push(FieldError::new("password", "contains_name"));
    }
    if fields.is_empty() {
        Ok(())
    } else {
        Err(Error::validation("identity.password_rejected", fields)
            .with_param("min_length", policy.password_min_length.to_string()))
    }
}

impl Identity {
    /// Sets the password of an account after the policy check, replacing any previous one.
    pub(crate) async fn set_password_in(
        &self,
        conn: &mut PgConnection,
        account: &Account,
        policy: &CampusPolicy,
        password: &str,
    ) -> Result<()> {
        if account.kind == AccountKind::Service {
            return Err(Error::validation(
                "identity.service_account_has_no_password",
                vec![FieldError::new("password", "not_allowed")],
            ));
        }
        validate(policy, account, password)?;
        let hash = self.hasher.hash(password).await?;
        sqlx::query(sql(format!(
            "insert into {} (account_id, hash, algorithm_params, set_at, must_change) values ($1, $2, $3, $4, false)
             on conflict (account_id) do update set hash = excluded.hash, algorithm_params = excluded.algorithm_params,
             set_at = excluded.set_at, must_change = false",
            self.table("password")
        )))
        .bind(account.id.as_uuid())
        .bind(hash)
        .bind(self.hasher.params())
        .bind(self.now().as_offset_date_time())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// The stored password hash of an account, if it has one.
    pub(crate) async fn password_hash_in(
        &self,
        conn: &mut PgConnection,
        account: crate::AccountId,
    ) -> Result<Option<String>> {
        sqlx::query_scalar(sql(format!(
            "select hash from {} where account_id = $1",
            self.table("password")
        )))
        .bind(account.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)
    }

    /// Changes the password of the session's account: needs a recent strong authentication (RN-032) and
    /// the current password; every session and token of the account ends (RN-031), this one included.
    ///
    /// # Errors
    ///
    /// `permission_denied` (`identity.step_up_required`) without a recent strong authentication,
    /// `unauthenticated` for a wrong current password, `validation` for a rejected new one.
    pub async fn change_password(&self, session: &Session, current: &str, new: &str) -> Result<()> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, session.account).await?;
        let stored = self.password_hash_in(&mut tx, account.id).await?;
        if !self.hasher.verify(current, stored.as_deref()).await? {
            return Err(Error::unauthenticated("identity.password_wrong"));
        }
        self.set_password_in(&mut tx, &account, &policy, new).await?;
        self.revoke_everything_in(&mut tx, account.id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        if let Some(email) = account.email.clone() {
            self.deliver(Message {
                campus: account.campus,
                account: account.id,
                email,
                notice: Notice::PasswordChanged,
            })
            .await;
        }
        Ok(())
    }
}
