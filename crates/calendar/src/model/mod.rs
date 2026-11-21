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

//! The entities of calendar: the entries people create, and the items a calendar shows, which also include
//! the dates of activities that calendar never copies.

use kampus_access::{PrincipalId, ScopeId};
use kampus_activities::ModuleId;
use kampus_common::Timestamp;
use serde::{Deserialize, Serialize};

kampus_common::define_id! {
    /// Identifies a calendar entry.
    pub struct EntryId;
}

/// An entry of a scope, or a personal one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// The entry.
    pub id: EntryId,
    /// The scope it belongs to: a campus, a category or a course; none for a personal entry.
    pub scope: Option<ScopeId>,
    /// The person a personal entry belongs to.
    pub owner: Option<PrincipalId>,
    /// Its title.
    pub title: String,
    /// What it is about.
    pub description: Option<String>,
    /// When it starts.
    pub starts_at: Timestamp,
    /// When it ends.
    pub ends_at: Timestamp,
    /// The time zone its recurrence and all-day dates follow.
    pub time_zone: String,
    /// Whether it lasts whole days, from midnight to midnight in its time zone.
    pub all_day: bool,
    /// Its recurrence rule, RFC 5545 (`FREQ=WEEKLY;BYDAY=MO`), for an entry that repeats.
    pub recurrence: Option<String>,
    /// The group of the course it is for; only its members and who manages the calendar see it.
    pub group: Option<kampus_courses::GroupId>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// What a person sets on an entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EntryFields {
    /// Its title.
    pub title: String,
    /// What it is about.
    #[serde(default)]
    pub description: Option<String>,
    /// When it starts.
    pub starts_at: Timestamp,
    /// When it ends.
    pub ends_at: Timestamp,
    /// The time zone its recurrence and all-day dates follow; UTC when left out.
    #[serde(default)]
    pub time_zone: Option<String>,
    /// Whether it lasts whole days.
    #[serde(default)]
    pub all_day: bool,
    /// Its recurrence rule, RFC 5545, for an entry that repeats.
    #[serde(default)]
    pub recurrence: Option<String>,
    /// The group of the course it is for.
    #[serde(default)]
    pub group: Option<kampus_courses::GroupId>,
}

/// What an item of a calendar is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// An entry people created.
    Entry,
    /// When an activity opens.
    Opens,
    /// When an activity is due.
    Due,
    /// When an activity closes.
    Closes,
    /// The last moment a late submission is accepted.
    Cutoff,
    /// An event of a calendar a scope subscribes to.
    Imported,
}

/// One thing a calendar shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    /// What it is.
    pub kind: ItemKind,
    /// Its title: the title of the entry or of the activity.
    pub title: String,
    /// When it starts, or happens.
    pub starts_at: Timestamp,
    /// When it ends, for an entry.
    pub ends_at: Option<Timestamp>,
    /// Whether it lasts whole days.
    pub all_day: bool,
    /// For an occurrence of a recurring entry, when it was due to start by the rule.
    pub occurrence: Option<Timestamp>,
    /// The entry, for an entry.
    pub entry: Option<EntryId>,
    /// The course module, for the date of an activity.
    pub course_module: Option<ModuleId>,
}

/// A change of one occurrence of a recurring entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceChange {
    /// The occurrence is cancelled.
    #[serde(default)]
    pub cancelled: bool,
    /// Another title for it.
    #[serde(default)]
    pub title: Option<String>,
    /// Another start.
    #[serde(default)]
    pub starts_at: Option<Timestamp>,
    /// Another end.
    #[serde(default)]
    pub ends_at: Option<Timestamp>,
}

kampus_common::define_id! {
    /// Identifies a personal feed.
    pub struct FeedId;
}

/// A personal feed: a secret address a calendar application reads what its owner sees from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Feed {
    /// The feed.
    pub id: FeedId,
    /// What its owner calls it.
    pub name: String,
    /// When it was made.
    pub created_at: Timestamp,
    /// When an application last read it.
    pub last_used_at: Option<Timestamp>,
    /// When it was revoked.
    pub revoked_at: Option<Timestamp>,
}

kampus_common::define_id! {
    /// Identifies a subscription to another calendar.
    pub struct SubscriptionId;
}

/// A calendar of another system shown in a scope, read again every so often.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subscription {
    /// The subscription.
    pub id: SubscriptionId,
    /// The scope it is shown in.
    pub scope: ScopeId,
    /// Where it is read from.
    pub url: String,
    /// What it is called.
    pub title: String,
    /// Every how many minutes it is read.
    pub refresh_minutes: i32,
    /// When it was last read.
    pub last_refreshed_at: Option<Timestamp>,
    /// Why the last read failed.
    pub last_error: Option<String>,
}
