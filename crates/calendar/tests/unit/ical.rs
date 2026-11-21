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

//! iCalendar: what a feed writes and what a subscription reads.

use jiff::Timestamp;
use kampus_calendar::support::ical::{Event, read, write};

/// A moment from its text.
fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

/// An event of an hour.
fn event(uid: &str, title: &str) -> Event {
    Event {
        uid: uid.to_owned(),
        sequence: 0,
        title: title.to_owned(),
        description: Some("Room 2, bring a pen".to_owned()),
        starts_at: at("2030-01-07T09:00:00Z"),
        ends_at: at("2030-01-07T10:00:00Z"),
        all_day: false,
        recurrence: Some("FREQ=WEEKLY;COUNT=3".to_owned()),
    }
}

#[test]
fn what_a_feed_writes_reads_back_the_same() {
    let long =
        "A very long title that goes on and on, with commas; semicolons and accents como acción".repeat(2);
    let events = [event("1@kampus", &long), event("2@kampus", "Exam")];
    let text = write("My calendar", &events, at("2030-01-01T00:00:00Z"));
    assert!(text.starts_with("BEGIN:VCALENDAR\r\n"));
    assert!(text.lines().all(|l| l.len() <= 76), "lines are folded");
    assert!(
        text.replace("\r\n ", "")
            .contains(r"SUMMARY:A very long title that goes on and on\, with commas\; "),
        "text is escaped"
    );
    assert_eq!(read(&text), events);
}

#[test]
fn a_subscription_reads_utc_local_and_whole_day_events_and_leaves_out_the_rest() {
    let text = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:a\r\nSEQUENCE:2\r\nSUMMARY:Local\r\n\
DTSTART;TZID=Europe/Madrid:20300701T090000\r\nDTEND;TZID=Europe/Madrid:20300701T100000\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:b\r\nSUMMARY:Holiday\r\nDTSTART;VALUE=DATE:20300815\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:c\r\nSUMMARY:Gone\r\nSTATUS:CANCELLED\r\nDTSTART:20300101T090000Z\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nSUMMARY:No uid\r\nDTSTART:20300101T090000Z\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:d\r\nSUMMARY:Bad rule\r\nDTSTART:20300101T090000Z\r\nRRULE:FREQ=SECONDLY\r\nEND:VEVENT\r\n\
END:VCALENDAR\r\n";
    let events = read(text);
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[0].starts_at,
        at("2030-07-01T07:00:00Z"),
        "nine in Madrid in summer"
    );
    assert_eq!(events[0].sequence, 2);
    assert!(events[1].all_day);
    assert_eq!(
        events[1].ends_at,
        at("2030-08-16T00:00:00Z"),
        "a whole day without an end"
    );
    assert_eq!(
        events[2].ends_at,
        at("2030-01-01T10:00:00Z"),
        "an hour without an end"
    );
    assert_eq!(events[2].recurrence, None, "a rule it cannot follow is dropped");
}
