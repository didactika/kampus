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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Visibility of one node: shown, hidden, or shown inside a window with open ends.

use kampus_common::Timestamp;
use kampus_courses::Visibility;

/// An instant of these tests.
fn at(text: &str) -> Timestamp {
    Timestamp::parse_rfc3339(text).unwrap()
}

#[test]
fn a_window_shows_the_node_only_inside_it() {
    let window = Visibility::Scheduled {
        from: Some(at("2026-09-01T00:00:00Z")),
        until: Some(at("2027-06-30T00:00:00Z")),
    };
    assert!(!window.shown_at(at("2026-08-31T23:59:59Z")));
    assert!(window.shown_at(at("2026-09-01T00:00:00Z")));
    assert!(
        !window.shown_at(at("2027-06-30T00:00:00Z")),
        "the end is not included"
    );
    let open = Visibility::Scheduled {
        from: Some(at("2026-09-01T00:00:00Z")),
        until: None,
    };
    assert!(open.shown_at(at("2099-01-01T00:00:00Z")));
    assert!(Visibility::Shown.shown_at(at("2026-01-01T00:00:00Z")));
    assert!(!Visibility::Hidden.shown_at(at("2026-01-01T00:00:00Z")));
}

#[test]
fn a_window_that_ends_before_it_starts_is_refused() {
    let backwards = Visibility::Scheduled {
        from: Some(at("2027-01-01T00:00:00Z")),
        until: Some(at("2026-01-01T00:00:00Z")),
    };
    let error = backwards.validate().unwrap_err();
    assert_eq!(error.code(), "courses.visibility_invalid");
    assert!(Visibility::Hidden.validate().is_ok());
}

#[test]
fn a_stored_visibility_reads_back_the_same() {
    let window = Visibility::Scheduled {
        from: Some(at("2026-09-01T00:00:00Z")),
        until: None,
    };
    let (from, until) = window.window();
    assert_eq!(Visibility::read(window.mode(), from, until), window);
    assert_eq!(Visibility::read("hidden", None, None), Visibility::Hidden);
}
