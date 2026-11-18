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

//! Server-side sessions (RN-031) and step-up (RN-032). A session token is random and stored as a hash;
//! every request checks the session against the account's status, so revocation is immediate.

use kampus_access::ScopeId;
use kampus_common::{Error, Result, Timestamp};
use sqlx::{PgConnection, Row};

use crate::support::secret::{new_token, token_hash};
use crate::{AccountId, CampusPolicy, Identity, MfaLevel, Session, SessionId, SessionSummary, later, sql};

/// How often the last use of a session is written, so reading a session rarely writes.
const SEEN_EVERY: time::Duration = time::Duration::minutes(1);

/// Where a session is opened from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Client {
    /// The client's name, such as the browser's user agent.
    pub name: Option<String>,
    /// The network address.
    pub address: Option<String>,
}

/// A new session and the token that carries it; the token is shown once, to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSession {
    /// The session.
    pub session: Session,
    /// Its token.
    pub token: String,
}

impl Identity {
    /// Opens a session for an account after the sign-in pipeline.
    pub(crate) async fn create_session_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
        campus: ScopeId,
        mfa_level: MfaLevel,
        client: &Client,
        policy: &CampusPolicy,
    ) -> Result<NewSession> {
        let id = SessionId::generate();
        let token = new_token()?;
        let now = self.now();
        let expires_at = later(now, policy.session_lifetime);
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, token_hash, mfa_level, strong_auth_at, created_at, last_seen_at, expires_at, client, address)
             values ($1, $2, $3, $4, $5, $5, $5, $6, $7, $8)",
            self.table("session")
        )))
        .bind(id.as_uuid())
        .bind(account.as_uuid())
        .bind(token_hash(&token))
        .bind(mfa_level.as_str())
        .bind(now.as_offset_date_time())
        .bind(expires_at.as_offset_date_time())
        .bind(client.name.as_deref())
        .bind(client.address.as_deref())
        .execute(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(NewSession {
            session: Session {
                id,
                account,
                campus,
                mfa_level,
                strong_auth_at: now,
                expires_at,
            },
            token,
        })
    }

    /// The session a token carries, checked against its lifetime, its idle time and the status of its
    /// account (RN-031).
    ///
    /// # Errors
    ///
    /// `unauthenticated` (`identity.session_invalid`) for an unknown, expired or revoked session, or one
    /// whose account is not active.
    pub async fn authenticate(&self, token: &str) -> Result<Session> {
        let row = sqlx::query(sql(format!(
            "select s.id, s.account_id, a.campus_id, s.mfa_level, s.strong_auth_at, s.expires_at, s.last_seen_at
             from {} s join {} a on a.id = s.account_id
             where s.token_hash = $1 and a.status = 'active'",
            self.table("session"),
            self.table("account")
        )))
        .bind(token_hash(token))
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(invalid_session)?;
        let instant = |name: &str| Timestamp::from_offset_date_time(row.get(name));
        let session = Session {
            id: SessionId::from_uuid(row.get("id")),
            account: AccountId::from_uuid(row.get("account_id")),
            campus: ScopeId::from_uuid(row.get("campus_id")),
            mfa_level: MfaLevel::parse(row.get("mfa_level")),
            strong_auth_at: instant("strong_auth_at"),
            expires_at: instant("expires_at"),
        };
        let last_seen = instant("last_seen_at").as_offset_date_time();
        let now = self.now();
        let policy = self.policies.policy(session.campus).await?;
        if now >= session.expires_at || now.as_offset_date_time() - last_seen >= policy.session_idle {
            self.end_session(session.id).await?;
            return Err(invalid_session());
        }
        if now.as_offset_date_time() - last_seen >= SEEN_EVERY {
            sqlx::query(sql(format!(
                "update {} set last_seen_at = $2 where id = $1",
                self.table("session")
            )))
            .bind(session.id.as_uuid())
            .bind(now.as_offset_date_time())
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(session)
    }

    /// Ends the session of a token: signing out. Unknown tokens are ignored.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sign_out(&self, token: &str) -> Result<()> {
        sqlx::query(sql(format!(
            "delete from {} where token_hash = $1",
            self.table("session")
        )))
        .bind(token_hash(token))
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Ends one session.
    async fn end_session(&self, id: SessionId) -> Result<()> {
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("session")
        )))
        .bind(id.as_uuid())
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// The open sessions of the session's account, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn sessions(&self, session: &Session) -> Result<Vec<SessionSummary>> {
        let rows = sqlx::query(sql(format!(
            "select id, created_at, last_seen_at, expires_at, client, address from {} where account_id = $1
             order by created_at desc",
            self.table("session")
        )))
        .bind(session.account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|row| SessionSummary {
                id: SessionId::from_uuid(row.get("id")),
                created_at: Timestamp::from_offset_date_time(row.get("created_at")),
                last_seen_at: Timestamp::from_offset_date_time(row.get("last_seen_at")),
                expires_at: Timestamp::from_offset_date_time(row.get("expires_at")),
                client: row.get("client"),
                address: row.get("address"),
            })
            .collect())
    }

    /// Ends another session of the same account, such as a lost device.
    ///
    /// # Errors
    ///
    /// `not_found` when the account has no such session.
    pub async fn revoke_session(&self, session: &Session, id: SessionId) -> Result<()> {
        let deleted = sqlx::query(sql(format!(
            "delete from {} where id = $1 and account_id = $2",
            self.table("session")
        )))
        .bind(id.as_uuid())
        .bind(session.account.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if deleted == 0 {
            Err(Error::not_found("identity.session_not_found"))
        } else {
            Ok(())
        }
    }

    /// Ends every session and revokes every API token of an account (RN-031).
    pub(crate) async fn revoke_everything_in(
        &self,
        conn: &mut PgConnection,
        account: AccountId,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "delete from {} where account_id = $1",
            self.table("session")
        )))
        .bind(account.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set revoked_at = $2 where account_id = $1 and revoked_at is null",
            self.table("api_token")
        )))
        .bind(account.as_uuid())
        .bind(self.now().as_offset_date_time())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Proves the password again for a sensitive operation (RN-032); the session's strong authentication
    /// becomes now.
    ///
    /// # Errors
    ///
    /// `unauthenticated` for a wrong password.
    pub async fn step_up(&self, session: &Session, password: &str) -> Result<Session> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let stored = self.password_hash_in(&mut conn, session.account).await?;
        if !self.hasher.verify(password, stored.as_deref()).await? {
            return Err(Error::unauthenticated("identity.password_wrong"));
        }
        let now = self.now();
        sqlx::query(sql(format!(
            "update {} set strong_auth_at = $2 where id = $1",
            self.table("session")
        )))
        .bind(session.id.as_uuid())
        .bind(now.as_offset_date_time())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(Session {
            strong_auth_at: now,
            ..session.clone()
        })
    }

    /// Refuses a sensitive operation when the session's last strong authentication is too old (RN-032).
    ///
    /// # Errors
    ///
    /// `permission_denied` (`identity.step_up_required`).
    pub fn require_recent_strong_auth(&self, session: &Session, policy: &CampusPolicy) -> Result<()> {
        if self.now().as_offset_date_time() - session.strong_auth_at.as_offset_date_time()
            > policy.step_up_window
        {
            Err(Error::permission_denied("identity.step_up_required"))
        } else {
            Ok(())
        }
    }
}

/// The error of a session that does not authenticate.
fn invalid_session() -> Error {
    Error::unauthenticated("identity.session_invalid")
}
