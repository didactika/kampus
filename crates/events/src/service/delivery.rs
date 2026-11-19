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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Delivery: the dispatch turns each committed event into one delivery per listener of its type, and
//! deliveries run after commit, at least once, in order for each object and listener, with retries and
//! back-off; a delivery that runs out of attempts is dead, visible and replayable.

use kampus_common::{Error, Result, Timestamp};
use kampus_module::Event;
use serde::Serialize;
use sqlx::Row;

use crate::{Events, sql};

/// How many times a delivery runs before it is dead.
pub const MAX_ATTEMPTS: i32 = 8;

/// The longest wait between two attempts, in seconds.
const LONGEST_WAIT: f64 = 3600.0;

/// How long one run of a listener may take, in seconds.
pub const TIMEOUT_SECONDS: i32 = 60;

/// A delivery a runner claimed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Claimed {
    /// The listener.
    pub(crate) listener: uuid::Uuid,
    /// Its module.
    pub(crate) module: String,
    /// Its name.
    pub(crate) name: String,
    /// The event.
    pub(crate) event: Event,
    /// This attempt's number, from 1.
    pub(crate) attempt: i32,
}

/// A delivery as operators see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Delivery {
    /// The module of the listener.
    pub module: String,
    /// The name of the listener.
    pub listener: String,
    /// The event.
    pub event: uuid::Uuid,
    /// The type of the event.
    pub code: String,
    /// `pending`, `done` or `dead`.
    pub state: String,
    /// How many times it ran.
    pub attempts: i32,
    /// The error of its last attempt.
    pub last_error: Option<String>,
}

impl Events {
    /// Turns the committed events of the outbox into deliveries for every listener of their type or of
    /// `events.any`, and marks them dispatched; returns how many events it dispatched.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn dispatch(&self) -> Result<u64> {
        let mut tx = self.db.begin().await?;
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select event_id from {} where dispatched_at is null order by created_at, event_id limit 200
             for update skip locked",
            self.table("outbox")
        )))
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if ids.is_empty() {
            return Ok(0);
        }
        sqlx::query(sql(format!(
            "insert into {deliveries} (listener_id, event_id, object, occurred_at)
             select l.id, e.id, e.object, e.occurred_at from {events} e
             join {listeners} l on l.type_code in (e.type_code, 'events.any') and l.retired_at is null
             where e.id = any($1) on conflict do nothing",
            deliveries = self.table("delivery"),
            events = self.table("event"),
            listeners = self.table("listener")
        )))
        .bind(&ids)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let dispatched = sqlx::query(sql(format!(
            "update {} set dispatched_at = now() where event_id = any($1)",
            self.table("outbox")
        )))
        .bind(&ids)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(dispatched)
    }

    /// Claims the next delivery this process can run: due, of an enabled listener it has, not leased, and
    /// with no earlier delivery to the same listener about the same object still waiting.
    pub(crate) async fn claim(&self, listeners: &[uuid::Uuid]) -> Result<Option<Claimed>> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "update {deliveries} d set attempts = d.attempts + 1,
                 lease_until = now() + make_interval(secs => {TIMEOUT_SECONDS})
             where (d.listener_id, d.event_id) = (
                 select q.listener_id, q.event_id from {deliveries} q
                 join {listeners} l on l.id = q.listener_id and l.enabled
                 where q.state = 'pending' and q.next_attempt_at <= now() and q.listener_id = any($1)
                 and (q.lease_until is null or q.lease_until < now())
                 and not exists (
                     select 1 from {deliveries} p where p.listener_id = q.listener_id and p.state = 'pending'
                     and p.object = q.object and (p.occurred_at, p.event_id) < (q.occurred_at, q.event_id))
                 order by q.next_attempt_at, q.occurred_at limit 1 for update of q skip locked)
             returning d.listener_id, d.event_id, d.attempts",
            deliveries = self.table("delivery"),
            listeners = self.table("listener")
        )))
        .bind(listeners)
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let listener: uuid::Uuid = row.get("listener_id");
        let event_id: uuid::Uuid = row.get("event_id");
        let detail = sqlx::query(sql(format!(
            "select l.module, l.name, e.type_code, e.type_version, e.scope, e.actor, e.object, e.payload,
             e.occurred_at, e.depth from {} e, {} l where e.id = $1 and l.id = $2",
            self.table("event"),
            self.table("listener")
        )))
        .bind(event_id)
        .bind(listener)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(Some(Claimed {
            listener,
            module: detail.get("module"),
            name: detail.get("name"),
            event: Event {
                id: event_id,
                code: detail.get("type_code"),
                version: u16::try_from(detail.get::<i32, _>("type_version")).unwrap_or(1),
                scope: detail.get("scope"),
                actor: detail.get("actor"),
                object: detail.get("object"),
                payload: detail.get("payload"),
                occurred_at: Timestamp::from_offset_date_time(detail.get("occurred_at")),
                depth: u8::try_from(detail.get::<i32, _>("depth")).unwrap_or(u8::MAX),
            },
            attempt: row.get("attempts"),
        }))
    }

    /// Records how a delivery ended: done, retried after a wait that doubles, or dead after its last
    /// attempt.
    pub(crate) async fn finish(&self, delivery: &Claimed, error: Option<String>) -> Result<()> {
        let deliveries = self.table("delivery");
        let statement = match &error {
            None => format!(
                "update {deliveries} set state = 'done', done_at = now(), lease_until = null, last_error = null
                 where listener_id = $1 and event_id = $2"
            ),
            Some(_) if delivery.attempt >= MAX_ATTEMPTS => format!(
                "update {deliveries} set state = 'dead', lease_until = null, last_error = $3
                 where listener_id = $1 and event_id = $2"
            ),
            Some(_) => format!(
                "update {deliveries} set lease_until = null, last_error = $3,
                 next_attempt_at = now() + make_interval(secs => $4::double precision)
                 where listener_id = $1 and event_id = $2"
            ),
        };
        let wait = 2_f64.powi(delivery.attempt).min(LONGEST_WAIT);
        sqlx::query(sql(statement))
            .bind(delivery.listener)
            .bind(delivery.event.id)
            .bind(&error)
            .bind(wait)
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        if delivery.attempt >= MAX_ATTEMPTS
            && let Some(error) = error
        {
            tracing::error!(
                module = delivery.module,
                listener = delivery.name,
                event = %delivery.event.id,
                error,
                "a delivery is dead after its last attempt"
            );
        }
        Ok(())
    }

    /// The deliveries in a state, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn deliveries(&self, state: &str, limit: i64) -> Result<Vec<Delivery>> {
        let rows = sqlx::query(sql(format!(
            "select l.module, l.name, d.event_id, e.type_code, d.state, d.attempts, d.last_error
             from {} d join {} l on l.id = d.listener_id join {} e on e.id = d.event_id
             where d.state = $1 order by d.occurred_at desc limit $2",
            self.table("delivery"),
            self.table("listener"),
            self.table("event")
        )))
        .bind(state)
        .bind(limit.clamp(1, 500))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Delivery {
                module: r.get("module"),
                listener: r.get("name"),
                event: r.get("event_id"),
                code: r.get("type_code"),
                state: r.get("state"),
                attempts: r.get("attempts"),
                last_error: r.get("last_error"),
            })
            .collect())
    }

    /// Runs the dead deliveries again, all of them or those of one event; returns how many.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn replay(&self, event: Option<uuid::Uuid>) -> Result<u64> {
        sqlx::query(sql(format!(
            "update {} set state = 'pending', attempts = 0, next_attempt_at = now(), last_error = null
             where state = 'dead' and ($1::uuid is null or event_id = $1)",
            self.table("delivery")
        )))
        .bind(event)
        .execute(self.db.pool())
        .await
        .map(|r| r.rows_affected())
        .map_err(kampus_db::classify)
    }

    /// Turns a listener on or off; while off, its deliveries wait, and are not lost.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown listener.
    pub async fn set_listener(&self, module: &str, name: &str, enabled: bool) -> Result<()> {
        let changed = sqlx::query(sql(format!(
            "update {} set enabled = $3 where module = $1 and name = $2",
            self.table("listener")
        )))
        .bind(module)
        .bind(name)
        .bind(enabled)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            return Err(Error::not_found("events.listener_not_found"));
        }
        Ok(())
    }
}
