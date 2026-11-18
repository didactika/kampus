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

//! The sign-in pipeline (UC-007, RN-024): every method identifies the account, checks throttling and the
//! status, verifies the credential, asks for the second factor when required, then opens the session.
//! Each method is one kind of [`Credential`]; none can skip a step.
//!
//! Failures give one answer whatever went wrong, and unknown names are throttled like real ones (RN-027).

use kampus_access::ScopeId;
use kampus_common::{Error, ErrorKind, Result};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::one_time::Purpose;
use crate::session::{Client, NewSession};
use crate::{Account, AccountKind, AccountStatus, CampusPolicy, Identity, MfaLevel, sql};

/// How long a sign-in waits for its second factor.
const SECOND_FACTOR_LIFETIME: time::Duration = time::Duration::minutes(5);

/// The longest wait of the slow-down before the lock.
const LONGEST_WAIT: i64 = 15 * 60;

/// What a person proves themselves with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    /// A password (RN-025).
    Password(String),
}

/// A sign-in request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignIn {
    /// The username or email.
    pub name: String,
    /// The credential.
    pub credential: Credential,
    /// Where the request comes from.
    pub client: Client,
}

/// The result of a sign-in that passed the credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignInOutcome {
    /// A session is open.
    SignedIn(NewSession),
    /// The second factor is required (RN-029); the challenge continues the sign-in.
    SecondFactorRequired {
        /// The one-time token of the pending sign-in.
        challenge: String,
    },
}

/// The outcome of an attempt, as recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// Signed in.
    Success,
    /// The credential, the account or its status failed.
    Failure,
    /// Refused by throttling before the credential was looked at.
    Throttled,
    /// The credential passed; the second factor is pending.
    SecondFactorRequired,
}

impl Outcome {
    /// The outcome as stored.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Throttled => "throttled",
            Self::SecondFactorRequired => "second_factor_required",
        }
    }
}

/// The hash a tried name is kept as, so the log holds no names.
fn name_hash(campus: ScopeId, name: &str) -> Vec<u8> {
    Sha256::digest(format!("{campus}:{name}").as_bytes()).to_vec()
}

/// The answer to every failed sign-in.
fn failed() -> Error {
    Error::unauthenticated("identity.sign_in_failed")
}

impl Identity {
    /// Signs in to a campus through the pipeline.
    ///
    /// # Errors
    ///
    /// `rate_limited` (`identity.sign_in_throttled`, with `retry_after` in seconds) while throttled,
    /// `unauthenticated` (`identity.sign_in_failed`) for any failure.
    pub async fn sign_in(&self, campus: ScopeId, request: SignIn) -> Result<SignInOutcome> {
        let policy = self.policies.policy(campus).await?;
        let name = request.name.trim().to_lowercase();
        let hash = name_hash(campus, &name);
        let address = request.client.address.as_deref();

        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let account = self.find_in(&mut conn, campus, &name).await?;
        drop(conn);
        let id = account.as_ref().map(|a| a.id);

        if let Some(retry_after) = self.throttled(campus, &policy, id, &hash, address).await? {
            self.record_attempt(campus, id, &hash, address, Outcome::Throttled)
                .await?;
            return Err(Error::new(ErrorKind::RateLimited, "identity.sign_in_throttled")
                .with_param("retry_after", retry_after.to_string()));
        }

        let Some(account) = self.verify(account, &request.credential).await? else {
            self.record_attempt(campus, id, &hash, address, Outcome::Failure)
                .await?;
            return Err(failed());
        };

        if self.second_factor_required(&account, &policy).await? {
            let mut tx = self.db.begin().await?;
            let challenge = self
                .issue_in(&mut tx, account.id, Purpose::SecondFactor, SECOND_FACTOR_LIFETIME)
                .await?;
            tx.commit().await.map_err(kampus_db::classify)?;
            self.record_attempt(campus, id, &hash, address, Outcome::SecondFactorRequired)
                .await?;
            return Ok(SignInOutcome::SecondFactorRequired { challenge });
        }

        let session = self
            .open(&account, MfaLevel::None, &request.client, &policy)
            .await?;
        self.record_attempt(campus, id, &hash, address, Outcome::Success)
            .await?;
        Ok(SignInOutcome::SignedIn(session))
    }

    /// Verifies the credential; the account passes only when it is an active person and the credential
    /// matches. The work is the same when there is no account, so timing tells nothing.
    async fn verify(&self, account: Option<Account>, credential: &Credential) -> Result<Option<Account>> {
        match credential {
            Credential::Password(password) => {
                let stored = match &account {
                    Some(account) => {
                        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
                        self.password_hash_in(&mut conn, account.id).await?
                    }
                    None => None,
                };
                let matches = self.hasher.verify(password, stored.as_deref()).await?;
                Ok(account.filter(|a| {
                    matches && a.status == AccountStatus::Active && a.kind == AccountKind::Person
                }))
            }
        }
    }

    /// Whether the account must give a second factor: it set one up, or the campus requires one.
    async fn second_factor_required(&self, account: &Account, policy: &CampusPolicy) -> Result<bool> {
        if policy.second_factor_required {
            return Ok(true);
        }
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where account_id = $1)",
            self.table("second_factor")
        )))
        .bind(account.id.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    /// Opens the session at the end of the pipeline and stamps the account's last sign-in.
    pub(crate) async fn open(
        &self,
        account: &Account,
        mfa_level: MfaLevel,
        client: &Client,
        policy: &CampusPolicy,
    ) -> Result<NewSession> {
        let mut tx = self.db.begin().await?;
        let session = self
            .create_session_in(&mut tx, account.id, account.campus, mfa_level, client, policy)
            .await?;
        sqlx::query(sql(format!(
            "update {} set last_sign_in_at = $2 where id = $1",
            self.table("account")
        )))
        .bind(account.id.as_uuid())
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(session)
    }

    /// The seconds to wait before trying again, when throttled (RN-027): failures since the last success
    /// first slow down, doubling the wait, then lock; an address with too many failures locks too.
    async fn throttled(
        &self,
        campus: ScopeId,
        policy: &CampusPolicy,
        account: Option<crate::AccountId>,
        hash: &[u8],
        address: Option<&str>,
    ) -> Result<Option<i64>> {
        let table = self.table("sign_in_attempt");
        let now = self.now().as_offset_date_time();
        let since = now - policy.throttle.window;
        let row = sqlx::query(sql(format!(
            "select count(*) as failures, max(at) as last from {table} f
             where f.campus_id = $1 and f.outcome = 'failure' and f.at > $4
             and (case when $2::uuid is null then f.account_id is null and f.name_tried_hash = $3 else f.account_id = $2 end)
             and f.at > coalesce((select max(s.at) from {table} s where s.account_id = $2 and s.outcome = 'success'), '-infinity')"
        )))
        .bind(campus.as_uuid())
        .bind(account.map(|a| a.as_uuid()))
        .bind(hash)
        .bind(since)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let failures = u32::try_from(row.get::<i64, _>("failures")).unwrap_or(u32::MAX);
        let last: Option<time::OffsetDateTime> = row.get("last");
        let throttle = policy.throttle;
        let mut wait = match last {
            Some(last) if failures >= throttle.lock_after => {
                (throttle.lock_for - (now - last)).whole_seconds()
            }
            Some(last) if failures >= throttle.slow_after => {
                let exponent = (failures - throttle.slow_after).min(20);
                let delay = (1_i64 << exponent).min(LONGEST_WAIT);
                delay - (now - last).whole_seconds()
            }
            _ => 0,
        };
        if let Some(address) = address {
            let row = sqlx::query(sql(format!(
                "select count(*) as failures, max(at) as last from {table}
                 where campus_id = $1 and address = $2 and outcome = 'failure' and at > $3"
            )))
            .bind(campus.as_uuid())
            .bind(address)
            .bind(since)
            .fetch_one(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            let failures = u32::try_from(row.get::<i64, _>("failures")).unwrap_or(u32::MAX);
            if let (true, Some(last)) = (
                failures >= throttle.address_lock_after,
                row.get::<Option<time::OffsetDateTime>, _>("last"),
            ) {
                wait = wait.max((throttle.lock_for - (now - last)).whole_seconds());
            }
        }
        Ok((wait > 0).then_some(wait))
    }

    /// Records an attempt, for throttling and the account's security log.
    async fn record_attempt(
        &self,
        campus: ScopeId,
        account: Option<crate::AccountId>,
        hash: &[u8],
        address: Option<&str>,
        outcome: Outcome,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, account_id, name_tried_hash, address, outcome, at) values ($1, $2, $3, $4, $5, $6, $7)",
            self.table("sign_in_attempt")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(campus.as_uuid())
        .bind(account.map(|a| a.as_uuid()))
        .bind(hash)
        .bind(address)
        .bind(outcome.as_str())
        .bind(self.now().as_offset_date_time())
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }
}
