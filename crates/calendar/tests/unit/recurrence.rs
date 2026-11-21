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

//! Recurrence: what a rule accepts and the occurrences it gives, in the time zone of the entry.

use jiff::Timestamp;
use jiff::tz::TimeZone;
use kampus_calendar::support::recurrence::{End, Frequency, Rule};

/// A moment from its text.
fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

/// The occurrences of a rule starting at a moment, in a window, as text.
fn occurrences(rule: &str, start: &str, zone: &str, from: &str, to: &str) -> Vec<String> {
    Rule::parse(rule)
        .unwrap()
        .occurrences(at(start), &TimeZone::get(zone).unwrap(), at(from), at(to))
        .into_iter()
        .map(|t| t.to_string())
        .collect()
}

#[test]
fn a_rule_is_read_with_its_parts_and_refused_when_a_part_does_not_fit() {
    let rule = Rule::parse("FREQ=WEEKLY;INTERVAL=2;BYDAY=WE,MO;COUNT=4").unwrap();
    assert_eq!(rule.frequency, Frequency::Weekly);
    assert_eq!(rule.interval, 2);
    assert_eq!(rule.end, End::Count(4));
    assert_eq!(rule.by_day.len(), 2);
    for wrong in [
        "INTERVAL=2",
        "FREQ=HOURLY",
        "FREQ=DAILY;INTERVAL=0",
        "FREQ=DAILY;COUNT=2;UNTIL=20300101",
        "FREQ=DAILY;BYDAY=MO",
        "FREQ=WEEKLY;BYMONTHDAY=3",
        "FREQ=DAILY;BYSETPOS=1",
        "FREQ=DAILY;UNTIL=tomorrow",
    ] {
        assert!(Rule::parse(wrong).is_err(), "{wrong}");
    }
}

#[test]
fn daily_and_weekly_rules_give_their_occurrences_in_the_window() {
    assert_eq!(
        occurrences(
            "FREQ=DAILY;COUNT=3",
            "2030-01-01T09:00:00Z",
            "UTC",
            "2029-01-01T00:00:00Z",
            "2031-01-01T00:00:00Z"
        ),
        [
            "2030-01-01T09:00:00Z",
            "2030-01-02T09:00:00Z",
            "2030-01-03T09:00:00Z"
        ]
    );
    assert_eq!(
        occurrences(
            "FREQ=WEEKLY;BYDAY=MO,WE",
            "2030-01-07T10:00:00Z",
            "UTC",
            "2030-01-08T00:00:00Z",
            "2030-01-15T00:00:00Z"
        ),
        ["2030-01-09T10:00:00Z", "2030-01-14T10:00:00Z"],
        "only what falls in the window, Monday 7 being before it"
    );
    assert_eq!(
        occurrences(
            "FREQ=WEEKLY;INTERVAL=2;UNTIL=20300201",
            "2030-01-01T08:00:00Z",
            "UTC",
            "2030-01-01T00:00:00Z",
            "2030-12-31T00:00:00Z"
        ),
        [
            "2030-01-01T08:00:00Z",
            "2030-01-15T08:00:00Z",
            "2030-01-29T08:00:00Z"
        ]
    );
}

#[test]
fn an_occurrence_keeps_its_time_of_day_across_a_change_of_daylight_saving() {
    assert_eq!(
        occurrences(
            "FREQ=WEEKLY;COUNT=2",
            "2030-03-25T08:00:00Z",
            "Europe/Madrid",
            "2030-01-01T00:00:00Z",
            "2031-01-01T00:00:00Z"
        ),
        ["2030-03-25T08:00:00Z", "2030-04-01T07:00:00Z"],
        "nine in the morning in Madrid before and after the change"
    );
}

#[test]
fn monthly_and_yearly_rules_skip_days_that_do_not_exist() {
    assert_eq!(
        occurrences(
            "FREQ=MONTHLY;COUNT=3",
            "2030-01-31T12:00:00Z",
            "UTC",
            "2030-01-01T00:00:00Z",
            "2031-01-01T00:00:00Z"
        ),
        [
            "2030-01-31T12:00:00Z",
            "2030-03-31T12:00:00Z",
            "2030-05-31T12:00:00Z"
        ],
        "there is no 31 in February or April"
    );
    assert_eq!(
        occurrences(
            "FREQ=MONTHLY;BYMONTHDAY=15;COUNT=2",
            "2030-01-20T12:00:00Z",
            "UTC",
            "2030-01-01T00:00:00Z",
            "2031-01-01T00:00:00Z"
        ),
        ["2030-01-20T12:00:00Z", "2030-02-15T12:00:00Z"],
        "the start is always the first occurrence"
    );
    assert_eq!(
        occurrences(
            "FREQ=YEARLY;COUNT=2",
            "2028-02-29T12:00:00Z",
            "UTC",
            "2028-01-01T00:00:00Z",
            "2040-01-01T00:00:00Z"
        ),
        ["2028-02-29T12:00:00Z", "2032-02-29T12:00:00Z"]
    );
}

#[test]
fn a_rule_without_an_end_stops_at_the_window_and_at_the_limit() {
    let found = occurrences(
        "FREQ=DAILY",
        "2030-01-01T00:00:00Z",
        "UTC",
        "2030-01-01T00:00:00Z",
        "2033-01-01T00:00:00Z",
    );
    assert_eq!(found.len(), 1000);
}
