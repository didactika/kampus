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

//! Recovering access: a link by email, then the second factor or a recovery code when
//! the account has one, then a new password. Recovery is rate-limited, ends every session and token,
//! tells the owner, and never signs the person in by itself.

use kampus_access::ScopeId;
use kampus_common::{Error, Result};

use crate::service::account::normalise_email;
use crate::service::one_time::Purpose;
use crate::service::second_factor::Proof;
use crate::{AccountKind, AccountStatus, FactorKind, Identity, Message, Notice};

/// How many recovery links an account receives per hour at most.
const LINKS_PER_HOUR: i64 = 3;

/// What completing a recovery needs, so the client asks for it before the new password.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct RecoveryNeeds {
    /// The kinds of second factor that may be given; empty when the link alone is enough. Recovery codes
    /// work whenever this is not empty; email codes never do, since the link came by email.
    pub methods: Vec<FactorKind>,
}

impl Identity {
    /// Starts a recovery: when the email has an active account, it receives a link. The answer is the
    /// same whatever happens, and an account receives a few links per hour at most.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn start_recovery(&self, campus: ScopeId, email: &str) -> Result<()> {
        let Ok(email) = normalise_email(email) else {
            return Ok(());
        };
        let policy = self.policies.policy(campus).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let account = self.find_in(&mut conn, campus, &email).await?;
        drop(conn);
        let Some(account) = account.filter(|a| {
            a.status == AccountStatus::Active
                && a.kind == AccountKind::Person
                && a.email.as_deref() == Some(&email)
        }) else {
            return Ok(());
        };
        let hour_ago = self.now().as_offset_date_time() - time::Duration::hours(1);
        if self.issued_since(account.id, Purpose::Recovery, hour_ago).await? >= LINKS_PER_HOUR {
            return Ok(());
        }
        let mut tx = self.db.begin().await?;
        let token = self
            .issue_alongside_in(&mut tx, account.id, Purpose::Recovery, policy.recovery_lifetime)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.deliver(Message {
            campus,
            account: account.id,
            email,
            notice: Notice::Recovery { token },
        })
        .await;
        Ok(())
    }

    /// What completing a recovery with this link needs.
    ///
    /// # Errors
    ///
    /// `validation` (`identity.token_invalid`) for an unknown, used or expired link.
    pub async fn recovery_needs(&self, token: &str) -> Result<RecoveryNeeds> {
        let account = self.peek(Purpose::Recovery, token).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let methods = self
            .factor_kinds_in(&mut conn, account)
            .await?
            .into_iter()
            .filter(|k| *k != FactorKind::Email)
            .collect();
        Ok(RecoveryNeeds { methods })
    }

    /// Completes a recovery: the second factor is checked before anything changes, then the password is
    /// set and every session and token ends. The person then signs in with the new password.
    ///
    /// # Errors
    ///
    /// `validation` for an invalid link or a rejected password, `unauthenticated`
    /// (`identity.second_factor_failed`) when the account has a second factor and the proof is missing or
    /// wrong; the link is then used up.
    pub async fn complete_recovery(&self, token: &str, proof: Option<Proof>, password: &str) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let id = self.consume_in(&mut tx, Purpose::Recovery, token).await?;
        let account = self.lock_account_in(&mut tx, id).await?;
        if account.status != AccountStatus::Active {
            return Err(crate::service::one_time::invalid_token());
        }
        let policy = self.policies.policy(account.campus).await?;
        if !self.factor_kinds_in(&mut tx, id).await?.is_empty() {
            let passed = match &proof {
                Some(proof @ (Proof::Totp(_) | Proof::RecoveryCode(_))) => self
                    .check_proof_in(&mut tx, &account, proof, false)
                    .await?
                    .is_some(),
                _ => false,
            };
            if !passed {
                tx.commit().await.map_err(kampus_db::classify)?;
                self.record_failure(&account, None).await?;
                return Err(Error::unauthenticated("identity.second_factor_failed"));
            }
        }
        self.set_password_in(&mut tx, &account, &policy, password).await?;
        self.revoke_everything_in(&mut tx, id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.tell_owner(&account, Notice::RecoveryUsed).await;
        Ok(())
    }
}
