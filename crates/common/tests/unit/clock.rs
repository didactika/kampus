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

//! Tests of `kampus-common`: clock.

use kampus_common::{Clock, FixedClock, Timestamp};

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

#[test]
fn the_system_clock_never_goes_back() {
    let threads: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let clock = kampus_common::SystemClock;
                let mut previous = clock.now();
                for _ in 0..10_000 {
                    let now = clock.now();
                    assert!(now >= previous, "{now} came after {previous}");
                    previous = now;
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}
