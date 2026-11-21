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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The personal data records keeps and how to export it, for the privacy module.

use kampus_access::PrincipalId;
use kampus_common::Result;
use kampus_module::{PrivacyFuture, PrivacyHandlerDeclaration, PrivacyRun};
use serde_json::{Value, json};

use crate::{Records, sql};

/// The handlers of this module.
pub const HANDLERS: &[PrivacyHandlerDeclaration] = &[PrivacyHandlerDeclaration {
    code: "records.grades",
    purpose: "Grades and the history of the changes to them",
    export,
    erase,
}];

/// Keeps the data, which is part of the academic record and survives pointing to the tombstone of the
/// account, until its retention period ends and it is purged.
fn erase(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        if run.purge {
            run.services
                .service::<Records>()?
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
            .service::<Records>()?
            .personal_data(PrincipalId::from_uuid(run.account))
            .await
    })
}

impl Records {
    /// Deletes what this module keeps about an account, once its retention period ended; returns how many
    /// rows of each table.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn purge_personal_data(&self, account: PrincipalId) -> Result<Value> {
        let mut tx = self.db.begin().await?;
        let mut report = serde_json::Map::new();
        for (table, column) in [
            ("entry", "account_id"),
            ("grade", "account_id"),
            ("recalculation", "account_id"),
        ] {
            let deleted = sqlx::query(sql(format!(
                "delete from {} where \"{column}\" = $1",
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
            "grade".to_owned(),
            self.rows_of("grade", "account_id", account).await?,
        );
        data.insert(
            "entry".to_owned(),
            self.rows_of("entry", "account_id", account).await?,
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
