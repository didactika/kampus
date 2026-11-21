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
//! were read at and deleted.

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, ErrorKind, FieldError, Result, Timestamp};
use serde_json::json;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::person;
use crate::permissions::{MANAGE, VIEW};
use crate::{Calendar, Entry, EntryFields, EntryId, events, sql};

/// The columns of an entry.
pub(crate) const COLUMNS: &str = "id, scope_id, owner_id, title, description, starts_at, ends_at, version";

/// The most characters of a title.
const TITLE_LIMIT: usize = 200;

/// The most characters of a description.
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
        version: row.get("version"),
    }
}

/// The title and description of an entry, checked with its times.
fn checked(fields: &EntryFields) -> Result<(String, Option<String>)> {
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
    if problems.is_empty() {
        Ok((title.to_owned(), description))
    } else {
        Err(Error::validation("calendar.entry_invalid", problems))
    }
}

impl Calendar {
    /// An entry in any case, whoever asks.
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

    /// Whether the actor sees an entry: their own personal entry, or one of a scope where they hold
    /// `calendar.view`.
    async fn can_see(&self, actor: &Actor, found: &Entry) -> Result<bool> {
        match (found.scope, found.owner) {
            (Some(scope), _) => Ok(self.access.check(actor, VIEW, scope).await?.allowed),
            (None, Some(owner)) => Ok(person(actor) == Some(owner)),
            (None, None) => Ok(false),
        }
    }

    /// Fails unless the actor may change an entry: whoever holds `calendar.manage` in its scope, or the
    /// owner of a personal one. Others do not learn that a personal entry exists.
    async fn require_changing(&self, actor: &Actor, found: &Entry) -> Result<()> {
        match (found.scope, found.owner) {
            (Some(scope), _) => self.access.require(actor, MANAGE, scope).await,
            (None, Some(owner)) if person(actor) == Some(owner) => Ok(()),
            _ => Err(Error::not_found("calendar.entry_not_found")),
        }
    }

    /// Publishes an event about an entry in the transaction of the change.
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

    /// Creates an entry in a scope, which needs `calendar.manage` there, or for the actor when no scope is
    /// given.
    ///
    /// # Errors
    ///
    /// `permission_denied`, `validation` (`calendar.entry_invalid`): a title, and an end after the start.
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
        let (title, description) = checked(&fields)?;
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, scope_id, owner_id, author, title, description, starts_at, ends_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8) returning {COLUMNS}",
            self.table("entry")
        )))
        .bind(EntryId::generate().as_uuid())
        .bind(scope.map(|s| s.as_uuid()))
        .bind(owner.map(|o| o.as_uuid()))
        .bind(person(actor).map(|p| p.as_uuid()))
        .bind(&title)
        .bind(description)
        .bind(fields.starts_at.as_offset_date_time())
        .bind(fields.ends_at.as_offset_date_time())
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
    /// `not_found` for an unknown entry or one the actor does not see.
    pub async fn entry(&self, actor: &Actor, id: EntryId) -> Result<Entry> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let found = self.stored(&mut conn, id, false).await?;
        if self.can_see(actor, &found).await? {
            Ok(found)
        } else {
            Err(Error::not_found("calendar.entry_not_found"))
        }
    }

    /// Replaces the title, description and times of an entry, on the version it was read at.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation`, and `precondition_failed`
    /// (`calendar.entry_changed`) when it changed since it was read.
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
        let (title, description) = checked(&fields)?;
        let row = sqlx::query(sql(format!(
            "update {} set title = $2, description = $3, starts_at = $4, ends_at = $5, version = version + 1
             where id = $1 returning {COLUMNS}",
            self.table("entry")
        )))
        .bind(id.as_uuid())
        .bind(&title)
        .bind(description)
        .bind(fields.starts_at.as_offset_date_time())
        .bind(fields.ends_at.as_offset_date_time())
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let updated = entry(&row);
        self.publish_in(&mut tx, events::ENTRY_UPDATED, &updated, actor)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(updated)
    }

    /// Deletes an entry.
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
}
