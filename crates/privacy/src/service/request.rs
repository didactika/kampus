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

//! Requests: a person asks for an export of their own data, or someone who handles privacy asks for another
//! person's; the person who asked reads the package until it expires.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, Result, Timestamp};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::permissions::REQUEST_MANAGE;
use crate::{ANSWER_WITHIN, Item, Privacy, Request, RequestId, events, sql};

/// The person behind an actor.
fn person(actor: &Actor) -> Result<PrincipalId> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Ok(*p),
        Actor::Visitor(_) => Err(Error::permission_denied("access.denied")),
    }
}

/// A request from its row, without its parts.
fn request(row: &PgRow) -> Request {
    let stamp = |column: &str| {
        row.get::<Option<time::OffsetDateTime>, _>(column)
            .map(Timestamp::from_offset_date_time)
    };
    Request {
        id: RequestId::from_uuid(row.get("id")),
        account: PrincipalId::from_uuid(row.get("account_id")),
        kind: row.get("kind"),
        state: row.get("state"),
        created_at: Timestamp::from_offset_date_time(row.get("created_at")),
        due_at: Timestamp::from_offset_date_time(row.get("due_at")),
        completed_at: stamp("completed_at"),
        expires_at: stamp("expires_at"),
        items: Vec::new(),
    }
}

/// The columns of a request.
const COLUMNS: &str = "id, account_id, requester, kind, state, created_at, due_at, completed_at, expires_at";

impl Privacy {
    /// Whether the actor may act on the requests about an account: it is their own, or the actor holds
    /// `privacy.request.manage` in the campus of the account.
    async fn may_handle(&self, actor: &Actor, account: PrincipalId) -> Result<bool> {
        if person(actor)? == account {
            return Ok(true);
        }
        let campus = self.access.principal_campus(account).await?;
        Ok(self.access.check(actor, REQUEST_MANAGE, campus).await?.allowed)
    }

    /// Asks for an export of the personal data of an account: one's own, or another's with
    /// `privacy.request.manage` in their campus. Every data category the modules declare becomes a part,
    /// answered by a job.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a request about someone else without the permission.
    pub async fn request_export(&self, actor: &Actor, account: PrincipalId) -> Result<Request> {
        let by = person(actor)?;
        if !self.may_handle(actor, account).await? {
            return Err(Error::permission_denied("access.denied").with_param("permission", REQUEST_MANAGE));
        }
        let id = RequestId::generate();
        let due = Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc() + ANSWER_WITHIN);
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, requester, kind, due_at) values ($1, $2, $3, 'export', $4)",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .bind(account.as_uuid())
        .bind(by.as_uuid())
        .bind(due.as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for handler in self.modules.iter().flat_map(|m| m.privacy_handlers.iter()) {
            sqlx::query(sql(format!(
                "insert into {} (request_id, category) values ($1, $2)",
                self.table("request_item")
            )))
            .bind(id.as_uuid())
            .bind(handler.code)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        let mut event = kampus_events::Publish::new(
            events::REQUEST_CREATED,
            Some(id.as_uuid()),
            json!({ "request": id, "account": account }),
        );
        event.actor = Some(by.as_uuid());
        event.summary = Some(format!("{} {id}", events::REQUEST_CREATED));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        kampus_tasks::Tasks::new(self.db.clone())
            .enqueue_in(&mut tx, "privacy.run", &json!({}), None)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.request(actor, id).await
    }

    /// A request with its parts, for whoever may handle it.
    ///
    /// # Errors
    ///
    /// `not_found` (`privacy.request_not_found`) for a request the actor may not see.
    pub async fn request(&self, actor: &Actor, id: RequestId) -> Result<Request> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("privacy.request_not_found"))?;
        let mut found = request(&row);
        let requester: Option<uuid::Uuid> = row.get("requester");
        let own = person(actor).is_ok_and(|p| Some(p.as_uuid()) == requester);
        if !own && !self.may_handle(actor, found.account).await.unwrap_or(false) {
            return Err(Error::not_found("privacy.request_not_found"));
        }
        let items = sqlx::query(sql(format!(
            "select category, state, error from {} where request_id = $1 order by category",
            self.table("request_item")
        )))
        .bind(id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        found.items = items
            .iter()
            .map(|r| Item {
                category: r.get("category"),
                state: r.get("state"),
                error: r.get("error"),
            })
            .collect();
        Ok(found)
    }

    /// The requests the actor made, newest first.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn requests(&self, actor: &Actor) -> Result<Vec<Request>> {
        let me = person(actor)?;
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where requester = $1 order by created_at desc, id desc limit 100",
            self.table("request")
        )))
        .bind(me.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(request).collect())
    }

    /// The exported data, by category, for the person who asked, while it is available.
    ///
    /// # Errors
    ///
    /// `not_found` for a request that is not theirs, `conflict` (`privacy.package_not_ready`) while parts
    /// are pending or failed, (`privacy.package_expired`) after its time.
    pub async fn package(&self, actor: &Actor, id: RequestId) -> Result<Value> {
        let me = person(actor)?;
        let found = self.request(actor, id).await?;
        let requester: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select requester from {} where id = $1",
            self.table("request")
        )))
        .bind(id.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if requester != Some(me.as_uuid()) {
            return Err(Error::not_found("privacy.request_not_found"));
        }
        match found.state.as_str() {
            "completed" => {}
            "expired" => return Err(Error::conflict("privacy.package_expired")),
            _ => return Err(Error::conflict("privacy.package_not_ready")),
        }
        let rows = sqlx::query(sql(format!(
            "select category, data from {} where request_id = $1 order by category",
            self.table("request_item")
        )))
        .bind(id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut data = serde_json::Map::new();
        for row in &rows {
            data.insert(
                row.get("category"),
                row.get::<Option<Value>, _>("data").unwrap_or(Value::Null),
            );
        }
        Ok(json!({ "request": id, "account": found.account, "data": data }))
    }
}
