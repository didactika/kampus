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

//! API tokens (RN-033): scoped to permissions in scopes, expiring, shown once and stored as hashes. A
//! token is `kmp_<prefix>_<secret>`; its prefix names it in lists and logs. A request with a token may do
//! only what both the token's scopes and the account's roles allow.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use serde::Serialize;
use sqlx::{PgConnection, Row};

use crate::permissions::{TOKEN_MANAGE_ANY, TOKEN_MANAGE_OWN};
use crate::support::secret::{new_token, random_hex, same, token_hash};
use crate::{AccountId, AccountStatus, Identity, Session, TokenId, later, sql};

/// How often the last use of a token is written.
const USED_EVERY: time::Duration = time::Duration::minutes(1);

/// A permission a token carries, in a scope and its descendants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenScope {
    /// The permission.
    pub permission: String,
    /// The scope.
    pub scope: ScopeId,
}

/// A token to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRequest {
    /// A name for its owner.
    pub name: String,
    /// What it may do; at least one.
    pub scopes: Vec<TokenScope>,
    /// When it expires; the campus default when absent.
    pub expires_at: Option<Timestamp>,
}

/// A created token; the token itself is shown this once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewToken {
    /// The token.
    pub id: TokenId,
    /// The prefix that names it.
    pub prefix: String,
    /// The secret to give to the integration.
    pub token: String,
    /// When it expires.
    pub expires_at: Timestamp,
}

/// A token that authenticated a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenAuth {
    /// The token.
    pub id: TokenId,
    /// The account it acts for.
    pub account: AccountId,
    /// The campus of the account.
    pub campus: ScopeId,
    /// What it may do.
    pub scopes: Vec<TokenScope>,
}

/// A token as listed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenSummary {
    /// The token.
    pub id: TokenId,
    /// Its name.
    pub name: String,
    /// Its prefix.
    pub prefix: String,
    /// When it was created.
    pub created_at: Timestamp,
    /// When it expires.
    pub expires_at: Timestamp,
    /// When it was last used.
    pub last_used_at: Option<Timestamp>,
    /// When it was revoked.
    pub revoked_at: Option<Timestamp>,
    /// What it may do.
    pub scopes: Vec<TokenScope>,
}

/// The error of a token that does not authenticate.
fn invalid_token() -> Error {
    Error::unauthenticated("identity.api_token_invalid")
}

impl Identity {
    /// Creates a token for the session's own account: needs `identity.token.manage_own` and a recent strong
    /// authentication (RN-032), since a token is a credential.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission or the step-up, `validation` for scopes the account does
    /// not hold or a lifetime out of range.
    pub async fn create_own_token(&self, session: &Session, request: TokenRequest) -> Result<NewToken> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        self.access
            .require(
                &Actor::Principal(session.account),
                TOKEN_MANAGE_OWN,
                session.campus,
            )
            .await?;
        self.insert_token(session.account, request).await
    }

    /// Creates a token for another account, such as a service account (RN-039): needs
    /// `identity.token.manage_any` in its campus.
    ///
    /// # Errors
    ///
    /// As [`Identity::create_own_token`], and `not_found` for an unknown account.
    pub async fn create_token_for(
        &self,
        actor: &Actor,
        owner: AccountId,
        request: TokenRequest,
    ) -> Result<NewToken> {
        let campus = self.campus_of(owner).await?;
        self.access.require(actor, TOKEN_MANAGE_ANY, campus).await?;
        self.insert_token(owner, request).await
    }

    /// The campus of an account.
    async fn campus_of(&self, account: AccountId) -> Result<ScopeId> {
        sqlx::query_scalar(sql(format!(
            "select campus_id from {} where id = $1",
            self.table("account")
        )))
        .bind(account.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .map(ScopeId::from_uuid)
        .ok_or_else(|| Error::not_found("identity.account_not_found"))
    }

    /// Checks a request and stores the token.
    async fn insert_token(&self, owner: AccountId, request: TokenRequest) -> Result<NewToken> {
        let mut tx = self.db.begin().await?;
        let account = self.lock_account_in(&mut tx, owner).await?;
        if account.status != AccountStatus::Active {
            return Err(Error::conflict("identity.account_not_active"));
        }
        let policy = self.policies.policy(account.campus).await?;
        let now = self.now();
        let expires_at = request
            .expires_at
            .unwrap_or_else(|| later(now, policy.token_default_lifetime));
        let mut fields = Vec::new();
        let name = request.name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            fields.push(FieldError::new("name", "length"));
        }
        if expires_at <= now || expires_at > later(now, policy.token_max_lifetime) {
            fields.push(FieldError::new("expires_at", "range"));
        }
        if request.scopes.is_empty() {
            fields.push(FieldError::new("scopes", "required"));
        }
        for (index, scope) in request.scopes.iter().enumerate() {
            let held = self
                .access
                .check(&Actor::Principal(owner), &scope.permission, scope.scope)
                .await
                .is_ok_and(|decision| decision.allowed);
            if !held {
                fields.push(FieldError::new(format!("scopes[{index}]"), "not_held"));
            }
        }
        if !fields.is_empty() {
            return Err(Error::validation("identity.api_token_invalid_request", fields));
        }
        let id = TokenId::generate();
        let prefix = random_hex::<5>()?;
        let token = format!("kmp_{prefix}_{}", new_token()?);
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, name, prefix, hash, created_at, expires_at) values ($1, $2, $3, $4, $5, $6, $7)",
            self.table("api_token")
        )))
        .bind(id.as_uuid())
        .bind(owner.as_uuid())
        .bind(name)
        .bind(&prefix)
        .bind(token_hash(&token))
        .bind(now.as_offset_date_time())
        .bind(expires_at.as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let scopes = self.table("api_token_scope");
        for scope in &request.scopes {
            sqlx::query(sql(format!(
                "insert into {scopes} (token_id, permission_code, scope_id) values ($1, $2, $3) on conflict do nothing"
            )))
            .bind(id.as_uuid())
            .bind(&scope.permission)
            .bind(scope.scope.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(NewToken {
            id,
            prefix,
            token,
            expires_at,
        })
    }

    /// The token a request carries, checked against its expiry, its revocation and its account's status.
    ///
    /// # Errors
    ///
    /// `unauthenticated` (`identity.api_token_invalid`) when it does not authenticate.
    pub async fn authenticate_token(&self, token: &str) -> Result<TokenAuth> {
        let prefix = token
            .strip_prefix("kmp_")
            .and_then(|rest| rest.split_once('_'))
            .map(|(prefix, _)| prefix)
            .ok_or_else(invalid_token)?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let row = sqlx::query(sql(format!(
            "select t.id, t.account_id, a.campus_id, t.hash, t.expires_at, t.revoked_at, t.last_used_at
             from {} t join {} a on a.id = t.account_id where t.prefix = $1 and a.status = 'active'",
            self.table("api_token"),
            self.table("account")
        )))
        .bind(prefix)
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(invalid_token)?;
        let now = self.now().as_offset_date_time();
        let stored: Vec<u8> = row.get("hash");
        let expires_at: time::OffsetDateTime = row.get("expires_at");
        let revoked: Option<time::OffsetDateTime> = row.get("revoked_at");
        if !same(&stored, &token_hash(token)) || revoked.is_some() || expires_at <= now {
            return Err(invalid_token());
        }
        let id = TokenId::from_uuid(row.get("id"));
        let last_used: Option<time::OffsetDateTime> = row.get("last_used_at");
        if last_used.is_none_or(|at| now - at >= USED_EVERY) {
            sqlx::query(sql(format!(
                "update {} set last_used_at = $2 where id = $1",
                self.table("api_token")
            )))
            .bind(id.as_uuid())
            .bind(now)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(TokenAuth {
            id,
            account: AccountId::from_uuid(row.get("account_id")),
            campus: ScopeId::from_uuid(row.get("campus_id")),
            scopes: self.token_scopes(&mut conn, id).await?,
        })
    }

    /// Whether a request with a token may use a permission in a scope: the token must carry it in the
    /// scope or an ancestor, and the account must hold it there.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn token_may(&self, auth: &TokenAuth, permission: &str, scope: ScopeId) -> Result<bool> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let path = self.access.path(&mut conn, scope).await?;
        let carried = auth
            .scopes
            .iter()
            .any(|s| s.permission == permission && path.iter().any(|p| p.id == s.scope));
        if !carried {
            return Ok(false);
        }
        Ok(self
            .access
            .check(&Actor::Principal(auth.account), permission, scope)
            .await?
            .allowed)
    }

    /// The scopes of a token.
    async fn token_scopes(&self, conn: &mut PgConnection, id: TokenId) -> Result<Vec<TokenScope>> {
        let rows = sqlx::query(sql(format!(
            "select permission_code, scope_id from {} where token_id = $1 order by permission_code, scope_id",
            self.table("api_token_scope")
        )))
        .bind(id.as_uuid())
        .fetch_all(conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|row| TokenScope {
                permission: row.get("permission_code"),
                scope: ScopeId::from_uuid(row.get("scope_id")),
            })
            .collect())
    }

    /// The tokens of the session's account, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn tokens(&self, session: &Session) -> Result<Vec<TokenSummary>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let rows = sqlx::query(sql(format!(
            "select id, name, prefix, created_at, expires_at, last_used_at, revoked_at from {} where account_id = $1
             order by created_at desc",
            self.table("api_token")
        )))
        .bind(session.account.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut tokens = Vec::with_capacity(rows.len());
        for row in &rows {
            let id = TokenId::from_uuid(row.get("id"));
            let instant = |name: &str| {
                row.get::<Option<time::OffsetDateTime>, _>(name)
                    .map(Timestamp::from_offset_date_time)
            };
            tokens.push(TokenSummary {
                id,
                name: row.get("name"),
                prefix: row.get("prefix"),
                created_at: Timestamp::from_offset_date_time(row.get("created_at")),
                expires_at: Timestamp::from_offset_date_time(row.get("expires_at")),
                last_used_at: instant("last_used_at"),
                revoked_at: instant("revoked_at"),
                scopes: self.token_scopes(&mut conn, id).await?,
            });
        }
        Ok(tokens)
    }

    /// Revokes one of the session's own tokens.
    ///
    /// # Errors
    ///
    /// `not_found` when the account has no such token.
    pub async fn revoke_own_token(&self, session: &Session, id: TokenId) -> Result<()> {
        self.revoke_token_of(Some(session.account), id).await
    }

    /// Revokes any token of a campus: needs `identity.token.manage_any` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `not_found` for an unknown token.
    pub async fn revoke_token(&self, actor: &Actor, id: TokenId) -> Result<()> {
        let campus: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select a.campus_id from {} t join {} a on a.id = t.account_id where t.id = $1",
            self.table("api_token"),
            self.table("account")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let campus = campus.ok_or_else(|| Error::not_found("identity.api_token_not_found"))?;
        self.access
            .require(actor, TOKEN_MANAGE_ANY, ScopeId::from_uuid(campus))
            .await?;
        self.revoke_token_of(None, id).await
    }

    /// Marks a token revoked, of one account when given.
    async fn revoke_token_of(&self, owner: Option<AccountId>, id: TokenId) -> Result<()> {
        let updated = sqlx::query(sql(format!(
            "update {} set revoked_at = coalesce(revoked_at, $3) where id = $1 and ($2::uuid is null or account_id = $2)",
            self.table("api_token")
        )))
        .bind(id.as_uuid())
        .bind(owner.map(|o| o.as_uuid()))
        .bind(self.now().as_offset_date_time())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if updated == 0 {
            Err(Error::not_found("identity.api_token_not_found"))
        } else {
            Ok(())
        }
    }
}
