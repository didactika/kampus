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

//! iCalendar (RFC 5545), without the database: writing the events of a feed, with escaped text and folded
//! lines, and reading the events of a subscription: their UID, sequence, title, description, start and end
//! (as UTC, as a local time of a named time zone, or as a whole day) and recurrence rule. A cancelled
//! event, or one without a UID or a start, is left out.

use jiff::Timestamp;
use jiff::civil::{Date, DateTime};
use jiff::tz::TimeZone;

/// The largest calendar a subscription may give, in bytes.
pub const MAX_CALENDAR: usize = 5 * 1024 * 1024;

/// The most events a subscription may give.
pub const MAX_EVENTS: usize = 5000;

/// One event of a calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Its identifier, unique in its calendar.
    pub uid: String,
    /// How many times it was changed.
    pub sequence: i32,
    /// Its title.
    pub title: String,
    /// What it is about.
    pub description: Option<String>,
    /// When it starts.
    pub starts_at: Timestamp,
    /// When it ends.
    pub ends_at: Timestamp,
    /// Whether it lasts whole days.
    pub all_day: bool,
    /// Its recurrence rule.
    pub recurrence: Option<String>,
}

/// Escapes text for a property value.
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace('\n', "\\n")
}

/// Reads an escaped property value back.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n' | 'N') => out.push('\n'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Folds a content line at 75 octets, never inside a character.
fn fold(line: &str) -> String {
    let mut out = String::new();
    let mut width = 0;
    for c in line.chars() {
        let size = c.len_utf8();
        if width + size > 75 {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(c);
        width += size;
    }
    out.push_str("\r\n");
    out
}

/// A moment as UTC date-time.
fn utc(at: Timestamp) -> String {
    at.strftime("%Y%m%dT%H%M%SZ").to_string()
}

/// Writes a calendar with its name and events.
#[must_use]
pub fn write(name: &str, events: &[Event], now: Timestamp) -> String {
    let mut out = String::new();
    for line in [
        "BEGIN:VCALENDAR",
        "VERSION:2.0",
        "PRODID:-//Kampus//Calendar//EN",
        "CALSCALE:GREGORIAN",
    ] {
        out.push_str(&fold(line));
    }
    out.push_str(&fold(&format!("X-WR-CALNAME:{}", escape(name))));
    for event in events {
        out.push_str(&fold("BEGIN:VEVENT"));
        out.push_str(&fold(&format!("UID:{}", escape(&event.uid))));
        out.push_str(&fold(&format!("SEQUENCE:{}", event.sequence)));
        out.push_str(&fold(&format!("DTSTAMP:{}", utc(now))));
        if event.all_day {
            let day = |at: Timestamp| {
                at.to_zoned_datetime(TimeZone::UTC)
                    .date()
                    .strftime("%Y%m%d")
                    .to_string()
            };
            out.push_str(&fold(&format!("DTSTART;VALUE=DATE:{}", day(event.starts_at))));
            out.push_str(&fold(&format!("DTEND;VALUE=DATE:{}", day(event.ends_at))));
        } else {
            out.push_str(&fold(&format!("DTSTART:{}", utc(event.starts_at))));
            out.push_str(&fold(&format!("DTEND:{}", utc(event.ends_at))));
        }
        out.push_str(&fold(&format!("SUMMARY:{}", escape(&event.title))));
        if let Some(description) = &event.description {
            out.push_str(&fold(&format!("DESCRIPTION:{}", escape(description))));
        }
        if let Some(rule) = &event.recurrence {
            out.push_str(&fold(&format!("RRULE:{rule}")));
        }
        out.push_str(&fold("END:VEVENT"));
    }
    out.push_str(&fold("END:VCALENDAR"));
    out
}

/// A date-time property: its parameters and its value.
fn moment(params: &str, value: &str) -> Option<(Timestamp, bool)> {
    let number = |range: std::ops::Range<usize>| value.get(range).and_then(|s| s.parse::<i16>().ok());
    let date = Date::new(
        number(0..4)?,
        i8::try_from(number(4..6)?).ok()?,
        i8::try_from(number(6..8)?).ok()?,
    )
    .ok()?;
    if params.to_ascii_uppercase().contains("VALUE=DATE")
        && !params.to_ascii_uppercase().contains("DATE-TIME")
        || value.len() == 8
    {
        return date
            .to_zoned_datetime(TimeZone::UTC)
            .ok()
            .map(|z| (z.timestamp(), true));
    }
    if value.get(8..9) != Some("T") {
        return None;
    }
    let time = DateTime::from_parts(
        date,
        jiff::civil::Time::new(
            i8::try_from(number(9..11)?).ok()?,
            i8::try_from(number(11..13)?).ok()?,
            i8::try_from(number(13..15)?).ok()?,
            0,
        )
        .ok()?,
    );
    let zone = if value.ends_with('Z') {
        TimeZone::UTC
    } else {
        params
            .split(';')
            .find_map(|p| p.strip_prefix("TZID="))
            .and_then(|name| TimeZone::get(name.trim_matches('"')).ok())
            .unwrap_or(TimeZone::UTC)
    };
    time.to_zoned_datetime(zone).ok().map(|z| (z.timestamp(), false))
}

/// The content lines of a calendar, unfolded.
fn lines(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = line.strip_prefix(' ').or_else(|| line.strip_prefix('\t')) {
            if let Some(last) = out.last_mut() {
                last.push_str(rest);
            }
        } else if !line.is_empty() {
            out.push(line.to_owned());
        }
    }
    out
}

/// What an event read so far holds.
#[derive(Default)]
struct Reading {
    /// Its UID.
    uid: Option<String>,
    /// Its sequence.
    sequence: i32,
    /// Its title.
    title: Option<String>,
    /// Its description.
    description: Option<String>,
    /// Its start, and whether it is a whole day.
    starts: Option<(Timestamp, bool)>,
    /// Its end.
    ends: Option<(Timestamp, bool)>,
    /// Its rule.
    recurrence: Option<String>,
    /// Whether it was cancelled.
    cancelled: bool,
}

impl Reading {
    /// The event, when it has what an event needs.
    fn finish(self) -> Option<Event> {
        if self.cancelled {
            return None;
        }
        let (starts_at, all_day) = self.starts?;
        let ends_at = match self.ends {
            Some((ends, _)) if ends > starts_at => ends,
            _ if all_day => starts_at.checked_add(jiff::SignedDuration::from_hours(24)).ok()?,
            _ => starts_at.checked_add(jiff::SignedDuration::from_hours(1)).ok()?,
        };
        Some(Event {
            uid: self.uid?,
            sequence: self.sequence,
            title: self.title.unwrap_or_else(|| "(untitled)".to_owned()),
            description: self.description,
            starts_at,
            ends_at,
            all_day,
            recurrence: self.recurrence,
        })
    }
}

/// Reads the events of a calendar, at most [`MAX_EVENTS`].
#[must_use]
pub fn read(text: &str) -> Vec<Event> {
    let mut events = Vec::new();
    let mut current: Option<Reading> = None;
    for line in lines(text) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let (key, params) = name.split_once(';').unwrap_or((name, ""));
        let key = key.to_ascii_uppercase();
        match (key.as_str(), current.as_mut()) {
            ("BEGIN", _) if value.eq_ignore_ascii_case("VEVENT") => current = Some(Reading::default()),
            ("END", Some(_)) if value.eq_ignore_ascii_case("VEVENT") => {
                if let Some(event) = current.take().and_then(Reading::finish) {
                    events.push(event);
                    if events.len() >= MAX_EVENTS {
                        break;
                    }
                }
            }
            ("UID", Some(r)) => r.uid = Some(unescape(value)).filter(|u| !u.is_empty()),
            ("SEQUENCE", Some(r)) => r.sequence = value.trim().parse().unwrap_or(0),
            ("SUMMARY", Some(r)) => r.title = Some(unescape(value)),
            ("DESCRIPTION", Some(r)) => r.description = Some(unescape(value)).filter(|d| !d.is_empty()),
            ("DTSTART", Some(r)) => r.starts = moment(params, value.trim()),
            ("DTEND", Some(r)) => r.ends = moment(params, value.trim()),
            ("RRULE", Some(r)) => {
                r.recurrence = crate::support::recurrence::Rule::parse(value)
                    .ok()
                    .map(|_| value.trim().to_owned());
            }
            ("STATUS", Some(r)) => r.cancelled = value.trim().eq_ignore_ascii_case("CANCELLED"),
            _ => {}
        }
    }
    events
}
