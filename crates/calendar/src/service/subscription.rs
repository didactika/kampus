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
// @package   kampus-calendar
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Subscriptions: a scope shows the events of a calendar of another system, read again by a job. Its events
//! are matched by UID: new ones are added, changed ones replaced and missing ones removed, and each read is
//! recorded and published. Only public network addresses are read, without following redirects, and a
//! calendar larger than a limit is refused.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::Duration;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::entry::from_jiff;
use super::person;
use crate::permissions::MANAGE;
use crate::support::ical::{self, Event, MAX_CALENDAR};
use crate::{Calendar, Subscription, SubscriptionId, events, sql};

/// How many subscriptions one run reads.
const BATCH: i64 = 20;

/// How long a read may take.
const TIMEOUT: Duration = Duration::from_secs(20);

/// The longest address, in characters.
const URL_LIMIT: usize = 2000;

/// The columns of a subscription.
const COLUMNS: &str = "id, scope_id, url, title, refresh_minutes, last_refreshed_at, last_error";

/// What a read changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct Refreshed {
    /// Events added.
    pub added: u64,
    /// Events changed.
    pub changed: u64,
    /// Events removed.
    pub removed: u64,
}

/// A subscription from its row.
fn subscription(row: &PgRow) -> Subscription {
    Subscription {
        id: SubscriptionId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        url: row.get("url"),
        title: row.get("title"),
        refresh_minutes: row.get("refresh_minutes"),
        last_refreshed_at: row
            .get::<Option<time::OffsetDateTime>, _>("last_refreshed_at")
            .map(Timestamp::from_offset_date_time),
        last_error: row.get("last_error"),
    }
}

/// A wrong field of a subscription.
fn wrong(field: &str, rule: &str) -> Error {
    Error::validation(
        "calendar.subscription_invalid",
        vec![FieldError::new(field, rule)],
    )
}

/// Whether an address is on the public internet: not loopback, private, link-local, shared, documentation,
/// multicast or unspecified.
fn public(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_documentation()
                || a == 0
                || (a == 100 && (64..128).contains(&b)))
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            !(v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || v6.to_ipv4_mapped().is_some_and(|v4| !public(IpAddr::V4(v4))))
        }
    }
}

impl Calendar {
    /// Subscribes a scope to a calendar of another system; needs `calendar.manage` there. It is read now by
    /// a job and then every so often.
    ///
    /// # Errors
    ///
    /// `permission_denied`, `validation` (`calendar.subscription_invalid`) for an address that is not http
    /// or https, a missing title or a refresh under fifteen minutes, `conflict` for an address already there.
    pub async fn subscribe(
        &self,
        actor: &Actor,
        scope: ScopeId,
        url: &str,
        title: &str,
        refresh_minutes: Option<i32>,
    ) -> Result<Subscription> {
        self.access.require(actor, MANAGE, scope).await?;
        let url = url.trim();
        let parsed = reqwest::Url::parse(url).map_err(|_| wrong("url", "format"))?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || url.chars().count() > URL_LIMIT
            || !parsed.username().is_empty()
        {
            return Err(wrong("url", "format"));
        }
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 200 {
            return Err(wrong("title", "length"));
        }
        let refresh = refresh_minutes.unwrap_or(360);
        if !(15..=10_080).contains(&refresh) {
            return Err(wrong("refresh_minutes", "range"));
        }
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, scope_id, url, title, refresh_minutes, created_by) values ($1, $2, $3, $4, $5, $6)
             returning {COLUMNS}",
            self.table("subscription")
        )))
        .bind(SubscriptionId::generate().as_uuid())
        .bind(scope.as_uuid())
        .bind(url)
        .bind(title)
        .bind(refresh)
        .bind(person(actor).map(|p| p.as_uuid()))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        kampus_tasks::Tasks::new(self.db.clone())
            .enqueue_in(&mut tx, "calendar.refresh", &json!({}), None)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(subscription(&row))
    }

    /// The subscriptions of a scope, for whoever manages its calendar.
    ///
    /// # Errors
    ///
    /// `permission_denied`.
    pub async fn subscriptions(&self, actor: &Actor, scope: ScopeId) -> Result<Vec<Subscription>> {
        self.access.require(actor, MANAGE, scope).await?;
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where scope_id = $1 order by title, id",
            self.table("subscription")
        )))
        .bind(scope.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(subscription).collect())
    }

    /// A subscription, whoever asks.
    async fn stored_subscription(&self, id: SubscriptionId) -> Result<Subscription> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1",
            self.table("subscription")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("calendar.subscription_not_found"))?;
        Ok(subscription(&row))
    }

    /// Removes a subscription with its events.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn unsubscribe(&self, actor: &Actor, id: SubscriptionId) -> Result<()> {
        let found = self.stored_subscription(id).await?;
        self.access.require(actor, MANAGE, found.scope).await?;
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("subscription")
        )))
        .bind(id.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    }

    /// Reads a subscription again now, for whoever manages its calendar.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, and the reason the read failed, which is also recorded.
    pub async fn refresh_now(&self, actor: &Actor, id: SubscriptionId) -> Result<Refreshed> {
        let found = self.stored_subscription(id).await?;
        self.access.require(actor, MANAGE, found.scope).await?;
        self.refresh(&found).await
    }

    /// Reads the subscriptions whose time came, at most a batch per run; returns how many it read. A failed
    /// read is recorded on its subscription and does not stop the others.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn refresh_due(&self) -> Result<u64> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where last_refreshed_at is null
                 or last_refreshed_at < now() - make_interval(mins => refresh_minutes)
             order by last_refreshed_at nulls first limit $1",
            self.table("subscription")
        )))
        .bind(BATCH)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        for row in &rows {
            self.refresh(&subscription(row)).await.ok();
        }
        Ok(u64::try_from(rows.len()).unwrap_or(0))
    }

    /// Reads one subscription and applies what changed; records the moment, or the reason it failed.
    async fn refresh(&self, found: &Subscription) -> Result<Refreshed> {
        let outcome = match self.fetch(&found.url).await {
            Ok(text) => self.apply(found, &ical::read(&text)).await,
            Err(error) => Err(error),
        };
        sqlx::query(sql(format!(
            "update {} set last_refreshed_at = now(), last_error = $2 where id = $1",
            self.table("subscription")
        )))
        .bind(found.id.as_uuid())
        .bind(outcome.as_ref().err().map(|e| e.code().to_owned()))
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        outcome
    }

    /// Reads a calendar from its address: public addresses only, no redirects, at most a limit of bytes.
    async fn fetch(&self, url: &str) -> Result<String> {
        let failed = |code: &str| Error::unavailable(format!("calendar.{code}"));
        let parsed = reqwest::Url::parse(url).map_err(|_| failed("subscription_unreadable"))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| failed("subscription_unreadable"))?
            .to_owned();
        let port = parsed.port_or_known_default().unwrap_or(443);
        let addresses: Vec<std::net::SocketAddr> = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|_| failed("subscription_unreachable"))?
            .collect();
        let Some(address) = addresses.first().copied() else {
            return Err(failed("subscription_unreachable"));
        };
        if !self.private_addresses && addresses.iter().any(|a| !public(a.ip())) {
            return Err(Error::validation(
                "calendar.subscription_address_refused",
                vec![FieldError::new("url", "public")],
            ));
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(TIMEOUT)
            .resolve(&host, address)
            .build()
            .map_err(|_| failed("subscription_unreadable"))?;
        let mut response = client
            .get(parsed)
            .send()
            .await
            .map_err(|_| failed("subscription_unreachable"))?;
        if !response.status().is_success() {
            return Err(failed("subscription_unreadable")
                .with_param("status", response.status().as_u16().to_string()));
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| failed("subscription_unreadable"))?
        {
            body.extend_from_slice(&chunk);
            if body.len() > MAX_CALENDAR {
                return Err(failed("subscription_too_large"));
            }
        }
        String::from_utf8(body).map_err(|_| failed("subscription_unreadable"))
    }

    /// Applies the events read to the stored ones, matched by UID, and publishes what changed.
    async fn apply(&self, found: &Subscription, read: &[Event]) -> Result<Refreshed> {
        let mut tx = self.db.begin().await?;
        let rows = sqlx::query(sql(format!(
            "select uid, sequence, title, description, starts_at, ends_at, all_day, recurrence from {}
             where subscription_id = $1",
            self.table("imported_entry")
        )))
        .bind(found.id.as_uuid())
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let stored: BTreeMap<String, Event> = rows
            .iter()
            .map(|r| {
                let at = |c: &str| super::entry::to_jiff(Timestamp::from_offset_date_time(r.get(c)));
                (
                    r.get::<String, _>("uid"),
                    Event {
                        uid: r.get("uid"),
                        sequence: r.get("sequence"),
                        title: r.get("title"),
                        description: r.get("description"),
                        starts_at: at("starts_at"),
                        ends_at: at("ends_at"),
                        all_day: r.get("all_day"),
                        recurrence: r.get("recurrence"),
                    },
                )
            })
            .collect();
        let mut done = Refreshed::default();
        let mut kept = Vec::new();
        for event in read {
            kept.push(event.uid.clone());
            match stored.get(&event.uid) {
                Some(old) if old == event => continue,
                Some(_) => done.changed += 1,
                None => done.added += 1,
            }
            sqlx::query(sql(format!(
                "insert into {} (subscription_id, uid, sequence, title, description, starts_at, ends_at, all_day,
                 recurrence) values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 on conflict (subscription_id, uid) do update set sequence = excluded.sequence,
                     title = excluded.title, description = excluded.description, starts_at = excluded.starts_at,
                     ends_at = excluded.ends_at, all_day = excluded.all_day, recurrence = excluded.recurrence",
                self.table("imported_entry")
            )))
            .bind(found.id.as_uuid())
            .bind(&event.uid)
            .bind(event.sequence)
            .bind(&event.title)
            .bind(&event.description)
            .bind(from_jiff(event.starts_at).as_offset_date_time())
            .bind(from_jiff(event.ends_at).as_offset_date_time())
            .bind(event.all_day)
            .bind(&event.recurrence)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        done.removed = sqlx::query(sql(format!(
            "delete from {} where subscription_id = $1 and uid <> all($2)",
            self.table("imported_entry")
        )))
        .bind(found.id.as_uuid())
        .bind(&kept)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        let mut event = kampus_events::Publish::new(
            events::SUBSCRIPTION_REFRESHED,
            Some(found.id.as_uuid()),
            json!({ "subscription": found.id, "added": done.added, "changed": done.changed, "removed": done.removed }),
        );
        event.scope = Some(found.scope.as_uuid());
        event.summary = Some(format!("{} {}", events::SUBSCRIPTION_REFRESHED, found.id));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(done)
    }
}
