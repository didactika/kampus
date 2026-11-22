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
// @package   kampus-mail
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! A message is checked before it leaves: plain addresses only, a bounded number of them, and a subject
//! that stays on one line.

use kampus_mail::{MAX_RECIPIENTS, MAX_SUBJECT, Mail};

#[test]
fn addresses_that_could_break_a_header_are_refused() {
    for to in [
        vec![],
        vec!["nobody".to_owned()],
        vec!["@example.org".to_owned()],
        vec!["lea@example.org>, intruder@example.org".to_owned()],
        vec!["lea@example.org\r\nBcc: intruder@example.org".to_owned()],
        vec!["lea@localhost".to_owned()],
        vec!["lea@@example.org".to_owned()],
    ] {
        let refused = Mail::new(to.clone(), "Hi", "Hi").unwrap_err();
        assert_eq!(refused.code(), "mail.invalid", "{to:?}");
    }
    let many = (0..=MAX_RECIPIENTS)
        .map(|i| format!("p{i}@example.org"))
        .collect();
    assert_eq!(Mail::new(many, "Hi", "Hi").unwrap_err().code(), "mail.invalid");
}

#[test]
fn a_subject_is_folded_onto_one_line_and_cut() {
    let mail = Mail::new(
        vec!["lea@example.org".to_owned()],
        " New\r\ngrade\tposted ",
        "line one\r\nline two",
    )
    .unwrap();
    assert_eq!(mail.subject(), "New grade posted");
    assert_eq!(mail.text(), "line one\nline two");
    assert_eq!(mail.to(), ["lea@example.org"]);
    let long = Mail::new(vec!["lea@example.org".to_owned()], &"á".repeat(500), "Hi").unwrap();
    assert_eq!(long.subject().chars().count(), MAX_SUBJECT);
}
