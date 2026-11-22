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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Instants and the clock: instants are UTC with microsecond precision, exchanged as RFC 3339,
//! and "now" always comes from a clock that tests and jobs can fix.

use std::fmt;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// An instant in UTC, with the microsecond precision the database keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(#[serde(with = "time::serde::rfc3339")] OffsetDateTime);

impl schemars::JsonSchema for Timestamp {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Timestamp".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "string", "format": "date-time" })
    }
}

impl Timestamp {
    /// Converts any instant to UTC and drops what is finer than a microsecond.
    #[must_use]
    pub fn from_offset_date_time(instant: OffsetDateTime) -> Self {
        let utc = instant.to_offset(time::UtcOffset::UTC);
        let micros = utc.nanosecond() / 1_000 * 1_000;
        Self(utc.replace_nanosecond(micros).unwrap_or(utc))
    }

    /// The instant, for arithmetic and for binding in queries.
    #[must_use]
    pub const fn as_offset_date_time(&self) -> OffsetDateTime {
        self.0
    }

    /// Parses an RFC 3339 instant with its offset.
    ///
    /// # Errors
    ///
    /// A `validation` error when the text is not RFC 3339.
    pub fn parse_rfc3339(text: &str) -> crate::Result<Self> {
        OffsetDateTime::parse(text, &Rfc3339)
            .map(Self::from_offset_date_time)
            .map_err(|error| {
                crate::Error::validation(
                    "common.timestamp_invalid",
                    vec![crate::FieldError::new("timestamp", "rfc3339")],
                )
                .with_source(error)
            })
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.format(&Rfc3339) {
            Ok(text) => f.write_str(&text),
            Err(_) => Err(fmt::Error),
        }
    }
}

/// Where "now" comes from: one clock for a request or a job, so tests and scheduled work can fix it.
pub trait Clock: Send + Sync {
    /// The current instant.
    fn now(&self) -> Timestamp;
}

/// The latest instant the system clock answered in this process, in microseconds since the Unix epoch.
static LATEST_MICROS: AtomicI64 = AtomicI64::new(i64::MIN);

/// The clock of the operating system, that never goes back within a process.
///
/// The wall clock of a host can step back when it is corrected (a virtual machine resyncing with its
/// host, an NTP adjustment). A grant or an enrolment that starts "now" would then not be valid yet for
/// the next check; this clock answers the latest instant it already gave until the wall clock is past
/// it again.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let wall = OffsetDateTime::now_utc();
        let Ok(micros) = i64::try_from(wall.unix_timestamp_nanos() / 1_000) else {
            return Timestamp::from_offset_date_time(wall);
        };
        let latest = LATEST_MICROS.fetch_max(micros, Ordering::AcqRel).max(micros);
        OffsetDateTime::from_unix_timestamp_nanos(i128::from(latest) * 1_000).map_or_else(
            |_| Timestamp::from_offset_date_time(wall),
            Timestamp::from_offset_date_time,
        )
    }
}

/// A clock that answers a set instant until it is moved, for tests.
#[derive(Debug)]
pub struct FixedClock {
    /// The instant the clock answers.
    instant: Mutex<Timestamp>,
}

impl FixedClock {
    /// Creates a clock fixed at an instant.
    #[must_use]
    pub const fn new(instant: Timestamp) -> Self {
        Self {
            instant: Mutex::new(instant),
        }
    }

    /// Moves the clock to another instant.
    pub fn set(&self, instant: Timestamp) {
        let mut current = self
            .instant
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *current = instant;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        *self
            .instant
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
