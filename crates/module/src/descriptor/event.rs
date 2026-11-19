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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events a module declares, with the fields of their payload, and the listeners it runs when an
//! event of any module happens.

use std::fmt;

use kampus_common::Timestamp;
use serde_json::Value;

use crate::{JobFuture, ServiceContext};

/// A kind of event a module publishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventDeclaration {
    /// Its code, `<module>.<what_happened>`.
    pub code: &'static str,
    /// The version of its payload; a change that breaks listeners is a new version.
    pub version: u16,
    /// What it means, for developers and operators.
    pub summary: &'static str,
    /// The fields every payload has.
    pub fields: &'static [&'static str],
    /// Whether each occurrence writes an entry in the audit log.
    pub audited: bool,
    /// The fields that hold personal data, for the privacy handlers.
    pub personal_fields: &'static [&'static str],
}

/// One event, as a listener receives it.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// The event; a listener that receives it twice does nothing the second time.
    pub id: uuid::Uuid,
    /// Its type.
    pub code: String,
    /// The version of its payload.
    pub version: u16,
    /// The scope it happened in.
    pub scope: Option<uuid::Uuid>,
    /// Who made it happen, when someone did.
    pub actor: Option<uuid::Uuid>,
    /// What it is about; events about one object reach a listener in order.
    pub object: Option<uuid::Uuid>,
    /// What happened.
    pub payload: Value,
    /// When it happened.
    pub occurred_at: Timestamp,
    /// How many events led to it, through listeners that published.
    pub depth: u8,
}

/// One delivery of an event to a listener.
#[derive(Debug, Clone, Copy)]
pub struct EventRun<'e> {
    /// The event.
    pub event: &'e Event,
    /// The services of the process.
    pub services: &'e ServiceContext,
}

/// A reaction of a module to an event.
#[derive(Clone, Copy)]
pub struct ListenerDeclaration {
    /// The event it listens to.
    pub event: &'static str,
    /// Its name, unique in its module.
    pub name: &'static str,
    /// Runs it; it may run again for the same event, so it does nothing twice.
    pub run: for<'e> fn(EventRun<'e>) -> JobFuture<'e>,
}

impl fmt::Debug for ListenerDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListenerDeclaration")
            .field("event", &self.event)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}
