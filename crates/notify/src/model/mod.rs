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

//! The entities of notify: a notification in the inbox of a person, and the choice a person has made about a
//! type of notification.

use kampus_access::PrincipalId;
use kampus_common::Timestamp;
use serde::Serialize;
use serde_json::Value;

kampus_common::define_id! {
    /// Identifies a notification.
    pub struct NotificationId;
}

/// One notification to one person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notification {
    /// The notification.
    pub id: NotificationId,
    /// Its type.
    #[serde(rename = "type")]
    pub type_code: String,
    /// Who it is for.
    pub recipient: PrincipalId,
    /// What it is about: the objects it links to, which are checked against the permissions of the person
    /// when they open it.
    pub payload: Value,
    /// When it was created.
    pub created_at: Timestamp,
    /// When the person read it.
    pub read_at: Option<Timestamp>,
}

/// What a person receives of a type of notification through a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Preference {
    /// The type.
    #[serde(rename = "type")]
    pub type_code: String,
    /// What it tells people.
    pub summary: String,
    /// The channel.
    pub channel: String,
    /// Whether the person receives it.
    pub enabled: bool,
    /// Whether it cannot be turned off.
    pub required: bool,
    /// Whether the campus fixed it, so the person cannot change it.
    pub locked: bool,
}

/// A page of the inbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inbox {
    /// The notifications, newest first.
    pub notifications: Vec<Notification>,
    /// How many of all of them the person has not read.
    pub unread: i64,
    /// Where the next page starts, when there is one.
    pub next: Option<NotificationId>,
}
