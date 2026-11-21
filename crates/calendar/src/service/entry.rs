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

//! Entries: created in a scope by whoever manages its calendar, or for oneself; changed on the version they
//! were read at and deleted. An entry can repeat by a rule, last whole days, follow a time zone and be for a
//! group of the course; one occurrence of a repeating entry can be cancelled or moved without copying it.

use jiff::tz::TimeZone;
use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, ErrorKind, FieldError, Result, Timestamp};
use kampus_courses::GroupId;
use serde_json::json;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::person;
use crate::permissions::{MANAGE, VIEW};
use crate::support::recurrence::Rule;
use crate::{Calendar, Entry, EntryFields, EntryId, OccurrenceChange, events, sql};

/// The columns of an entry.
pub(crate) const COLUMNS: &str = "id, scope_id, owner_id, title, description, starts_at, ends_at, time_zone, all_day, \
                                  recurrence, group_id, version";

/// The longest title, in characters.
const TITLE_LIMIT: usize = 200;

/// The longest description, in characters.
const DESCRIPTION_LIMIT: usize = 5000;

/// An entry from its row.
pub(crate) fn entry(row: &PgRow) -> Entry {
    Entry {
        id: EntryId::from_uuid(row.get("id")),
        scope: row
            .get::<Option<uuid::Uuid>, _>("scope_id")
            .map(ScopeId::from_uuid),
        owner: row
            .get::<Option<uuid::Uuid>, _>("owner_id")
            .map(PrincipalId::from_uuid),
        title: row.get("title"),
        description: row.get("description"),
        starts_at: Timestamp::from_offset_date_time(row.get("starts_at")),
        ends_at: Timestamp::from_offset_date_time(row.get("ends_at")),
        time_zone: row.get("time_zone"),
        all_day: row.get("all_day"),
        recurrence: row.get("recurrence"),
        group: row
            .get::<Option<uuid::Uuid>, _>("group_id")
            .map(GroupId::from_uuid),
        version: row.get("version"),
    }
}

/// A moment in the representation of jiff.
pub(crate) fn to_jiff(at: Timestamp) -> jiff::Timestamp {
    jiff::Timestamp::from_second(at.as_offset_date_time().unix_timestamp())
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
}

/// A moment from the representation of jiff.
pub(crate) fn from_jiff(at: jiff::Timestamp) -> Timestamp {
    Timestamp::from_offset_date_time(
        time::OffsetDateTime::from_unix_timestamp(at.as_second()).unwrap_or(time::OffsetDateTime::UNIX_EPOCH),
    )
}

/// What a person set, checked.
struct Checked {
    /// The title, trimmed.
    title: String,
    /// The description, trimmed, when there is one.
    description: Option<String>,
    /// The name of the time zone.
    zone: String,
    /// The rule, as given.
    recurrence: Option<String>,
}

/// Checks what a person set on an entry.
fn checked(fields: &EntryFields) -> Result<Checked> {
    let mut problems = Vec::new();
    let title = fields.title.trim();
    if title.is_empty() || title.chars().count() > TITLE_LIMIT {
        problems.push(FieldError::new("title", "length"));
    }
    let description = fields
        .description
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(str::to_owned);
    if description
        .as_ref()
        .is_some_and(|d| d.chars().count() > DESCRIPTION_LIMIT)
    {
        problems.push(FieldError::new("description", "length"));
    }
    if fields.ends_at <= fields.starts_at {
        problems.push(FieldError::new("ends_at", "after_starts_at"));
    }
    let zone_name = fields.time_zone.as_deref().unwrap_or("UTC").trim().to_owned();
    let zone = TimeZone::get(&zone_name).ok();
    if zone.is_none() {
        problems.push(FieldError::new("time_zone", "time_zone"));
    }
    if let (true, Some(zone)) = (fields.all_day, &zone) {
        let midnight = |at: Timestamp| {
            let local = to_jiff(at).to_zoned_datetime(zone.clone());
            local.hour() == 0 && local.minute() == 0 && local.second() == 0
        };
        if !midnight(fields.starts_at) || !midnight(fields.ends_at) {
            problems.push(FieldError::new("all_day", "midnight"));
        }
    }
    let recurrence = fields
        .recurrence
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(str::to_owned);
    if let Some(rule) = &recurrence
        && Rule::parse(rule).is_err()
    {
        problems.push(FieldError::new("recurrence", "rule"));
    }
    if problems.is_empty() {
        Ok(Checked {
            title: title.to_owned(),
            description,
            zone: zone_name,
            recurrence,
        })
    } else {
        Err(Error::validation("calendar.entry_invalid", problems))
    }
}

impl Calendar {
    /// An entry, whoever asks.
    async fn stored(&self, conn: &mut PgConnection, id: EntryId, lock: bool) -> Result<Entry> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1 {}",
            self.table("entry"),
            if lock { "for update" } else { "" }
        )))
        .bind(id.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("calendar.entry_not_found"))?;
        Ok(entry(&row))
    }

    /// Whether an actor sees an entry: in a scope with `calendar.view` there, and for a group as one of its
    /// members or with `calendar.manage`; a personal one only its owner.
    pub(crate) async fn can_see(&self, actor: &Actor, found: &Entry) -> Result<bool> {
        match (found.scope, found.owner) {
            (Some(scope), _) => {
                if !self.access.check(actor, VIEW, scope).await?.allowed {
                    return Ok(false);
                }
                let Some(group) = found.group else {
                    return Ok(true);
                };
                if self.access.check(actor, MANAGE, scope).await?.allowed {
                    return Ok(true);
                }
                let Some(account) = person(actor) else {
                    return Ok(false);
                };
                let course = self.courses.group(group).await?.course;
                Ok(self.courses.groups_of(course, account).await?.contains(&group))
            }
            (None, Some(owner)) => Ok(person(actor) == Some(owner)),
            (None, None) => Ok(false),
        }
    }

    /// Fails unless the actor may change an entry.
    async fn require_changing(&self, actor: &Actor, found: &Entry) -> Result<()> {
        match (found.scope, found.owner) {
            (Some(scope), _) => self.access.require(actor, MANAGE, scope).await,
            (None, Some(owner)) if person(actor) == Some(owner) => Ok(()),
            _ => Err(Error::not_found("calendar.entry_not_found")),
        }
    }

    /// Checks that a group is of the course the entry is in.
    async fn check_group(&self, scope: Option<ScopeId>, group: Option<GroupId>) -> Result<()> {
        let Some(group) = group else { return Ok(()) };
        let wrong = || {
            Error::validation(
                "calendar.entry_invalid",
                vec![FieldError::new("group", "same_course")],
            )
        };
        let Some(scope) = scope else { return Err(wrong()) };
        let found = self.courses.group(group).await.map_err(|_| wrong())?;
        if self.courses.course(found.course).await?.scope == scope {
            Ok(())
        } else {
            Err(wrong())
        }
    }

    /// Publishes an event about an entry, in the transaction of the change.
    async fn publish_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        found: &Entry,
        actor: &Actor,
    ) -> Result<()> {
        let mut event =
            kampus_events::Publish::new(code, Some(found.id.as_uuid()), json!({ "entry": found.id }));
        event.scope = found.scope.map(|s| s.as_uuid());
        event.actor = person(actor).map(|p| p.as_uuid());
        event.summary = Some(format!("{code} {}", found.id));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }

    /// Adds an entry to a scope, with `calendar.manage` there, or a personal one when no scope is given.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission or for a visitor, `validation` (`calendar.entry_invalid`)
    /// for a title, a description, dates, a time zone, a rule or a group that do not fit.
    pub async fn create_entry(
        &self,
        actor: &Actor,
        scope: Option<ScopeId>,
        fields: EntryFields,
    ) -> Result<Entry> {
        let owner = match scope {
            Some(scope) => {
                self.access.require(actor, MANAGE, scope).await?;
                None
            }
            None => Some(person(actor).ok_or_else(|| Error::permission_denied("access.denied"))?),
        };
        let ok = checked(&fields)?;
        self.check_group(scope, fields.group).await?;
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, scope_id, owner_id, author, title, description, starts_at, ends_at, time_zone,
             all_day, recurrence, group_id)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) returning {COLUMNS}",
            self.table("entry")
        )))
        .bind(EntryId::generate().as_uuid())
        .bind(scope.map(|s| s.as_uuid()))
        .bind(owner.map(|o| o.as_uuid()))
        .bind(person(actor).map(|p| p.as_uuid()))
        .bind(&ok.title)
        .bind(ok.description)
        .bind(fields.starts_at.as_offset_date_time())
        .bind(fields.ends_at.as_offset_date_time())
        .bind(&ok.zone)
        .bind(fields.all_day)
        .bind(ok.recurrence)
        .bind(fields.group.map(|g| g.as_uuid()))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let created = entry(&row);
        self.publish_in(&mut tx, events::ENTRY_CREATED, &created, actor)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(created)
    }

    /// An entry the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` (`calendar.entry_not_found`) for an unknown entry or one the actor does not see.
    pub async fn entry(&self, actor: &Actor, id: EntryId) -> Result<Entry> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let found = self.stored(&mut conn, id, false).await?;
        drop(conn);
        if self.can_see(actor, &found).await? {
            Ok(found)
        } else {
            Err(Error::not_found("calendar.entry_not_found"))
        }
    }

    /// Replaces an entry, on the version it was read at.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation`, `precondition_failed` (`calendar.entry_changed`).
    pub async fn update_entry(
        &self,
        actor: &Actor,
        id: EntryId,
        fields: EntryFields,
        expected_version: Option<i32>,
    ) -> Result<Entry> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored(&mut tx, id, true).await?;
        self.require_changing(actor, &stored).await?;
        if expected_version.is_some_and(|v| v != stored.version) {
            return Err(
                Error::new(ErrorKind::PreconditionFailed, "calendar.entry_changed")
                    .with_param("version", stored.version.to_string()),
            );
        }
        let ok = checked(&fields)?;
        self.check_group(stored.scope, fields.group).await?;
        let row = sqlx::query(sql(format!(
            "update {} set title = $2, description = $3, starts_at = $4, ends_at = $5, time_zone = $6,
             all_day = $7, recurrence = $8, group_id = $9, version = version + 1 where id = $1 returning {COLUMNS}",
            self.table("entry")
        )))
        .bind(id.as_uuid())
        .bind(&ok.title)
        .bind(ok.description)
        .bind(fields.starts_at.as_offset_date_time())
        .bind(fields.ends_at.as_offset_date_time())
        .bind(&ok.zone)
        .bind(fields.all_day)
        .bind(ok.recurrence)
        .bind(fields.group.map(|g| g.as_uuid()))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let updated = entry(&row);
        self.publish_in(&mut tx, events::ENTRY_UPDATED, &updated, actor)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(updated)
    }

    /// Deletes an entry with its occurrences.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn delete_entry(&self, actor: &Actor, id: EntryId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored(&mut tx, id, true).await?;
        self.require_changing(actor, &stored).await?;
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("entry"))))
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        self.publish_in(&mut tx, events::ENTRY_DELETED, &stored, actor)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Cancels or changes one occurrence of a recurring entry, named by when the rule starts it; changing it
    /// again replaces the change.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`calendar.occurrence_invalid`) for an entry that does
    /// not repeat, a moment that is not one of its occurrences, or an end before the start.
    pub async fn change_occurrence(
        &self,
        actor: &Actor,
        id: EntryId,
        occurrence: Timestamp,
        change: OccurrenceChange,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored(&mut tx, id, true).await?;
        self.require_changing(actor, &stored).await?;
        let wrong = |field: &str| {
            Error::validation(
                "calendar.occurrence_invalid",
                vec![FieldError::new(field, "occurrence")],
            )
        };
        let rule = stored
            .recurrence
            .as_deref()
            .and_then(|r| Rule::parse(r).ok())
            .ok_or_else(|| wrong("entry"))?;
        let zone = TimeZone::get(&stored.time_zone).map_err(|_| wrong("entry"))?;
        let at = to_jiff(occurrence);
        let found = rule.occurrences(
            to_jiff(stored.starts_at),
            &zone,
            at,
            at.checked_add(jiff::Span::new().seconds(1)).unwrap_or(at),
        );
        if found.first() != Some(&at) {
            return Err(wrong("occurrence"));
        }
        if let (Some(starts), Some(ends)) = (change.starts_at, change.ends_at)
            && ends <= starts
        {
            return Err(wrong("ends_at"));
        }
        let title = change.title.as_deref().map(str::trim).filter(|t| !t.is_empty());
        if title.is_some_and(|t| t.chars().count() > TITLE_LIMIT) {
            return Err(wrong("title"));
        }
        sqlx::query(sql(format!(
            "insert into {} (entry_id, occurrence_at, cancelled, title, starts_at, ends_at)
             values ($1, $2, $3, $4, $5, $6)
             on conflict (entry_id, occurrence_at) do update set cancelled = excluded.cancelled,
                 title = excluded.title, starts_at = excluded.starts_at, ends_at = excluded.ends_at",
            self.table("entry_exception")
        )))
        .bind(id.as_uuid())
        .bind(occurrence.as_offset_date_time())
        .bind(change.cancelled)
        .bind(title)
        .bind(change.starts_at.map(|t| t.as_offset_date_time()))
        .bind(change.ends_at.map(|t| t.as_offset_date_time()))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(&mut tx, events::ENTRY_UPDATED, &stored, actor)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Takes back the change of one occurrence, which follows the rule again.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn restore_occurrence(&self, actor: &Actor, id: EntryId, occurrence: Timestamp) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let stored = self.stored(&mut tx, id, true).await?;
        self.require_changing(actor, &stored).await?;
        sqlx::query(sql(format!(
            "delete from {} where entry_id = $1 and occurrence_at = $2",
            self.table("entry_exception")
        )))
        .bind(id.as_uuid())
        .bind(occurrence.as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(&mut tx, events::ENTRY_UPDATED, &stored, actor)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
