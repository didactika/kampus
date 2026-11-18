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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The history of grants, overrides and roles (RN-015): who changed what, when, and the value before.

use kampus_common::Result;
use serde_json::Value as Json;
use sqlx::PgConnection;

use crate::{Access, PrincipalId, sql};

impl Access {
    /// Records one change in the transaction of the change itself.
    pub(crate) async fn record(
        &self,
        conn: &mut PgConnection,
        actor: Option<PrincipalId>,
        entity: &str,
        key: &str,
        before: Option<Json>,
        after: Option<Json>,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, actor, entity, entity_key, before, after, at) values ($1, $2, $3, $4, $5, $6, $7)",
            self.table("change")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(actor.map(|a| a.as_uuid()))
        .bind(entity)
        .bind(key)
        .bind(before)
        .bind(after)
        .bind(self.now().as_offset_date_time())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }
}
