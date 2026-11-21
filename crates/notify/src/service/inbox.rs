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

//! The inbox: notifications are created for a person when the type is enabled for them, read newest first and
//! marked read one by one or all together.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, Result, Timestamp};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::person;
use crate::{EMAIL, INBOX, Inbox, Notification, NotificationId, Notify, PUSH, events, sql};

/// The most notifications in a page.
const PAGE_LIMIT: i64 = 100;

/// A notification from its row.
fn notification(row: &PgRow) -> Notification {
    Notification {
        id: NotificationId::from_uuid(row.get("id")),
        type_code: row.get("type_code"),
        recipient: PrincipalId::from_uuid(row.get("recipient_id")),
        payload: row.get("payload"),
        created_at: Timestamp::from_offset_date_time(row.get("created_at")),
        read_at: row
            .get::<Option<time::OffsetDateTime>, _>("read_at")
            .map(Timestamp::from_offset_date_time),
    }
}

impl Notify {
    /// Notifies a person of something that happened, on every channel they receive that type through: the
    /// inbox at once, email and push by a delivery job each. `dedupe` names what caused it, so a listener that
    /// runs twice for the same event notifies once. Returns the notification, or none when the person
    /// receives that type through no channel or it was already sent.
    ///
    /// # Errors
    ///
    /// `validation` (`notify.type_unknown`) for a type no module declares.
    pub async fn notify(
        &self,
        type_code: &str,
        recipient: PrincipalId,
        payload: Value,
        dedupe: Option<&str>,
    ) -> Result<Option<Notification>> {
        let reach = self.resolved(recipient, Some(type_code)).await?;
        if reach.is_empty() {
            return Err(Error::validation(
                "notify.type_unknown",
                vec![kampus_common::FieldError::new("type", "declared")],
            ));
        }
        let on = |channel: &str| reach.iter().any(|p| p.channel == channel && p.enabled);
        let sent: Vec<&str> = [EMAIL, PUSH].into_iter().filter(|c| on(c)).collect();
        if !on(INBOX) && sent.is_empty() {
            return Ok(None);
        }
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, type_code, recipient_id, dedupe, payload, in_inbox) values ($1, $2, $3, $4, $5, $6)
             on conflict (recipient_id, type_code, dedupe) do nothing
             returning id, type_code, recipient_id, payload, created_at, read_at",
            self.table("notification")
        )))
        .bind(NotificationId::generate().as_uuid())
        .bind(type_code)
        .bind(recipient.as_uuid())
        .bind(dedupe)
        .bind(&payload)
        .bind(on(INBOX))
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let created = notification(&row);
        for channel in &sent {
            sqlx::query(sql(format!(
                "insert into {} (notification_id, channel) values ($1, $2)",
                self.table("delivery")
            )))
            .bind(created.id.as_uuid())
            .bind(channel)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        if !sent.is_empty() {
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(&mut tx, "notify.deliver", &json!({}), None)
                .await?;
        }
        let mut event = kampus_events::Publish::new(
            events::DELIVERED,
            Some(created.id.as_uuid()),
            json!({ "notification": created.id, "type": type_code, "recipient": recipient }),
        );
        event.summary = Some(format!("{} {}", events::DELIVERED, created.id));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(Some(created))
    }

    /// A page of the inbox of the actor, newest first.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn inbox(
        &self,
        actor: &Actor,
        unread_only: bool,
        limit: i64,
        after: Option<NotificationId>,
    ) -> Result<Inbox> {
        let me = person(actor)?;
        let limit = limit.clamp(1, PAGE_LIMIT);
        let rows = sqlx::query(sql(format!(
            "select id, type_code, recipient_id, payload, created_at, read_at from {t}
             where recipient_id = $1 and in_inbox and ($2 = false or read_at is null)
             and ($3::uuid is null or (created_at, id) < (select created_at, id from {t} where id = $3))
             order by created_at desc, id desc limit $4",
            t = self.table("notification")
        )))
        .bind(me.as_uuid())
        .bind(unread_only)
        .bind(after.map(|a| a.as_uuid()))
        .bind(limit + 1)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let unread: i64 = sqlx::query_scalar(sql(format!(
            "select count(*) from {} where recipient_id = $1 and in_inbox and read_at is null",
            self.table("notification")
        )))
        .bind(me.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let more = i64::try_from(rows.len()).unwrap_or(i64::MAX) > limit;
        let notifications: Vec<Notification> = rows
            .iter()
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .map(notification)
            .collect();
        let next = if more {
            notifications.last().map(|n| n.id)
        } else {
            None
        };
        Ok(Inbox {
            notifications,
            unread,
            next,
        })
    }

    /// Marks a notification of the actor as read; reading it again changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found` (`notify.notification_not_found`) for a notification that is not theirs.
    pub async fn mark_read(&self, actor: &Actor, id: NotificationId) -> Result<Notification> {
        let me = person(actor)?;
        let row = sqlx::query(sql(format!(
            "update {} set read_at = coalesce(read_at, now()) where id = $1 and recipient_id = $2
             returning id, type_code, recipient_id, payload, created_at, read_at",
            self.table("notification")
        )))
        .bind(id.as_uuid())
        .bind(me.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("notify.notification_not_found"))?;
        Ok(notification(&row))
    }

    /// Marks every notification of the actor as read; returns how many it changed.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn mark_all_read(&self, actor: &Actor) -> Result<u64> {
        let me = person(actor)?;
        Ok(sqlx::query(sql(format!(
            "update {} set read_at = now() where recipient_id = $1 and read_at is null",
            self.table("notification")
        )))
        .bind(me.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected())
    }
}
