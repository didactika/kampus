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

//! Tests of `kampus-common`: page.

use kampus_common::page::DEFAULT_LIMIT;
use kampus_common::{Cursor, Page, PageRequest};

#[test]
fn the_default_page_holds_fifty_items() {
    let page = PageRequest::new(None, None).unwrap();
    assert_eq!(page.limit(), DEFAULT_LIMIT);
    assert!(page.after().is_none());
}

#[test]
fn cursors_round_trip() {
    let cursor = Cursor::from_bytes(b"course:0192".to_vec());
    let page = PageRequest::new(Some(10), Some(&cursor.encode())).unwrap();
    assert_eq!(page.after(), Some(&cursor));
}

#[test]
fn every_invalid_parameter_is_reported_at_once() {
    let error = PageRequest::new(Some(500), Some("not a cursor!")).unwrap_err();
    let fields: Vec<_> = error
        .fields()
        .iter()
        .map(|f| (f.field.as_str(), f.rule.as_str()))
        .collect();
    assert_eq!(fields, [("limit", "range"), ("after", "cursor")]);
    assert_eq!(
        PageRequest::new(Some(0), None).unwrap_err().fields()[0].rule,
        "range"
    );
}

#[test]
fn pages_serialise_the_next_cursor_as_text() {
    let page = Page {
        items: vec![1, 2],
        next: Some(Cursor::from_bytes(vec![1, 2, 3])),
    };
    assert_eq!(
        serde_json::to_string(&page).unwrap(),
        r#"{"items":[1,2],"next":"AQID"}"#
    );
    let last: Page<u8> = Page {
        items: vec![],
        next: None,
    };
    assert_eq!(
        serde_json::to_string(&last).unwrap(),
        r#"{"items":[],"next":null}"#
    );
}
