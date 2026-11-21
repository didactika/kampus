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

//! Retention: a campus says how long each kind of personal data that an erasure keeps is kept after the
//! account is deleted; a job purges it when the time ends, unless a legal hold covers the account.

use std::collections::BTreeMap;

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result};
use kampus_module::{PrivacyRun, ServiceContext};
use serde_json::json;
use sqlx::Row;

use super::request::person;
use crate::permissions::RETENTION_MANAGE;
use crate::{Privacy, Retention, events, sql};

/// How many purges one run makes.
const BATCH: i64 = 100;

impl Privacy {
    /// The data categories the modules declare with the days the campus of the actor keeps each one after
    /// an account is deleted.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `privacy.retention.manage`.
    pub async fn retention(&self, actor: &Actor) -> Result<Vec<Retention>> {
        let me = person(actor)?;
        let campus = self.access.principal_campus(me).await?;
        if !self.access.check(actor, RETENTION_MANAGE, campus).await?.allowed {
            return Err(Error::permission_denied("access.denied").with_param("permission", RETENTION_MANAGE));
        }
        let rows = sqlx::query(sql(format!(
            "select category, keep_days from {} where campus_id = $1",
            self.table("retention_rule")
        )))
        .bind(campus.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let rules: BTreeMap<String, i32> = rows
            .iter()
            .map(|r| (r.get("category"), r.get("keep_days")))
            .collect();
        let mut found: Vec<Retention> = self
            .modules
            .iter()
            .flat_map(|m| m.privacy_handlers.iter())
            .map(|h| Retention {
                category: h.code.to_owned(),
                purpose: h.purpose.to_owned(),
                keep_days: rules.get(h.code).copied(),
            })
            .collect();
        found.sort_by(|a, b| a.category.cmp(&b.category));
        Ok(found)
    }

    /// Sets how many days the campus of the actor keeps a data category after an account is deleted, or
    /// keeps it for ever when no days are given.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `privacy.retention.manage`, `not_found` (`privacy.category_unknown`) for a
    /// category no module declares, `validation` for less than one day.
    pub async fn set_retention(
        &self,
        actor: &Actor,
        category: &str,
        keep_days: Option<i32>,
    ) -> Result<Vec<Retention>> {
        self.retention(actor).await?;
        if !self
            .modules
            .iter()
            .flat_map(|m| m.privacy_handlers.iter())
            .any(|h| h.code == category)
        {
            return Err(Error::not_found("privacy.category_unknown"));
        }
        if keep_days.is_some_and(|days| days < 1) {
            return Err(Error::validation(
                "privacy.keep_days_invalid",
                vec![FieldError::new("keep_days", "range")],
            ));
        }
        let campus = self.access.principal_campus(person(actor)?).await?;
        match keep_days {
            Some(days) => {
                sqlx::query(sql(format!(
                    "insert into {} (campus_id, category, keep_days) values ($1, $2, $3)
                     on conflict (campus_id, category) do update set keep_days = excluded.keep_days",
                    self.table("retention_rule")
                )))
                .bind(campus.as_uuid())
                .bind(category)
                .bind(days)
                .execute(self.db.pool())
                .await
            }
            None => {
                sqlx::query(sql(format!(
                    "delete from {} where campus_id = $1 and category = $2",
                    self.table("retention_rule")
                )))
                .bind(campus.as_uuid())
                .bind(category)
                .execute(self.db.pool())
                .await
            }
        }
        .map_err(kampus_db::classify)?;
        self.retention(actor).await
    }

    /// Purges the data of deleted accounts whose retention period ended, at most a batch per run, and
    /// returns how many purges it made. A legal hold on the account or its campus postpones its purge, and a
    /// handler that fails is left for the next run.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn retain(&self, services: &ServiceContext) -> Result<u64> {
        let due = sqlx::query(sql(format!(
            "select a.id, r.category from {rule} r
             join {account} a on a.campus_id = r.campus_id and a.status = 'deleted'
                 and a.deleted_at < now() - make_interval(days => r.keep_days)
             where not exists (select 1 from {purge} p where p.account_id = a.id and p.category = r.category)
                 and not {held}
             order by a.deleted_at, a.id limit $1",
            rule = self.table("retention_rule"),
            account = self.accounts(),
            purge = self.table("purge"),
            held = self.held_on("a.id"),
        )))
        .bind(BATCH)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut purged = 0;
        for row in &due {
            let account: uuid::Uuid = row.get("id");
            let category: String = row.get("category");
            let Some(handler) = self
                .modules
                .iter()
                .flat_map(|m| m.privacy_handlers.iter())
                .find(|h| h.code == category)
            else {
                continue;
            };
            let run = PrivacyRun {
                account,
                services,
                purge: true,
            };
            if (handler.erase)(run).await.is_err() {
                continue;
            }
            let mut tx = self.db.begin().await?;
            sqlx::query(sql(format!(
                "insert into {} (account_id, category) values ($1, $2) on conflict do nothing",
                self.table("purge")
            )))
            .bind(account)
            .bind(&category)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            let mut event = kampus_events::Publish::new(
                events::DATA_PURGED,
                Some(account),
                json!({ "account": PrincipalId::from_uuid(account), "category": category }),
            );
            event.summary = Some(format!("{} {category}", events::DATA_PURGED));
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, event)
                .await?;
            tx.commit().await.map_err(kampus_db::classify)?;
            purged += 1;
        }
        Ok(purged)
    }
}
