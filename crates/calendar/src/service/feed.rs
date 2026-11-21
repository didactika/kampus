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

//! Personal feeds: a person makes a secret address that a calendar application reads, with what they see in
//! their courses and their own entries, from a month ago to almost a year ahead. Only the hash of the secret
//! is kept, it is shown once, and it can be revoked.

use std::collections::BTreeSet;

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::entry::to_jiff;
use super::person;
use crate::permissions::VIEW;
use crate::support::ical::{self, Event};
use crate::{Calendar, Feed, FeedId, events, sql};

/// The longest name of a feed, in characters.
const NAME_LIMIT: usize = 100;

/// How far back a feed goes.
const BACK: time::Duration = time::Duration::days(30);

/// How far ahead a feed goes.
const AHEAD: time::Duration = time::Duration::days(335);

/// The columns of a feed.
const COLUMNS: &str = "id, name, created_at, last_used_at, revoked_at";

/// A feed from its row.
fn feed(row: &PgRow) -> Feed {
    let stamp = |c: &str| {
        row.get::<Option<time::OffsetDateTime>, _>(c)
            .map(Timestamp::from_offset_date_time)
    };
    Feed {
        id: FeedId::from_uuid(row.get("id")),
        name: row.get("name"),
        created_at: Timestamp::from_offset_date_time(row.get("created_at")),
        last_used_at: stamp("last_used_at"),
        revoked_at: stamp("revoked_at"),
    }
}

/// The hash a secret is kept as.
fn hash(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}

impl Calendar {
    /// Publishes an event about a feed.
    async fn publish_feed_in(
        &self,
        conn: &mut sqlx::PgConnection,
        code: &str,
        id: FeedId,
        account: PrincipalId,
    ) -> Result<()> {
        let mut event = kampus_events::Publish::new(
            code,
            Some(id.as_uuid()),
            json!({ "feed": id, "account": account }),
        );
        event.actor = Some(account.as_uuid());
        event.summary = Some(format!("{code} {id}"));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }

    /// Makes a personal feed; the secret it answers with is the only time it is shown.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor, `validation` (`calendar.feed_invalid`) for an empty or long name.
    pub async fn create_feed(&self, actor: &Actor, name: &str) -> Result<(Feed, String)> {
        let owner = person(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > NAME_LIMIT {
            return Err(Error::validation(
                "calendar.feed_invalid",
                vec![FieldError::new("name", "length")],
            ));
        }
        let secret = kampus_identity::support::secret::random_hex::<32>()?;
        let id = FeedId::generate();
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, account_id, hash, name) values ($1, $2, $3, $4) returning {COLUMNS}",
            self.table("feed_token")
        )))
        .bind(id.as_uuid())
        .bind(owner.as_uuid())
        .bind(hash(&secret))
        .bind(name)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_feed_in(&mut tx, events::FEED_CREATED, id, owner)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok((feed(&row), secret))
    }

    /// The feeds of the actor, newest first.
    ///
    /// # Errors
    ///
    /// `permission_denied` for a visitor.
    pub async fn feeds(&self, actor: &Actor) -> Result<Vec<Feed>> {
        let owner = person(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where account_id = $1 order by created_at desc, id desc",
            self.table("feed_token")
        )))
        .bind(owner.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(feed).collect())
    }

    /// Revokes a feed of the actor; revoking it again changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found` (`calendar.feed_not_found`) for a feed of someone else.
    pub async fn revoke_feed(&self, actor: &Actor, id: FeedId) -> Result<()> {
        let owner = person(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        let mut tx = self.db.begin().await?;
        let found: Option<bool> = sqlx::query_scalar(sql(format!(
            "update {} set revoked_at = coalesce(revoked_at, now()) where id = $1 and account_id = $2
             returning revoked_at = now()",
            self.table("feed_token")
        )))
        .bind(id.as_uuid())
        .bind(owner.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        match found {
            None => return Err(Error::not_found("calendar.feed_not_found")),
            Some(true) => {
                self.publish_feed_in(&mut tx, events::FEED_REVOKED, id, owner)
                    .await?;
            }
            Some(false) => {}
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The calendar a feed gives, as iCalendar: what its owner sees now in the scopes they belong to and
    /// their own entries. An unknown or revoked secret is not found.
    ///
    /// # Errors
    ///
    /// `not_found` (`calendar.feed_not_found`).
    pub async fn feed_calendar(&self, secret: &str) -> Result<String> {
        let row = sqlx::query(sql(format!(
            "update {} set last_used_at = now() where hash = $1 and revoked_at is null
             returning account_id, name",
            self.table("feed_token")
        )))
        .bind(hash(secret))
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("calendar.feed_not_found"))?;
        let actor = Actor::Principal(PrincipalId::from_uuid(row.get("account_id")));
        let now = time::OffsetDateTime::now_utc();
        let from = Timestamp::from_offset_date_time(now - BACK);
        let to = Timestamp::from_offset_date_time(now + AHEAD);
        let mut items = self.entries_for(&actor, None, from, to).await?;
        for scope in self.access.allowed_scopes(&actor, VIEW).await? {
            items.extend(self.entries_for(&actor, Some(scope), from, to).await?);
        }
        let mut seen = BTreeSet::new();
        let events: Vec<Event> = items
            .into_iter()
            .filter_map(|item| {
                let id = item.entry.map_or_else(|| item.title.clone(), |e| e.to_string());
                let at = item.occurrence.unwrap_or(item.starts_at);
                let uid = format!("{id}-{}@kampus", to_jiff(at).as_second());
                seen.insert(uid.clone()).then(|| Event {
                    uid,
                    sequence: 0,
                    title: item.title,
                    description: None,
                    starts_at: to_jiff(item.starts_at),
                    ends_at: to_jiff(item.ends_at.unwrap_or(item.starts_at)),
                    all_day: item.all_day,
                    recurrence: None,
                })
            })
            .collect();
        Ok(ical::write(
            &row.get::<String, _>("name"),
            &events,
            to_jiff(Timestamp::from_offset_date_time(now)),
        ))
    }
}
