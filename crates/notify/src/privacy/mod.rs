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

//! The personal data notify keeps and how to export it, for the privacy module.

use kampus_access::PrincipalId;
use kampus_common::Result;
use kampus_module::{PrivacyFuture, PrivacyHandlerDeclaration, PrivacyRun};
use serde_json::{Value, json};

use crate::{Notify, sql};

/// The handlers of this module.
pub const HANDLERS: &[PrivacyHandlerDeclaration] = &[PrivacyHandlerDeclaration {
    code: "notify.notifications",
    purpose: "Notifications and choices about them",
    export,
}];

/// Exports the data of the account of a request.
fn export(run: PrivacyRun<'_>) -> PrivacyFuture<'_> {
    Box::pin(async move {
        run.services
            .service::<Notify>()?
            .personal_data(PrincipalId::from_uuid(run.account))
            .await
    })
}

impl Notify {
    /// Everything this module keeps about an account, as JSON by table.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn personal_data(&self, account: PrincipalId) -> Result<Value> {
        let mut data = serde_json::Map::new();
        data.insert(
            "notification".to_owned(),
            self.rows_of("notification", "recipient_id", account).await?,
        );
        data.insert(
            "preference".to_owned(),
            self.rows_of("preference", "account_id", account).await?,
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
