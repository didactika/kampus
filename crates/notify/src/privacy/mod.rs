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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The personal data notify keeps and how to export and erase it, for the privacy module.

use kampus_access::PrincipalId;
use kampus_common::Result;
use kampus_module::{PrivacyFuture, PrivacyHandlerDeclaration, PrivacyRun};
use serde_json::{Value, json};

use crate::{Notify, sql};

/// The handlers of this module.
pub const HANDLERS: &[PrivacyHandlerDeclaration] = &[PrivacyHandlerDeclaration {
    code: "notify.notifications",
    purpose: "Notifications, choices about them, devices, conversations, messages and blocks",
    export,
    erase,
}];

/// Erases the data of the account of a request.
fn erase(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        run.services
            .service::<Notify>()?
            .erase_personal_data(PrincipalId::from_uuid(run.account))
            .await
    })
}

/// Exports the data of the account of a request.
fn export(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        run.services
            .service::<Notify>()?
            .personal_data(PrincipalId::from_uuid(run.account))
            .await
    })
}

/// What erasing an account deletes besides its messages, which go with their files first: by name in the report,
/// table and column.
const ERASED: &[(&str, &str, &str)] = &[
    ("notification", "notification", "recipient_id"),
    ("preference", "preference", "account_id"),
    ("push_subscription", "push_subscription", "account_id"),
    ("message_deletion", "message_deletion", "account_id"),
    ("member", "member", "account_id"),
    ("block", "block", "account_id"),
    ("blocked_by", "block", "blocked_id"),
];

/// What exporting an account reads, by name, table and column.
const EXPORTED: &[(&str, &str, &str)] = &[
    ("notification", "notification", "recipient_id"),
    ("preference", "preference", "account_id"),
    ("push_subscription", "push_subscription", "account_id"),
    ("member", "member", "account_id"),
    ("message", "message", "author_id"),
    ("block", "block", "account_id"),
];

impl Notify {
    /// Deletes the notifications, choices, devices, messages, memberships and blocks of an account; returns how
    /// many of each.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn erase_personal_data(&self, account: PrincipalId) -> Result<Value> {
        let authored: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select id from {} where author_id = $1",
            self.table("message")
        )))
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let messages = self.remove_messages(&authored).await?;
        let mut tx = self.db.begin().await?;
        let mut report = serde_json::Map::new();
        report.insert("message".to_owned(), json!(messages));
        for (key, table, column) in ERASED {
            let deleted = sqlx::query(sql(format!(
                "delete from {} where \"{column}\" = $1",
                self.table(table)
            )))
            .bind(account.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
            report.insert((*key).to_owned(), json!(deleted));
        }
        sqlx::query(sql(format!(
            "update {} set created_by = null where created_by = $1",
            self.table("conversation")
        )))
        .bind(account.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(json!({ "deleted": report }))
    }

    /// Everything this module keeps about an account, as JSON by table.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn personal_data(&self, account: PrincipalId) -> Result<Value> {
        let mut data = serde_json::Map::new();
        for (key, table, column) in EXPORTED {
            data.insert((*key).to_owned(), self.rows_of(table, column, account).await?);
        }
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
