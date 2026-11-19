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

//! Cron rules: their fields, the two day fields, time zones and summer time.

use jiff::Timestamp;
use kampus_tasks::support::cron::Rule;

/// An instant of these tests.
fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

/// The next run of a rule after an instant, as text.
fn next(rule: &str, zone: &str, after: &str) -> String {
    Rule::parse(rule, zone)
        .unwrap()
        .next_after(at(after))
        .unwrap()
        .to_string()
}

#[test]
fn every_minute_is_the_next_whole_minute() {
    assert_eq!(
        next("* * * * *", "UTC", "2026-11-19T10:15:30Z"),
        "2026-11-19T10:16:00Z"
    );
    assert_eq!(
        next("* * * * *", "UTC", "2026-11-19T10:15:00Z"),
        "2026-11-19T10:16:00Z"
    );
}

#[test]
fn steps_lists_and_ranges_match_as_cron_does() {
    assert_eq!(
        next("*/15 * * * *", "UTC", "2026-11-19T10:16:00Z"),
        "2026-11-19T10:30:00Z"
    );
    assert_eq!(
        next("0,30 9-17 * * *", "UTC", "2026-11-19T17:31:00Z"),
        "2026-11-20T09:00:00Z"
    );
    assert_eq!(
        next("5/20 * * * *", "UTC", "2026-11-19T10:26:00Z"),
        "2026-11-19T10:45:00Z"
    );
}

#[test]
fn a_daily_rule_moves_to_the_next_day_month_and_year() {
    assert_eq!(
        next("0 3 * * *", "UTC", "2026-11-19T03:00:00Z"),
        "2026-11-20T03:00:00Z"
    );
    assert_eq!(
        next("0 0 1 * *", "UTC", "2026-12-15T00:00:00Z"),
        "2027-01-01T00:00:00Z"
    );
    assert_eq!(
        next("0 0 29 2 *", "UTC", "2026-03-01T00:00:00Z"),
        "2028-02-29T00:00:00Z"
    );
}

#[test]
fn either_day_field_matches_when_both_are_restricted() {
    assert_eq!(
        next("0 9 * * 1", "UTC", "2026-11-19T00:00:00Z"),
        "2026-11-23T09:00:00Z",
        "the next Monday"
    );
    assert_eq!(
        next("0 9 1 * 1", "UTC", "2026-11-24T00:00:00Z"),
        "2026-11-30T09:00:00Z",
        "a Monday comes before the first of the month"
    );
    assert_eq!(
        next("0 9 * * 7", "UTC", "2026-11-19T00:00:00Z"),
        "2026-11-22T09:00:00Z",
        "7 is Sunday"
    );
}

#[test]
fn a_rule_is_read_in_its_time_zone_across_summer_time() {
    assert_eq!(
        next("0 3 * * *", "Europe/Madrid", "2026-11-19T00:00:00Z"),
        "2026-11-19T02:00:00Z",
        "03:00 in Madrid is 02:00 UTC in winter"
    );
    assert_eq!(
        next("0 3 * * *", "Europe/Madrid", "2026-07-01T00:00:00Z"),
        "2026-07-01T01:00:00Z",
        "and 01:00 UTC in summer"
    );
}

#[test]
fn invalid_rules_and_impossible_dates_are_refused() {
    assert_eq!(
        Rule::parse("* * * *", "UTC").unwrap_err().code(),
        "tasks.cron_invalid"
    );
    assert_eq!(
        Rule::parse("60 * * * *", "UTC").unwrap_err().code(),
        "tasks.cron_invalid"
    );
    assert_eq!(
        Rule::parse("*/0 * * * *", "UTC").unwrap_err().code(),
        "tasks.cron_invalid"
    );
    assert_eq!(
        Rule::parse("* * * * *", "Mars/Olympus").unwrap_err().code(),
        "tasks.time_zone_invalid"
    );
    let never = Rule::parse("0 0 31 2 *", "UTC").unwrap();
    assert_eq!(
        never.next_after(at("2026-01-01T00:00:00Z")).unwrap_err().code(),
        "tasks.cron_never"
    );
}
