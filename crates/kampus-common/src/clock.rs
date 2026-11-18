// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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

//! Instants and the clock (DEC-022): instants are UTC with microsecond precision, exchanged as RFC 3339,
//! and "now" always comes from a clock that tests and jobs can fix.

use std::fmt;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// An instant in UTC, with the microsecond precision the database keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(#[serde(with = "time::serde::rfc3339")] OffsetDateTime);

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

/// The clock of the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_offset_date_time(OffsetDateTime::now_utc())
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

#[cfg(test)]
mod tests {
    use super::{Clock, FixedClock, Timestamp};

    #[test]
    fn instants_are_utc_with_microseconds() {
        let instant = Timestamp::parse_rfc3339("2026-11-18T11:00:00.123456789+01:00").unwrap();
        assert_eq!(instant.to_string(), "2026-11-18T10:00:00.123456Z");
    }

    #[test]
    fn instants_round_trip_as_json() {
        let instant = Timestamp::parse_rfc3339("2026-11-18T10:00:00Z").unwrap();
        let json = serde_json::to_string(&instant).unwrap();
        assert_eq!(json, "\"2026-11-18T10:00:00Z\"");
        assert_eq!(serde_json::from_str::<Timestamp>(&json).unwrap(), instant);
    }

    #[test]
    fn invalid_text_is_a_validation_error() {
        assert_eq!(
            Timestamp::parse_rfc3339("18/11/2026").unwrap_err().code(),
            "common.timestamp_invalid"
        );
    }

    #[test]
    fn a_fixed_clock_answers_its_instant_until_moved() {
        let start = Timestamp::parse_rfc3339("2026-11-18T10:00:00Z").unwrap();
        let later = Timestamp::parse_rfc3339("2026-11-18T11:00:00Z").unwrap();
        let clock = FixedClock::new(start);
        assert_eq!(clock.now(), start);
        clock.set(later);
        assert_eq!(clock.now(), later);
    }
}
