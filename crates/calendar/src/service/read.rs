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

//! What a calendar shows in a window of time: the entries the actor sees and, for a course, the dates of its
//! activities as they apply to the person who asks.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::CourseId;

use jiff::tz::TimeZone;
use sqlx::Row;

use super::entry::{COLUMNS, entry, from_jiff, to_jiff};
use super::person;
use crate::permissions::VIEW;
use crate::support::recurrence::Rule;
use crate::{Calendar, Item, ItemKind, sql};

/// The longest window a calendar answers for.
const LONGEST_WINDOW: time::Duration = time::Duration::days(366);

/// Fails unless the window is a real one that is not too long.
fn require_window(from: Timestamp, to: Timestamp) -> Result<()> {
    if to <= from || to.as_offset_date_time() - from.as_offset_date_time() > LONGEST_WINDOW {
        return Err(Error::validation(
            "calendar.window_invalid",
            vec![kampus_common::FieldError::new("to", "window")],
        ));
    }
    Ok(())
}

/// An item for an entry.
fn item(found: crate::Entry) -> Item {
    Item {
        kind: ItemKind::Entry,
        title: found.title,
        starts_at: found.starts_at,
        ends_at: Some(found.ends_at),
        all_day: found.all_day,
        occurrence: None,
        entry: Some(found.id),
        course_module: None,
    }
}

impl Calendar {
    /// The entries of a scope that touch a window, or the personal entries of the actor when no scope is
    /// given.
    async fn entries_in(
        &self,
        actor: &Actor,
        scope: Option<ScopeId>,
        owner: Option<kampus_access::PrincipalId>,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Item>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where ((scope_id = $1) or (owner_id = $2))
             and starts_at < $4 and (ends_at > $3 or recurrence is not null) order by starts_at, id",
            self.table("entry")
        )))
        .bind(scope.map(|s| s.as_uuid()))
        .bind(owner.map(|o| o.as_uuid()))
        .bind(from.as_offset_date_time())
        .bind(to.as_offset_date_time())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut items = Vec::new();
        for found in rows.iter().map(entry) {
            if found.group.is_some() && !self.can_see(actor, &found).await? {
                continue;
            }
            if found.recurrence.is_some() {
                items.extend(self.occurrences_of(&found, from, to).await?);
            } else {
                items.push(item(found));
            }
        }
        if let Some(scope) = scope {
            items.extend(self.imported_in(scope, from, to).await?);
        }
        items.sort_by_key(|i| i.starts_at);
        Ok(items)
    }

    /// The events of the calendars a scope subscribes to, in a window; recurring ones are expanded in UTC.
    async fn imported_in(&self, scope: ScopeId, from: Timestamp, to: Timestamp) -> Result<Vec<Item>> {
        let rows = sqlx::query(sql(format!(
            "select e.title, e.starts_at, e.ends_at, e.all_day, e.recurrence from {} e
             join {} s on s.id = e.subscription_id
             where s.scope_id = $1 and e.starts_at < $3 and (e.ends_at > $2 or e.recurrence is not null)
             order by e.starts_at",
            self.table("imported_entry"),
            self.table("subscription")
        )))
        .bind(scope.as_uuid())
        .bind(from.as_offset_date_time())
        .bind(to.as_offset_date_time())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut items = Vec::new();
        for row in &rows {
            let starts = Timestamp::from_offset_date_time(row.get("starts_at"));
            let ends = Timestamp::from_offset_date_time(row.get("ends_at"));
            let length = ends.as_offset_date_time() - starts.as_offset_date_time();
            let rule = row
                .get::<Option<String>, _>("recurrence")
                .and_then(|r| Rule::parse(&r).ok());
            let starts_list = match rule {
                Some(rule) => rule
                    .occurrences(
                        to_jiff(starts),
                        &TimeZone::UTC,
                        to_jiff(Timestamp::from_offset_date_time(
                            from.as_offset_date_time() - length,
                        )),
                        to_jiff(to),
                    )
                    .into_iter()
                    .map(from_jiff)
                    .collect(),
                None => vec![starts],
            };
            for at in starts_list {
                let until = Timestamp::from_offset_date_time(at.as_offset_date_time() + length);
                if until <= from || at >= to {
                    continue;
                }
                items.push(Item {
                    kind: ItemKind::Imported,
                    title: row.get("title"),
                    starts_at: at,
                    ends_at: Some(until),
                    all_day: row.get("all_day"),
                    occurrence: None,
                    entry: None,
                    course_module: None,
                });
            }
        }
        Ok(items)
    }

    /// The occurrences of a recurring entry in a window, with the changes of single occurrences applied.
    async fn occurrences_of(
        &self,
        found: &crate::Entry,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Item>> {
        let (Some(rule), Ok(zone)) = (
            found.recurrence.as_deref().and_then(|r| Rule::parse(r).ok()),
            TimeZone::get(&found.time_zone),
        ) else {
            return Ok(Vec::new());
        };
        let length = found.ends_at.as_offset_date_time() - found.starts_at.as_offset_date_time();
        let rows = sqlx::query(sql(format!(
            "select occurrence_at, cancelled, title, starts_at, ends_at from {} where entry_id = $1",
            self.table("entry_exception")
        )))
        .bind(found.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let changes: std::collections::BTreeMap<time::OffsetDateTime, &sqlx::postgres::PgRow> =
            rows.iter().map(|r| (r.get("occurrence_at"), r)).collect();
        let starts_from = to_jiff(Timestamp::from_offset_date_time(
            from.as_offset_date_time() - length,
        ));
        let mut items = Vec::new();
        for at in rule.occurrences(to_jiff(found.starts_at), &zone, starts_from, to_jiff(to)) {
            let occurrence = from_jiff(at);
            let mut starts_at = occurrence;
            let mut ends_at = Timestamp::from_offset_date_time(occurrence.as_offset_date_time() + length);
            let mut title = found.title.clone();
            if let Some(change) = changes.get(&occurrence.as_offset_date_time()) {
                if change.get::<bool, _>("cancelled") {
                    continue;
                }
                let moment = |c: &str| {
                    change
                        .get::<Option<time::OffsetDateTime>, _>(c)
                        .map(Timestamp::from_offset_date_time)
                };
                starts_at = moment("starts_at").unwrap_or(starts_at);
                ends_at = moment("ends_at").unwrap_or(ends_at);
                if let Some(changed) = change.get::<Option<String>, _>("title") {
                    title = changed;
                }
            }
            if ends_at <= from || starts_at >= to {
                continue;
            }
            items.push(Item {
                kind: ItemKind::Entry,
                title,
                starts_at,
                ends_at: Some(ends_at),
                all_day: found.all_day,
                occurrence: Some(occurrence),
                entry: Some(found.id),
                course_module: None,
            });
        }
        Ok(items)
    }

    /// The entries of a scope the actor sees that touch a window, or their personal entries when no scope is
    /// given.
    ///
    /// # Errors
    ///
    /// `validation` (`calendar.window_invalid`) for a window that is empty or longer than a year,
    /// `permission_denied` without `calendar.view` in the scope.
    pub async fn entries_for(
        &self,
        actor: &Actor,
        scope: Option<ScopeId>,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Item>> {
        require_window(from, to)?;
        if let Some(scope) = scope {
            self.access.require(actor, VIEW, scope).await?;
            self.entries_in(actor, Some(scope), None, from, to).await
        } else {
            let owner = person(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
            self.entries_in(actor, None, Some(owner), from, to).await
        }
    }

    /// The calendar of a course in a window: its entries, and when each course module the actor sees opens,
    /// is due, closes and stops accepting late work, with the exceptions of the actor applied.
    ///
    /// # Errors
    ///
    /// `not_found` for a course the actor does not see, `validation` for the window, `permission_denied`
    /// without `calendar.view` in the course.
    pub async fn course_calendar(
        &self,
        actor: &Actor,
        course: CourseId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Item>> {
        require_window(from, to)?;
        let found = self.courses.course(course).await?;
        if !self.courses.sees(actor, found.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        self.access.require(actor, VIEW, found.scope).await?;
        let mut items = self.entries_in(actor, Some(found.scope), None, from, to).await?;
        if let Some(person) = person(actor) {
            for section in self.courses.sections(actor, course).await? {
                for module in self.activities.modules(actor, section.id).await? {
                    let values = self.activities.effective_values(module.id, person).await?;
                    for (kind, at) in [
                        (ItemKind::Opens, values.opens_at),
                        (ItemKind::Due, values.due_at),
                        (ItemKind::Closes, values.closes_at),
                        (ItemKind::Cutoff, values.cutoff_at),
                    ] {
                        if let Some(at) = at.filter(|at| *at >= from && *at < to) {
                            items.push(Item {
                                kind,
                                title: module.title.clone(),
                                starts_at: at,
                                ends_at: None,
                                all_day: false,
                                occurrence: None,
                                entry: None,
                                course_module: Some(module.id),
                            });
                        }
                    }
                }
            }
        }
        items.sort_by_key(|i| i.starts_at);
        Ok(items)
    }
}
