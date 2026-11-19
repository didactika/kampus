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

//! The audit log: append-only entries of the audited events, read by scope or by actor, and removed only
//! by their retention.

use kampus_common::{Result, Timestamp};
use serde::Serialize;
use sqlx::Row;

use crate::{Events, sql};

/// How long events that no audit entry keeps are retained, in days.
const EVENT_RETENTION_DAYS: i32 = 90;

/// How long audit entries, and the events they keep, are retained, in days.
const AUDIT_RETENTION_DAYS: i32 = 730;

/// An entry of the audit log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditEntry {
    /// The event it records.
    pub event: uuid::Uuid,
    /// Its type.
    pub code: String,
    /// The scope it happened in.
    pub scope: Option<uuid::Uuid>,
    /// Who did it.
    pub actor: Option<uuid::Uuid>,
    /// What it was about.
    pub object: Option<uuid::Uuid>,
    /// What happened.
    pub summary: String,
    /// When.
    pub at: Timestamp,
}

/// What was purged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Purged {
    /// Audit entries past their retention.
    pub audit_entries: u64,
    /// Events past their retention, with their deliveries.
    pub events: u64,
}

impl Events {
    /// The latest audit entries of a scope, of an actor, or of both, newest first. Who may read them is
    /// decided by the caller, as for the objects they are about.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn audit(
        &self,
        scope: Option<uuid::Uuid>,
        actor: Option<uuid::Uuid>,
        limit: i64,
    ) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query(sql(format!(
            "select a.event_id, e.type_code, a.scope, a.actor, a.object, a.summary, a.at
             from {} a join {} e on e.id = a.event_id
             where ($1::uuid is null or a.scope = $1) and ($2::uuid is null or a.actor = $2)
             order by a.at desc, a.id desc limit $3",
            self.table("audit_entry"),
            self.table("event")
        )))
        .bind(scope)
        .bind(actor)
        .bind(limit.clamp(1, 500))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| AuditEntry {
                event: r.get("event_id"),
                code: r.get("type_code"),
                scope: r.get("scope"),
                actor: r.get("actor"),
                object: r.get("object"),
                summary: r.get("summary"),
                at: Timestamp::from_offset_date_time(r.get("at")),
            })
            .collect())
    }

    /// Removes audit entries and events past their retention; events still waiting for a delivery stay.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn purge(&self) -> Result<Purged> {
        let mut tx = self.db.begin().await?;
        sqlx::query(sql("set local kampus.audit_retention = 'on'".to_owned()))
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        let audit_entries = sqlx::query(sql(format!(
            "delete from {} where at < now() - make_interval(days => $1)",
            self.table("audit_entry")
        )))
        .bind(AUDIT_RETENTION_DAYS)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        let events = sqlx::query(sql(format!(
            "delete from {events} e where e.occurred_at < now() - make_interval(days => $1)
             and not exists (select 1 from {audit} a where a.event_id = e.id)
             and not exists (select 1 from {deliveries} d where d.event_id = e.id and d.state = 'pending')
             and not exists (select 1 from {outbox} o where o.event_id = e.id and o.dispatched_at is null)",
            events = self.table("event"),
            audit = self.table("audit_entry"),
            deliveries = self.table("delivery"),
            outbox = self.table("outbox")
        )))
        .bind(EVENT_RETENTION_DAYS)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(Purged {
            audit_entries,
            events,
        })
    }
}
