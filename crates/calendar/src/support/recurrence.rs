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

//! Recurrence of entries, without the database: the part of the RFC 5545 rule a calendar of courses needs
//! (`FREQ` daily, weekly, monthly or yearly, `INTERVAL`, `COUNT` or `UNTIL`, `BYDAY` for weekly rules and
//! `BYMONTHDAY` for monthly ones), expanded in the time zone of the entry so an occurrence keeps its time
//! of day across daylight saving changes.

use jiff::Timestamp;
use jiff::civil::{Date, Weekday};
use jiff::tz::TimeZone;

/// The most occurrences a rule may give, or a person read in one window.
pub const MAX_OCCURRENCES: usize = 1000;

/// How often a rule repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    /// Every day.
    Daily,
    /// Every week.
    Weekly,
    /// Every month.
    Monthly,
    /// Every year.
    Yearly,
}

/// Where a rule ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// It never ends.
    Never,
    /// After a number of occurrences.
    Count(usize),
    /// At a moment, included.
    Until(Timestamp),
}

/// A recurrence rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// How often it repeats.
    pub frequency: Frequency,
    /// Every how many of those.
    pub interval: i64,
    /// Where it ends.
    pub end: End,
    /// The days of the week of a weekly rule.
    pub by_day: Vec<Weekday>,
    /// The day of the month of a monthly rule.
    pub by_month_day: Option<i8>,
}

/// What is wrong with a rule: the part that breaks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError(pub &'static str);

/// A day of the week from its two letters.
fn weekday(code: &str) -> Option<Weekday> {
    Some(match code {
        "MO" => Weekday::Monday,
        "TU" => Weekday::Tuesday,
        "WE" => Weekday::Wednesday,
        "TH" => Weekday::Thursday,
        "FR" => Weekday::Friday,
        "SA" => Weekday::Saturday,
        "SU" => Weekday::Sunday,
        _ => return None,
    })
}

/// A moment of `UNTIL`: a UTC date-time (`20300101T090000Z`) or a date, its end included.
fn until(text: &str) -> Option<Timestamp> {
    let digits = |range: std::ops::Range<usize>| text.get(range).and_then(|s| s.parse::<i16>().ok());
    let date = Date::new(
        digits(0..4)?,
        i8::try_from(digits(4..6)?).ok()?,
        i8::try_from(digits(6..8)?).ok()?,
    )
    .ok()?;
    let at = if text.len() == 8 {
        date.at(23, 59, 59, 0)
    } else if text.len() == 16 && &text[8..9] == "T" && text.ends_with('Z') {
        date.at(
            i8::try_from(digits(9..11)?).ok()?,
            i8::try_from(digits(11..13)?).ok()?,
            i8::try_from(digits(13..15)?).ok()?,
            0,
        )
    } else {
        return None;
    };
    at.to_zoned_datetime(TimeZone::UTC).ok().map(|z| z.timestamp())
}

impl Rule {
    /// Parses a rule such as `FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE;COUNT=10`.
    ///
    /// # Errors
    ///
    /// A [`RuleError`] naming the part that is missing, unknown or out of range.
    pub fn parse(text: &str) -> Result<Self, RuleError> {
        let mut frequency = None;
        let mut interval = 1;
        let mut count = None;
        let mut until_at = None;
        let mut by_day = Vec::new();
        let mut by_month_day = None;
        for part in text
            .trim()
            .trim_start_matches("RRULE:")
            .split(';')
            .filter(|p| !p.is_empty())
        {
            let (key, value) = part.split_once('=').ok_or(RuleError("part"))?;
            match key.to_ascii_uppercase().as_str() {
                "FREQ" => {
                    frequency = Some(match value.to_ascii_uppercase().as_str() {
                        "DAILY" => Frequency::Daily,
                        "WEEKLY" => Frequency::Weekly,
                        "MONTHLY" => Frequency::Monthly,
                        "YEARLY" => Frequency::Yearly,
                        _ => return Err(RuleError("FREQ")),
                    });
                }
                "INTERVAL" => {
                    interval = value
                        .parse::<i64>()
                        .ok()
                        .filter(|i| (1..=1000).contains(i))
                        .ok_or(RuleError("INTERVAL"))?;
                }
                "COUNT" => {
                    count = Some(
                        value
                            .parse::<usize>()
                            .ok()
                            .filter(|c| (1..=MAX_OCCURRENCES).contains(c))
                            .ok_or(RuleError("COUNT"))?,
                    );
                }
                "UNTIL" => until_at = Some(until(value).ok_or(RuleError("UNTIL"))?),
                "BYDAY" => {
                    for code in value.split(',') {
                        by_day.push(weekday(&code.to_ascii_uppercase()).ok_or(RuleError("BYDAY"))?);
                    }
                }
                "BYMONTHDAY" => {
                    by_month_day = Some(
                        value
                            .parse::<i8>()
                            .ok()
                            .filter(|d| (1..=31).contains(d))
                            .ok_or(RuleError("BYMONTHDAY"))?,
                    );
                }
                _ => return Err(RuleError("unsupported")),
            }
        }
        let frequency = frequency.ok_or(RuleError("FREQ"))?;
        if !by_day.is_empty() && frequency != Frequency::Weekly {
            return Err(RuleError("BYDAY"));
        }
        if by_month_day.is_some() && frequency != Frequency::Monthly {
            return Err(RuleError("BYMONTHDAY"));
        }
        let end = match (count, until_at) {
            (Some(_), Some(_)) => return Err(RuleError("COUNT")),
            (Some(c), None) => End::Count(c),
            (None, Some(u)) => End::Until(u),
            (None, None) => End::Never,
        };
        by_day.sort_by_key(|d| d.to_monday_zero_offset());
        by_day.dedup();
        Ok(Self {
            frequency,
            interval,
            end,
            by_day,
            by_month_day,
        })
    }

    /// The dates of the period `index` after the first, in order, before the time of day is added.
    fn dates_of(&self, first: Date, index: i64) -> Vec<Date> {
        let step = index * self.interval;
        match self.frequency {
            Frequency::Daily => first
                .checked_add(jiff::Span::new().days(step))
                .into_iter()
                .collect(),
            Frequency::Weekly => {
                let monday = first
                    .checked_sub(jiff::Span::new().days(i64::from(first.weekday().to_monday_zero_offset())))
                    .ok();
                let Some(week) = monday.and_then(|m| m.checked_add(jiff::Span::new().weeks(step)).ok())
                else {
                    return Vec::new();
                };
                if self.by_day.is_empty() {
                    return first
                        .checked_add(jiff::Span::new().weeks(step))
                        .into_iter()
                        .collect();
                }
                self.by_day
                    .iter()
                    .filter_map(|d| {
                        week.checked_add(jiff::Span::new().days(i64::from(d.to_monday_zero_offset())))
                            .ok()
                    })
                    .filter(|d| *d >= first)
                    .collect()
            }
            Frequency::Monthly => {
                let day = self.by_month_day.unwrap_or_else(|| first.day());
                let Ok(month) = first.first_of_month().checked_add(jiff::Span::new().months(step)) else {
                    return Vec::new();
                };
                Date::new(month.year(), month.month(), day)
                    .ok()
                    .filter(|d| *d >= first)
                    .into_iter()
                    .collect()
            }
            Frequency::Yearly => i16::try_from(i64::from(first.year()) + step)
                .ok()
                .and_then(|year| Date::new(year, first.month(), first.day()).ok())
                .into_iter()
                .collect(),
        }
    }

    /// The starts of the occurrences of an entry that begins at `start` in `zone`, within `[from, to)`, at
    /// most [`MAX_OCCURRENCES`]. The first occurrence is the start itself.
    #[must_use]
    pub fn occurrences(
        &self,
        start: Timestamp,
        zone: &TimeZone,
        from: Timestamp,
        to: Timestamp,
    ) -> Vec<Timestamp> {
        let local = start.to_zoned_datetime(zone.clone());
        let (first, time) = (local.date(), local.time());
        let mut found = Vec::new();
        if start >= to {
            return found;
        }
        if start >= from {
            found.push(start);
        }
        let mut counted = 1;
        for index in 0..100_000 {
            let mut ended = false;
            for date in self.dates_of(first, index) {
                let Ok(at) = date.to_datetime(time).to_zoned_datetime(zone.clone()) else {
                    continue;
                };
                let at = at.timestamp();
                if at <= start {
                    continue;
                }
                counted += 1;
                let over = match self.end {
                    End::Count(c) => counted > c,
                    End::Until(u) => at > u,
                    End::Never => false,
                };
                if over || at >= to {
                    ended = true;
                    break;
                }
                if at >= from {
                    found.push(at);
                    if found.len() >= MAX_OCCURRENCES {
                        return found;
                    }
                }
            }
            if ended {
                break;
            }
        }
        found
    }
}
