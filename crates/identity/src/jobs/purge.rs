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

//! Purges what identity keeps only for a while: used or expired one-time tokens, expired sessions, and
//! sign-in attempts older than the retention of their campus.

use kampus_access::ScopeId;
use kampus_common::Result;

use crate::{Identity, sql};

/// What one purge removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Purged {
    /// One-time tokens.
    pub tokens: u64,
    /// Sessions.
    pub sessions: u64,
    /// Sign-in attempts.
    pub attempts: u64,
}

impl Identity {
    /// Removes what has expired.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn purge(&self) -> Result<Purged> {
        let now = self.now().as_offset_date_time();
        let pool = self.db.pool();
        let tokens = sqlx::query(sql(format!(
            "delete from {} where expires_at <= $1 or used_at is not null",
            self.table("one_time_token")
        )))
        .bind(now)
        .execute(pool)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        let sessions = sqlx::query(sql(format!(
            "delete from {} where expires_at <= $1",
            self.table("session")
        )))
        .bind(now)
        .execute(pool)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        let campuses: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct campus_id from {}",
            self.table("sign_in_attempt")
        )))
        .fetch_all(pool)
        .await
        .map_err(kampus_db::classify)?;
        let mut attempts = 0;
        for campus in campuses {
            let policy = self.policies.policy(ScopeId::from_uuid(campus)).await?;
            attempts += sqlx::query(sql(format!(
                "delete from {} where campus_id = $1 and at < $2",
                self.table("sign_in_attempt")
            )))
            .bind(campus)
            .bind(now - policy.attempt_retention)
            .execute(pool)
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
        }
        Ok(Purged {
            tokens,
            sessions,
            attempts,
        })
    }
}
