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

//! Second factors: codes from an authenticator app, codes by email, and recovery codes
//! shown once and stored as hashes. Adding or removing a factor needs a recent strong authentication.
//! A sign-in that needs a second factor ends here, with a session of the higher level.

use kampus_common::{Error, FieldError, Result, Timestamp};
use sqlx::{PgConnection, Row};

use crate::service::one_time::Purpose;
use crate::service::session::{Client, NewSession};
use crate::support::secret::{random_bytes, token_hash};
use crate::support::totp;
use crate::{
    Account, AccountId, AccountKind, AccountStatus, CampusPolicy, FactorId, FactorKind, Identity, Message,
    MfaLevel, Notice, SecondFactor, Session, sql,
};

/// The issuer authenticator apps show next to the code.
const ISSUER: &str = "Kampus";

/// How many recovery codes a set has.
const RECOVERY_CODES: usize = 10;

/// The characters of recovery codes: no look-alikes such as `0` and `o`.
const RECOVERY_ALPHABET: &[u8; 31] = b"abcdefghjkmnpqrstuvwxyz23456789";

/// How long an email code lasts.
const EMAIL_CODE_LIFETIME: time::Duration = time::Duration::minutes(10);

/// The length of a new TOTP secret, in bytes (160 bits, as RFC 4226 recommends).
const TOTP_SECRET_LENGTH: usize = 20;

/// What a person gives as second factor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    /// A code from an authenticator app.
    Totp(String),
    /// One of the recovery codes.
    RecoveryCode(String),
    /// The code sent by email.
    EmailCode(String),
    /// A first authenticator app, set up during a sign-in that requires a second factor the account does
    /// not have yet.
    EnrolTotp {
        /// The name of the factor.
        label: String,
        /// The base32 secret of [`Identity::new_totp_secret`].
        secret: String,
        /// A current code of the app.
        code: String,
    },
}

/// A new secret for an authenticator app, to show as a QR code.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct TotpSetup {
    /// The secret in base32.
    pub secret: String,
    /// The `otpauth://` link.
    pub uri: String,
}

/// A factor just added; the recovery codes are shown this once, when the account had none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorAdded {
    /// The factor.
    pub factor: FactorId,
    /// A new set of recovery codes, when the account had none.
    pub recovery_codes: Option<Vec<String>>,
}

/// A sign-in completed with its second factor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completed {
    /// The session.
    pub session: NewSession,
    /// The recovery codes, when the sign-in set up the account's first factor.
    pub recovery_codes: Option<Vec<String>>,
}

/// The answer to a wrong second factor.
fn failed() -> Error {
    Error::unauthenticated("identity.second_factor_failed")
}

/// Checks the name of a factor.
fn label_of(label: &str) -> Result<String> {
    let label = label.trim();
    if (1..=100).contains(&label.chars().count()) {
        Ok(label.to_owned())
    } else {
        Err(Error::validation(
            "identity.factor_label_invalid",
            vec![FieldError::new("label", "length")],
        ))
    }
}

/// A recovery code as it is hashed: lower-case, without spaces or dashes.
fn normalise_code(code: &str) -> String {
    code.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Turns the unique violation of a taken label into a conflict.
fn taken(error: Error) -> Error {
    if error.code() == "db.unique_violation" {
        Error::conflict("identity.factor_label_taken")
    } else {
        error
    }
}

impl Identity {
    /// A new secret for an authenticator app; nothing is stored until a code confirms it.
    ///
    /// # Errors
    ///
    /// `permission_denied` (`identity.step_up_required`) without a recent strong authentication.
    pub async fn new_totp_secret(&self, session: &Session) -> Result<TotpSetup> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let account = self.read_account(session.account).await?;
        let secret = totp::base32_encode(&random_bytes::<TOTP_SECRET_LENGTH>()?);
        let name = account.email.or(account.username).unwrap_or_default();
        Ok(TotpSetup {
            uri: totp::uri(ISSUER, &name, &secret),
            secret,
        })
    }

    /// Adds an authenticator app once a current code proves it is set up.
    ///
    /// # Errors
    ///
    /// `permission_denied` without a recent strong authentication, `validation` for a wrong code or
    /// secret, `conflict` for a label in use.
    pub async fn confirm_totp(
        &self,
        session: &Session,
        label: &str,
        secret: &str,
        code: &str,
    ) -> Result<FactorAdded> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let label = label_of(label)?;
        let bytes = Self::proven_secret(secret, code, self.now())?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, session.account).await?;
        let added = self.insert_totp_in(&mut tx, &account, &label, &bytes).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.tell_owner(&account, Notice::SecondFactorAdded).await;
        Ok(added)
    }

    /// Adds codes by email as a second factor.
    ///
    /// # Errors
    ///
    /// `permission_denied` without a recent strong authentication, `conflict` for an account without
    /// email or a label in use.
    pub async fn add_email_factor(&self, session: &Session, label: &str) -> Result<FactorAdded> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let label = label_of(label)?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, session.account).await?;
        if account.email.is_none() {
            return Err(Error::conflict("identity.account_has_no_email"));
        }
        let had_codes = self.has_recovery_codes_in(&mut tx, account.id).await?;
        let factor = FactorId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, kind, label, created_at) values ($1, $2, 'email', $3, $4)",
            self.table("second_factor")
        )))
        .bind(factor.as_uuid())
        .bind(account.id.as_uuid())
        .bind(&label)
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken)?;
        let recovery_codes = if had_codes {
            None
        } else {
            Some(self.new_recovery_codes_in(&mut tx, account.id).await?)
        };
        tx.commit().await.map_err(kampus_db::classify)?;
        self.tell_owner(&account, Notice::SecondFactorAdded).await;
        Ok(FactorAdded {
            factor,
            recovery_codes,
        })
    }

    /// The second factors of the session's account, oldest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn second_factors(&self, session: &Session) -> Result<Vec<SecondFactor>> {
        let rows = sqlx::query(sql(format!(
            "select id, kind, label, created_at, last_used_at from {} where account_id = $1 order by created_at, id",
            self.table("second_factor")
        )))
        .bind(session.account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.iter()
            .map(|row| {
                Ok(SecondFactor {
                    id: FactorId::from_uuid(row.get("id")),
                    kind: row.get::<String, _>("kind").parse()?,
                    label: row.get("label"),
                    created_at: Timestamp::from_offset_date_time(row.get("created_at")),
                    last_used_at: row
                        .get::<Option<time::OffsetDateTime>, _>("last_used_at")
                        .map(Timestamp::from_offset_date_time),
                })
            })
            .collect()
    }

    /// Removes a second factor; removing the last one also removes the recovery codes, and is refused
    /// while the campus or a role requires a second factor.
    ///
    /// # Errors
    ///
    /// `permission_denied` without a recent strong authentication, `not_found` for an unknown factor,
    /// `conflict` (`identity.second_factor_required`) for the last required factor.
    pub async fn remove_second_factor(&self, session: &Session, id: FactorId) -> Result<()> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, session.account).await?;
        let deleted = sqlx::query(sql(format!(
            "delete from {} where id = $1 and account_id = $2",
            self.table("second_factor")
        )))
        .bind(id.as_uuid())
        .bind(account.id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if deleted == 0 {
            return Err(Error::not_found("identity.factor_not_found"));
        }
        if self.factor_kinds_in(&mut tx, account.id).await?.is_empty() {
            if policy.second_factor_required
                || self.access.second_factor_required_in(&mut tx, account.id).await?
            {
                return Err(Error::conflict("identity.second_factor_required"));
            }
            sqlx::query(sql(format!(
                "delete from {} where account_id = $1",
                self.table("recovery_code")
            )))
            .bind(account.id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        self.tell_owner(&account, Notice::SecondFactorRemoved).await;
        Ok(())
    }

    /// Replaces the recovery codes with a new set, shown this once.
    ///
    /// # Errors
    ///
    /// `permission_denied` without a recent strong authentication, `conflict` for an account without a
    /// second factor.
    pub async fn regenerate_recovery_codes(&self, session: &Session) -> Result<Vec<String>> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, session.account).await?;
        if self.factor_kinds_in(&mut tx, account.id).await?.is_empty() {
            return Err(Error::conflict("identity.no_second_factor"));
        }
        let codes = self.new_recovery_codes_in(&mut tx, account.id).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(codes)
    }

    /// Sends a code by email for a sign-in waiting for its second factor, when the account has that
    /// factor; the answer is the same otherwise.
    ///
    /// # Errors
    ///
    /// `validation` (`identity.token_invalid`) for an unknown or expired challenge.
    pub async fn send_email_code(&self, challenge: &str) -> Result<()> {
        let id = self.peek(Purpose::SecondFactor, challenge).await?;
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, id).await?;
        let kinds = self.factor_kinds_in(&mut tx, id).await?;
        let (true, Some(email)) = (kinds.contains(&FactorKind::Email), account.email.clone()) else {
            return Ok(());
        };
        let code = self
            .issue_code_in(&mut tx, id, Purpose::EmailCode, EMAIL_CODE_LIFETIME)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.deliver(Message {
            campus: account.campus,
            account: id,
            email,
            notice: Notice::EmailCode { code },
        })
        .await;
        Ok(())
    }

    /// Completes a sign-in with its second factor. The challenge works once: a wrong
    /// factor ends it, counts as a failed sign-in, and the person signs in again.
    ///
    /// # Errors
    ///
    /// `validation` for an unknown or used challenge, `rate_limited` while throttled, `unauthenticated`
    /// (`identity.second_factor_failed`) for a wrong factor.
    pub async fn complete_sign_in(
        &self,
        challenge: &str,
        proof: Proof,
        client: &Client,
    ) -> Result<Completed> {
        let mut tx = self.db.begin().await?;
        let id = self.consume_in(&mut tx, Purpose::SecondFactor, challenge).await?;
        let account = self.lock_account_in(&mut tx, id).await?;
        let policy = self.policies.policy_in(&mut tx, account.campus).await?;
        let name = account.username.clone().unwrap_or_default();
        let address = client.address.as_deref();
        if let Some(retry_after) = self
            .throttled(
                account.campus,
                &policy,
                Some(id),
                &crate::service::sign_in::name_hash(account.campus, &name),
                address,
            )
            .await?
        {
            tx.commit().await.map_err(kampus_db::classify)?;
            return Err(crate::service::sign_in::throttled_error(retry_after));
        }
        if account.status != AccountStatus::Active || account.kind != AccountKind::Person {
            tx.commit().await.map_err(kampus_db::classify)?;
            return Err(failed());
        }
        let outcome = self.check_proof_in(&mut tx, &account, &proof, true).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        let Some(recovery_codes) = outcome else {
            self.record_failure(&account, address).await?;
            return Err(failed());
        };
        let session = self
            .open(&account, MfaLevel::SecondFactor, client, &policy)
            .await?;
        self.record_success(&account, address).await?;
        Ok(Completed {
            session,
            recovery_codes,
        })
    }

    /// Whether the account must give a second factor: it set one up, one of its roles requires it,
    /// or the campus does.
    pub(crate) async fn second_factor_needed(
        &self,
        account: &Account,
        policy: &CampusPolicy,
    ) -> Result<bool> {
        if policy.second_factor_required {
            return Ok(true);
        }
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let has_factors = !self.factor_kinds_in(&mut conn, account.id).await?.is_empty();
        drop(conn);
        if has_factors {
            return Ok(true);
        }
        self.access.second_factor_required(account.id).await
    }

    /// The kinds of the account's factors, without repeats.
    pub(crate) async fn factor_kinds_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
    ) -> Result<Vec<FactorKind>> {
        let kinds: Vec<String> = sqlx::query_scalar(sql(format!(
            "select distinct kind from {} where account_id = $1 order by kind",
            self.table("second_factor")
        )))
        .bind(account.as_uuid())
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        kinds.iter().map(|k| k.parse()).collect()
    }

    /// Checks a proof; `Some` when it passes, with the recovery codes of a factor it set up. Email codes
    /// count only when `email` allows them: recovery, which starts by email, does not.
    pub(crate) async fn check_proof_in(
        &self,
        conn: &mut PgConnection,
        account: &Account,
        proof: &Proof,
        email: bool,
    ) -> Result<Option<Option<Vec<String>>>> {
        let passed = match proof {
            Proof::Totp(code) => self.check_totp_in(conn, account.id, code).await?,
            Proof::RecoveryCode(code) => self.use_recovery_code_in(conn, account.id, code).await?,
            Proof::EmailCode(code) => {
                email
                    && self
                        .factor_kinds_in(conn, account.id)
                        .await?
                        .contains(&FactorKind::Email)
                    && self
                        .consume_code_in(conn, account.id, Purpose::EmailCode, code)
                        .await
                        .is_ok()
            }
            Proof::EnrolTotp { label, secret, code } => {
                if !self.factor_kinds_in(conn, account.id).await?.is_empty() {
                    return Ok(None);
                }
                let Ok(bytes) = Self::proven_secret(secret, code, self.now()) else {
                    return Ok(None);
                };
                let added = self
                    .insert_totp_in(conn, account, &label_of(label)?, &bytes)
                    .await?;
                return Ok(Some(added.recovery_codes));
            }
        };
        Ok(passed.then_some(None))
    }

    /// The bytes of a base32 secret, when a code proves the app holds it.
    fn proven_secret(secret: &str, code: &str, now: Timestamp) -> Result<Vec<u8>> {
        let bytes = totp::base32_decode(secret)
            .filter(|b| b.len() >= 16)
            .ok_or_else(|| {
                Error::validation(
                    "identity.totp_secret_invalid",
                    vec![FieldError::new("secret", "base32")],
                )
            })?;
        totp::verify(&bytes, code, now.as_offset_date_time().unix_timestamp(), None)
            .map(|_| bytes)
            .ok_or_else(|| {
                Error::validation("identity.totp_code_wrong", vec![FieldError::new("code", "valid")])
            })
    }

    /// Stores an authenticator app with its secret sealed, marked used now so the confirming code cannot
    /// be used again, and the first recovery codes when the account has none.
    async fn insert_totp_in(
        &self,
        conn: &mut PgConnection,
        account: &Account,
        label: &str,
        secret: &[u8],
    ) -> Result<FactorAdded> {
        let had_codes = self.has_recovery_codes_in(conn, account.id).await?;
        let factor = FactorId::generate();
        let now = self.now().as_offset_date_time();
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, kind, secret_encrypted, label, created_at, last_used_at)
             values ($1, $2, 'totp', $3, $4, $5, $5)",
            self.table("second_factor")
        )))
        .bind(factor.as_uuid())
        .bind(account.id.as_uuid())
        .bind(self.key.seal(secret)?)
        .bind(label)
        .bind(now)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)
        .map_err(taken)?;
        let recovery_codes = if had_codes {
            None
        } else {
            Some(self.new_recovery_codes_in(conn, account.id).await?)
        };
        Ok(FactorAdded {
            factor,
            recovery_codes,
        })
    }

    /// Whether a code matches one of the account's apps, after the step of its last use.
    async fn check_totp_in(&self, conn: &mut PgConnection, account: AccountId, code: &str) -> Result<bool> {
        let rows = sqlx::query(sql(format!(
            "select id, secret_encrypted, last_used_at from {} where account_id = $1 and kind = 'totp' for update",
            self.table("second_factor")
        )))
        .bind(account.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let now = self.now().as_offset_date_time();
        for row in &rows {
            let sealed: Vec<u8> = row.get("secret_encrypted");
            let secret = self.key.open(&sealed)?;
            let after = row
                .get::<Option<time::OffsetDateTime>, _>("last_used_at")
                .map(|at| totp::step_of(at.unix_timestamp()));
            if totp::verify(&secret, code, now.unix_timestamp(), after).is_some() {
                sqlx::query(sql(format!(
                    "update {} set last_used_at = $2 where id = $1",
                    self.table("second_factor")
                )))
                .bind(row.get::<uuid::Uuid, _>("id"))
                .bind(now)
                .execute(&mut *conn)
                .await
                .map_err(kampus_db::classify)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Uses one recovery code.
    async fn use_recovery_code_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
        code: &str,
    ) -> Result<bool> {
        let used = sqlx::query(sql(format!(
            "update {} set used_at = $3 where account_id = $1 and hash = $2 and used_at is null",
            self.table("recovery_code")
        )))
        .bind(account.as_uuid())
        .bind(token_hash(&normalise_code(code)))
        .bind(self.now().as_offset_date_time())
        .execute(conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        Ok(used == 1)
    }

    /// Whether the account has unused recovery codes.
    async fn has_recovery_codes_in(&self, conn: &mut PgConnection, account: AccountId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where account_id = $1 and used_at is null)",
            self.table("recovery_code")
        )))
        .bind(account.as_uuid())
        .fetch_one(conn)
        .await
        .map_err(kampus_db::classify)
    }

    /// Replaces the account's recovery codes with a new set and returns it.
    async fn new_recovery_codes_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
    ) -> Result<Vec<String>> {
        let table = self.table("recovery_code");
        sqlx::query(sql(format!("delete from {table} where account_id = $1")))
            .bind(account.as_uuid())
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        let mut codes = Vec::with_capacity(RECOVERY_CODES);
        while codes.len() < RECOVERY_CODES {
            let code: String = random_bytes::<10>()?
                .iter()
                .map(|b| char::from(RECOVERY_ALPHABET[usize::from(b % 31)]))
                .collect();
            let code = format!("{}-{}", &code[..5], &code[5..]);
            if codes.contains(&code) {
                continue;
            }
            sqlx::query(sql(format!(
                "insert into {table} (id, account_id, hash, created_at) values ($1, $2, $3, $4)"
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(account.as_uuid())
            .bind(token_hash(&normalise_code(&code)))
            .bind(self.now().as_offset_date_time())
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
            codes.push(code);
        }
        Ok(codes)
    }

    /// Reads an account without locking it.
    pub(crate) async fn read_account(&self, id: AccountId) -> Result<Account> {
        let row = sqlx::query(sql(format!(
            "select {} from {} where id = $1",
            crate::service::account::COLUMNS,
            self.table("account")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("identity.account_not_found"))?;
        crate::service::account::from_row(&row)
    }

    /// Tells the owner of an account about a change to its credentials.
    pub(crate) async fn tell_owner(&self, account: &Account, notice: Notice) {
        if let Some(email) = account.email.clone() {
            self.deliver(Message {
                campus: account.campus,
                account: account.id,
                email,
                notice,
            })
            .await;
        }
    }
}
