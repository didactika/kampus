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

//! Accounts (RN-022) and sign-up with confirmation (UC-006). The answer to a sign-up never depends on
//! whether the email has an account (RN-027): the owner of the email is told instead.

use kampus_access::{Actor, ScopeId, ScopeKind};
use kampus_common::{Error, FieldError, Result, Timestamp};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use crate::declarations::{ACCOUNT_READ, MEMBER_TEMPLATE};
use crate::one_time::{Purpose, invalid_token};
use crate::{Account, AccountId, AccountKind, AccountStatus, Identity, MODULE, Message, Notice, sql};

/// The columns of an account, in the order [`from_row`] reads them.
pub(crate) const COLUMNS: &str =
    "id, campus_id, scope_id, kind, username, email, email_verified_at, status, status_reason,
    last_sign_in_at, deletion_due_at, pseudonym, version";

/// A sign-up request (UC-006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignUp {
    /// The email, which is also the username.
    pub email: String,
    /// Whether the visitor accepted the campus terms.
    pub accepted_terms: bool,
}

/// Normalises an email: trimmed, lower-case, one `@` between a local part and a domain with a dot.
///
/// # Errors
///
/// `validation` on the field `email` when it is not an email.
pub fn normalise_email(email: &str) -> Result<String> {
    let email = email.trim().to_lowercase();
    let valid = email.len() <= 254
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !domain.contains('@')
        })
        && !email.chars().any(char::is_whitespace);
    if valid {
        Ok(email)
    } else {
        Err(Error::validation(
            "identity.email_invalid",
            vec![FieldError::new("email", "format")],
        ))
    }
}

/// Normalises a username: trimmed, lower-case, 1 to 100 letters, digits or `.`, `_`, `-`, `@`.
///
/// # Errors
///
/// `validation` on the field `username` when it breaks the format.
pub fn normalise_username(username: &str) -> Result<String> {
    let username = username.trim().to_lowercase();
    let valid = (1..=100).contains(&username.chars().count())
        && username
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '@'));
    if valid {
        Ok(username)
    } else {
        Err(Error::validation(
            "identity.username_invalid",
            vec![FieldError::new("username", "format")],
        ))
    }
}

/// Reads an account from a row selected with [`COLUMNS`].
pub(crate) fn from_row(row: &PgRow) -> Result<Account> {
    let instant = |name: &str| -> Option<Timestamp> {
        row.get::<Option<time::OffsetDateTime>, _>(name)
            .map(Timestamp::from_offset_date_time)
    };
    Ok(Account {
        id: AccountId::from_uuid(row.get("id")),
        campus: ScopeId::from_uuid(row.get("campus_id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        kind: row.get::<String, _>("kind").parse()?,
        username: row.get("username"),
        email: row.get("email"),
        email_verified_at: instant("email_verified_at"),
        status: row.get::<String, _>("status").parse()?,
        status_reason: row.get("status_reason"),
        last_sign_in_at: instant("last_sign_in_at"),
        deletion_due_at: instant("deletion_due_at"),
        pseudonym: row.get("pseudonym"),
        version: row.get("version"),
    })
}

/// A new account, before it is stored.
pub(crate) struct NewAccount<'a> {
    /// Its campus.
    pub campus: ScopeId,
    /// Person or service.
    pub kind: AccountKind,
    /// The normalised username.
    pub username: &'a str,
    /// The normalised email.
    pub email: Option<&'a str>,
    /// The first status.
    pub status: AccountStatus,
    /// Who creates it, absent for a sign-up.
    pub actor: Option<AccountId>,
}

impl Identity {
    /// Stores a new account with its principal and user scope, records its first status and, for an
    /// active person, grants the member role of the campus.
    pub(crate) async fn insert_account_in(
        &self,
        conn: &mut PgConnection,
        new: NewAccount<'_>,
    ) -> Result<Account> {
        let id = AccountId::generate();
        self.access.create_principal_in(conn, id, new.campus).await?;
        let active = new.status == AccountStatus::Active;
        if !active {
            self.access.set_principal_active_in(conn, id, false).await?;
        }
        let scope = self
            .access
            .create_scope_in(conn, new.campus, ScopeKind::User)
            .await?;
        let now = self.now().as_offset_date_time();
        let row = sqlx::query(sql(format!(
            "insert into {} (id, campus_id, scope_id, kind, username, email, status, status_changed_at, status_changed_by)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning {COLUMNS}",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .bind(new.campus.as_uuid())
        .bind(scope.as_uuid())
        .bind(new.kind.as_str())
        .bind(new.username)
        .bind(new.email)
        .bind(new.status.as_str())
        .bind(now)
        .bind(new.actor.map(|a| a.as_uuid()))
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let account = from_row(&row)?;
        self.record_transition_in(conn, id, None, new.status, None, new.actor)
            .await?;
        if active && new.kind == AccountKind::Person {
            self.grant_membership_in(conn, &account).await?;
        }
        Ok(account)
    }

    /// Grants the member role of the campus to an account that becomes active for the first time.
    pub(crate) async fn grant_membership_in(&self, conn: &mut PgConnection, account: &Account) -> Result<()> {
        let role = self
            .access
            .campus_role_in(conn, account.campus, MEMBER_TEMPLATE)
            .await?;
        self.access
            .grant_for_module_in(conn, MODULE, role, account.campus, account.id, None)
            .await
            .map(drop)
    }

    /// Records one step of the lifecycle (RN-023).
    pub(crate) async fn record_transition_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
        from: Option<AccountStatus>,
        to: AccountStatus,
        reason: Option<&str>,
        actor: Option<AccountId>,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, from_status, to_status, reason, actor, at) values ($1, $2, $3, $4, $5, $6, $7)",
            self.table("account_transition")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(account.as_uuid())
        .bind(from.map(AccountStatus::as_str))
        .bind(to.as_str())
        .bind(reason)
        .bind(actor.map(|a| a.as_uuid()))
        .bind(self.now().as_offset_date_time())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Reads an account and locks it until the transaction ends.
    pub(crate) async fn lock_account_in(&self, conn: &mut PgConnection, id: AccountId) -> Result<Account> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1 for update",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("identity.account_not_found"))?;
        from_row(&row)
    }

    /// The account of a campus with this username or email, if any.
    pub(crate) async fn find_in(
        &self,
        conn: &mut PgConnection,
        campus: ScopeId,
        name: &str,
    ) -> Result<Option<Account>> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where campus_id = $1 and (username = $2 or email = $2)
             order by (username = $2) desc limit 1",
            self.table("account")
        )))
        .bind(campus.as_uuid())
        .bind(name)
        .fetch_optional(conn)
        .await
        .map_err(kampus_db::classify)?;
        row.as_ref().map(from_row).transpose()
    }

    /// An account, for the account itself or for someone with `identity.account.read` in its campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown account, `permission_denied` without the permission.
    pub async fn account(&self, actor: &Actor, id: AccountId) -> Result<Account> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("identity.account_not_found"))?;
        let account = from_row(&row)?;
        if !matches!(actor, Actor::Principal(p) if *p == id) {
            self.access.require(actor, ACCOUNT_READ, account.campus).await?;
        }
        Ok(account)
    }

    /// Signs up a visitor (UC-006): the account waits for the link sent to the email. When the email has
    /// an account already, its owner is told instead and the answer is the same (A1).
    ///
    /// # Errors
    ///
    /// `permission_denied` when the campus does not allow sign-up, `validation` for a missing acceptance,
    /// an invalid email or a domain the campus does not allow.
    pub async fn sign_up(&self, campus: ScopeId, request: SignUp) -> Result<()> {
        let policy = self.policies.policy(campus).await?;
        if !policy.self_sign_up {
            return Err(Error::permission_denied("identity.sign_up_closed"));
        }
        let mut fields = Vec::new();
        if !request.accepted_terms {
            fields.push(FieldError::new("accepted_terms", "required"));
        }
        let email = match normalise_email(&request.email) {
            Ok(email) => Some(email),
            Err(error) => {
                fields.extend(error.fields().iter().cloned());
                None
            }
        };
        if let Some(email) = &email {
            let domain = email.rsplit('@').next().unwrap_or_default();
            if !policy.allowed_domains.is_empty()
                && !policy
                    .allowed_domains
                    .iter()
                    .any(|d| d.eq_ignore_ascii_case(domain))
            {
                fields.push(FieldError::new("email", "domain"));
            }
        }
        let Some(email) = email.filter(|_| fields.is_empty()) else {
            return Err(Error::validation("identity.sign_up_invalid", fields));
        };

        let notice = match self.sign_up_in(campus, &policy, &email).await {
            Ok(notice) => notice,
            Err(error) if error.code() == "db.unique_violation" => None,
            Err(error) => return Err(error),
        };
        if let Some((account, notice)) = notice {
            self.deliver(Message {
                campus,
                account,
                email,
                notice,
            })
            .await;
        }
        Ok(())
    }

    /// The stored part of a sign-up: the account and its link, or what to tell the existing owner. A
    /// concurrent sign-up with the same email fails here with a unique violation.
    async fn sign_up_in(
        &self,
        campus: ScopeId,
        policy: &crate::CampusPolicy,
        email: &str,
    ) -> Result<Option<(AccountId, Notice)>> {
        let mut tx = self.db.begin().await?;
        let notice = match self.find_in(&mut tx, campus, email).await? {
            Some(existing) if existing.status == AccountStatus::PendingConfirmation => {
                let token = self
                    .issue_in(
                        &mut tx,
                        existing.id,
                        Purpose::ConfirmEmail,
                        policy.confirmation_lifetime,
                    )
                    .await?;
                Some((existing.id, Notice::ConfirmEmail { token }))
            }
            Some(existing) if existing.status == AccountStatus::Deleted => None,
            Some(existing) => Some((existing.id, Notice::AlreadyRegistered)),
            None => {
                let account = self
                    .insert_account_in(
                        &mut tx,
                        NewAccount {
                            campus,
                            kind: AccountKind::Person,
                            username: email,
                            email: Some(email),
                            status: AccountStatus::PendingConfirmation,
                            actor: None,
                        },
                    )
                    .await?;
                let token = self
                    .issue_in(
                        &mut tx,
                        account.id,
                        Purpose::ConfirmEmail,
                        policy.confirmation_lifetime,
                    )
                    .await?;
                Some((account.id, Notice::ConfirmEmail { token }))
            }
        };
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(notice)
    }

    /// Confirms a signed-up account with the link's token and the password the person chooses (UC-006
    /// step 3): the account becomes active and joins its campus.
    ///
    /// # Errors
    ///
    /// `validation` for an invalid token (A2) or a rejected password.
    pub async fn confirm(&self, token: &str, password: &str) -> Result<AccountId> {
        self.activate_with(
            Purpose::ConfirmEmail,
            AccountStatus::PendingConfirmation,
            token,
            password,
        )
        .await
    }

    /// Activates an account with a one-time token of a purpose, from the status the purpose expects.
    pub(crate) async fn activate_with(
        &self,
        purpose: Purpose,
        from: AccountStatus,
        token: &str,
        password: &str,
    ) -> Result<AccountId> {
        let mut tx = self.db.begin().await?;
        let id = self.consume_in(&mut tx, purpose, token).await?;
        let account = self.lock_account_in(&mut tx, id).await?;
        if account.status != from {
            return Err(invalid_token());
        }
        let policy = self.policies.policy(account.campus).await?;
        self.set_password_in(&mut tx, &account, &policy, password).await?;
        sqlx::query(sql(format!(
            "update {} set email_verified_at = $2 where id = $1",
            self.table("account")
        )))
        .bind(id.as_uuid())
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.transition_in(&mut tx, &account, AccountStatus::Active, None, None)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }
}
