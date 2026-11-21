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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Answering the parts of requests: each part asks the handler of its data category, a failing handler is
//! recorded and retried on the next run, and a request is complete when every part is. Exported data is
//! forgotten when its time ends.

use kampus_access::PrincipalId;
use kampus_common::Result;
use kampus_module::{PrivacyRun, ServiceContext};
use serde_json::json;
use sqlx::Row;

use crate::{PACKAGE_LIFETIME, Privacy, RequestId, events, sql};

/// How many parts one run answers.
const BATCH: i64 = 50;

impl Privacy {
    /// Answers pending and failed parts, then completes the requests whose parts are all answered; returns
    /// how many parts it answered. A part whose handler fails is recorded and left for the next run.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn run(&self, services: &ServiceContext) -> Result<u64> {
        self.apply_holds().await?;
        let rows = sqlx::query(sql(format!(
            "select i.request_id, i.category, r.account_id, r.kind from {} i join {} r on r.id = i.request_id
             where i.state <> 'completed' and r.state = 'pending' order by r.created_at, i.category limit $1",
            self.table("request_item"),
            self.table("request")
        )))
        .bind(BATCH)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut answered = 0;
        for row in &rows {
            let request: uuid::Uuid = row.get("request_id");
            let category: String = row.get("category");
            let account: uuid::Uuid = row.get("account_id");
            let erasing = row.get::<String, _>("kind") == "deletion";
            let outcome = match self
                .modules
                .iter()
                .flat_map(|m| m.privacy_handlers.iter())
                .find(|h| h.code == category)
            {
                Some(handler) if erasing => {
                    (handler.erase)(PrivacyRun {
                        account,
                        services,
                        purge: false,
                    })
                    .await
                }
                Some(handler) => {
                    (handler.export)(PrivacyRun {
                        account,
                        services,
                        purge: false,
                    })
                    .await
                }
                None => Err(kampus_common::Error::not_found("privacy.handler_missing")),
            };
            let (state, data, error) = match outcome {
                Ok(data) => ("completed", Some(data), None),
                Err(error) => ("failed", None, Some(error.code().to_owned())),
            };
            sqlx::query(sql(format!(
                "update {} set state = $3, data = $4, error = $5 where request_id = $1 and category = $2",
                self.table("request_item")
            )))
            .bind(request)
            .bind(&category)
            .bind(state)
            .bind(data)
            .bind(error)
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
            answered += u64::from(state == "completed");
        }
        self.complete_requests(services).await?;
        Ok(answered)
    }

    /// The condition of an account, named by an expression, covered by an active hold on it or on its campus.
    pub(crate) fn held_on(&self, account: &str) -> String {
        format!(
            "exists (select 1 from {h} h where h.released_at is null and (h.until is null or h.until > now())
                 and (h.account_id = {account}
                      or h.campus_id = (select campus_id from {p} where id = {account})))",
            h = self.table("legal_hold"),
            p = self.principals()
        )
    }

    /// Puts the deletions under a legal hold to wait, and lets the ones whose hold ended go on.
    async fn apply_holds(&self) -> Result<()> {
        let held = self.held_on("r.account_id");
        for (from, to, negate) in [("pending", "held", ""), ("held", "pending", "not ")] {
            sqlx::query(sql(format!(
                "update {r} r set state = '{to}' where r.kind = 'deletion' and r.state = '{from}' and {negate}{held}",
                r = self.table("request")
            )))
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(())
    }

    /// Completes the deletions whose parts are all erased: the account becomes a tombstone.
    async fn complete_deletions(&self, services: &ServiceContext) -> Result<()> {
        let due = sqlx::query(sql(format!(
            "select id, account_id from {r} r where kind = 'deletion' and state = 'pending' and not exists
                 (select 1 from {i} where request_id = r.id and state <> 'completed')",
            r = self.table("request"),
            i = self.table("request_item")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if due.is_empty() {
            return Ok(());
        }
        let identity = services.service::<kampus_identity::Identity>()?;
        for row in &due {
            let id = RequestId::from_uuid(row.get("id"));
            let account = PrincipalId::from_uuid(row.get("account_id"));
            identity
                .erase_account(kampus_identity::AccountId::from_uuid(account.as_uuid()))
                .await?;
            let mut tx = self.db.begin().await?;
            let changed = sqlx::query(sql(format!(
                "update {} set state = 'completed', completed_at = now() where id = $1 and state = 'pending'",
                self.table("request")
            )))
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
            if changed > 0 {
                for code in [events::REQUEST_COMPLETED, events::ACCOUNT_ERASED] {
                    let mut event = kampus_events::Publish::new(
                        code,
                        Some(id.as_uuid()),
                        json!({ "request": id, "account": account }),
                    );
                    event.summary = Some(format!("{code} {id}"));
                    kampus_events::Events::new(self.db.clone())
                        .publish_in(&mut tx, event)
                        .await?;
                }
            }
            tx.commit().await.map_err(kampus_db::classify)?;
        }
        Ok(())
    }

    /// Completes the requests whose parts are all answered.
    async fn complete_requests(&self, services: &ServiceContext) -> Result<()> {
        self.complete_deletions(services).await?;
        let mut tx = self.db.begin().await?;
        let rows = sqlx::query(sql(format!(
            "update {r} set state = 'completed', completed_at = now(), expires_at = $1
             where state = 'pending' and kind = 'export' and not exists
                 (select 1 from {i} where request_id = {r}.id and state <> 'completed')
             returning id, account_id",
            r = self.table("request"),
            i = self.table("request_item")
        )))
        .bind(time::OffsetDateTime::now_utc() + PACKAGE_LIFETIME)
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for row in &rows {
            let id = RequestId::from_uuid(row.get("id"));
            let account = PrincipalId::from_uuid(row.get("account_id"));
            let mut event = kampus_events::Publish::new(
                events::REQUEST_COMPLETED,
                Some(id.as_uuid()),
                json!({ "request": id, "account": account }),
            );
            event.summary = Some(format!("{} {id}", events::REQUEST_COMPLETED));
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, event)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Forgets the data of the requests whose time ended; returns how many requests expired.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn expire(&self) -> Result<u64> {
        let mut tx = self.db.begin().await?;
        let expired: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "update {} set state = 'expired' where state = 'completed' and expires_at < now() returning id",
            self.table("request")
        )))
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "update {} set data = null where request_id = any($1)",
            self.table("request_item")
        )))
        .bind(&expired)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(u64::try_from(expired.len()).unwrap_or(u64::MAX))
    }
}
