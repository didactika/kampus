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
// @package   kampus-tasks
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Cron rules: minute, hour, day of month, month and day of week, each `*`, a value, a range, a list or
//! a step (`*/15`, `1-5`, `0,30`), read in a time zone. When both days are restricted, either matching is
//! enough, as cron does.

use jiff::civil::{DateTime, date};
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp};
use kampus_common::{Error, FieldError, Result};

/// The most minutes a search for the next run looks ahead: a little over four years, for 29 February.
const HORIZON_DAYS: i32 = 4 * 366 + 1;

/// The allowed values of one field.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    /// Which values match, by value.
    allowed: Vec<bool>,
    /// Whether the field was `*`, which matters for the two day fields.
    any: bool,
}

impl Field {
    /// Whether a value matches.
    fn matches(&self, value: i32) -> bool {
        usize::try_from(value)
            .ok()
            .and_then(|v| self.allowed.get(v))
            .copied()
            .unwrap_or(false)
    }
}

/// A parsed rule in its time zone.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Minutes, 0 to 59.
    minute: Field,
    /// Hours, 0 to 23.
    hour: Field,
    /// Days of the month, 1 to 31.
    day: Field,
    /// Months, 1 to 12.
    month: Field,
    /// Days of the week, 0 (Sunday) to 6; 7 is Sunday too.
    weekday: Field,
    /// The time zone the rule is read in.
    zone: TimeZone,
}

/// The error of a rule that does not parse.
fn invalid(field: &'static str) -> Error {
    Error::validation("tasks.cron_invalid", vec![FieldError::new(field, "cron")])
}

/// Parses one field between `low` and `high`.
fn field(text: &str, name: &'static str, low: i32, high: i32) -> Result<Field> {
    let size = usize::try_from(high + 1).map_err(|_| invalid(name))?;
    let mut allowed = vec![false; size];
    let number = |t: &str| t.parse::<i32>().map_err(|_| invalid(name));
    for part in text.split(',') {
        let (range, step) = match part.split_once('/') {
            Some((range, step)) => (range, number(step)?),
            None => (part, 1),
        };
        let (from, to) = if range == "*" {
            (low, high)
        } else if let Some((a, b)) = range.split_once('-') {
            (number(a)?, number(b)?)
        } else {
            let value = number(range)?;
            (value, if part.contains('/') { high } else { value })
        };
        if step < 1 || from < low || to > high || from > to {
            return Err(invalid(name));
        }
        let mut value = from;
        while value <= to {
            if let Some(slot) = usize::try_from(value).ok().and_then(|v| allowed.get_mut(v)) {
                *slot = true;
            }
            value += step;
        }
    }
    Ok(Field {
        allowed,
        any: text == "*",
    })
}

impl Rule {
    /// Parses a rule read in a time zone.
    ///
    /// # Errors
    ///
    /// `validation` (`tasks.cron_invalid`) naming the field that does not parse, or `time_zone` for an
    /// unknown zone.
    pub fn parse(text: &str, time_zone: &str) -> Result<Self> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        let [minute, hour, day, month, weekday] = parts.as_slice() else {
            return Err(invalid("cron"));
        };
        let mut weekday = field(weekday, "weekday", 0, 7)?;
        if weekday.matches(7) {
            weekday.allowed[0] = true;
        }
        Ok(Self {
            minute: field(minute, "minute", 0, 59)?,
            hour: field(hour, "hour", 0, 23)?,
            day: field(day, "day", 1, 31)?,
            month: field(month, "month", 1, 12)?,
            weekday,
            zone: TimeZone::get(time_zone).map_err(|_| {
                Error::validation(
                    "tasks.time_zone_invalid",
                    vec![FieldError::new("time_zone", "iana")],
                )
            })?,
        })
    }

    /// Whether the rule matches a day, by the cron rule for the two day fields.
    fn day_matches(&self, at: DateTime) -> bool {
        let day = self.day.matches(i32::from(at.day()));
        let weekday = self
            .weekday
            .matches(i32::from(at.weekday().to_sunday_zero_offset()));
        match (self.day.any, self.weekday.any) {
            (true, true) => true,
            (true, false) => weekday,
            (false, true) => day,
            (false, false) => day || weekday,
        }
    }

    /// The first instant after `after` the rule matches, to the minute.
    ///
    /// # Errors
    ///
    /// `validation` (`tasks.cron_never`) for a rule that never matches, such as 31 February.
    pub fn next_after(&self, after: Timestamp) -> Result<Timestamp> {
        let never = || Error::validation("tasks.cron_never", vec![FieldError::new("cron", "matches")]);
        let start = after.to_zoned_datetime(self.zone.clone()).datetime();
        let mut at = start
            .with()
            .second(0)
            .subsec_nanosecond(0)
            .build()
            .map_err(|_| never())?
            .checked_add(Span::new().minutes(1))
            .map_err(|_| never())?;
        let limit = start
            .checked_add(Span::new().days(HORIZON_DAYS))
            .map_err(|_| never())?;
        while at <= limit {
            if !self.month.matches(i32::from(at.month())) {
                let (year, month) = if at.month() == 12 {
                    (at.year() + 1, 1)
                } else {
                    (at.year(), at.month() + 1)
                };
                at = date(year, month, 1).at(0, 0, 0, 0);
            } else if !self.day_matches(at) {
                at = at.date().tomorrow().map_err(|_| never())?.at(0, 0, 0, 0);
            } else if !self.hour.matches(i32::from(at.hour())) {
                at = at
                    .with()
                    .minute(0)
                    .build()
                    .map_err(|_| never())?
                    .checked_add(Span::new().hours(1))
                    .map_err(|_| never())?;
            } else if !self.minute.matches(i32::from(at.minute())) {
                at = at.checked_add(Span::new().minutes(1)).map_err(|_| never())?;
            } else {
                let zoned = at.to_zoned_datetime(self.zone.clone()).map_err(|_| never())?;
                if zoned.timestamp() > after {
                    return Ok(zoned.timestamp());
                }
                at = at.checked_add(Span::new().minutes(1)).map_err(|_| never())?;
            }
        }
        Err(never())
    }
}
