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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The personal data progress keeps and how to export it, for the privacy module.

use kampus_access::PrincipalId;
use kampus_common::Result;
use kampus_module::{PrivacyFuture, PrivacyHandlerDeclaration, PrivacyRun};
use serde_json::{Value, json};

use crate::{Progress, sql};

/// The handlers of this module.
pub const HANDLERS: &[PrivacyHandlerDeclaration] = &[PrivacyHandlerDeclaration {
    code: "progress.activity",
    purpose: "What the person has done on course modules and decisions about it",
    export,
    erase,
}];

/// Keeps the data, which is part of the academic record and survives pointing to the tombstone of the
/// account, until its retention period ends and it is purged.
fn erase(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        if run.purge {
            run.services
                .service::<Progress>()?
                .purge_personal_data(PrincipalId::from_uuid(run.account))
                .await
        } else {
            Ok(json!({ "kept": "academic record" }))
        }
    })
}

/// Exports the data of the account of a request.
fn export(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        run.services
            .service::<Progress>()?
            .personal_data(PrincipalId::from_uuid(run.account))
            .await
    })
}

impl Progress {
    /// Deletes what this module keeps about an account, once its retention period ended; returns how many
    /// rows of each table.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn purge_personal_data(&self, account: PrincipalId) -> Result<Value> {
        let mut tx = self.db.begin().await?;
        let mut report = serde_json::Map::new();
        for table in [
            "activity_history",
            "activity_exception",
            "activity_state",
            "signal",
        ] {
            let deleted = sqlx::query(sql(format!(
                "delete from {} where account_id = $1",
                self.table(table)
            )))
            .bind(account.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
            report.insert(table.to_owned(), json!(deleted));
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(json!({ "purged": report }))
    }

    /// Everything this module keeps about an account, as JSON by table.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn personal_data(&self, account: PrincipalId) -> Result<Value> {
        let mut data = serde_json::Map::new();
        data.insert(
            "activity_state".to_owned(),
            self.rows_of("activity_state", "account_id", account).await?,
        );
        data.insert(
            "signal".to_owned(),
            self.rows_of("signal", "account_id", account).await?,
        );
        data.insert(
            "activity_exception".to_owned(),
            self.rows_of("activity_exception", "account_id", account).await?,
        );
        data.insert(
            "activity_history".to_owned(),
            self.rows_of("activity_history", "account_id", account).await?,
        );
        Ok(json!(data))
    }

    /// The rows of a table that belong to an account, as a JSON list.
    async fn rows_of(&self, table: &str, column: &str, account: PrincipalId) -> Result<Value> {
        sqlx::query_scalar(sql(format!(
            "select coalesce(jsonb_agg(to_jsonb(t)), '[]'::jsonb) from {} t where t.\"{column}\" = $1",
            self.table(table)
        )))
        .bind(account.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }
}
