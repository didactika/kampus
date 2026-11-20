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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Worked examples of turning a percentage into a letter.

use kampus_records::Band;
use kampus_records::letters::letter_for;
use rust_decimal::Decimal;

/// A band from a percentage and a letter.
fn band(percent: i64, letter: &str) -> Band {
    Band {
        min_percent: Decimal::from(percent),
        letter: letter.to_owned(),
    }
}

#[test]
fn a_percentage_gets_the_letter_of_the_highest_band_it_reaches() {
    let bands = [
        band(0, "F"),
        band(90, "A"),
        band(60, "D"),
        band(80, "B"),
        band(70, "C"),
    ];
    let at = |text: &str| letter_for(&bands, text.parse().unwrap());
    assert_eq!(at("95"), Some("A"));
    assert_eq!(at("90"), Some("A"), "a band starts at its percentage");
    assert_eq!(at("89.99999"), Some("B"));
    assert_eq!(at("70"), Some("C"));
    assert_eq!(at("0"), Some("F"));
}

#[test]
fn a_percentage_below_every_band_has_no_letter() {
    let bands = [band(50, "Pass")];
    assert_eq!(letter_for(&bands, Decimal::from(49)), None);
    assert_eq!(letter_for(&[], Decimal::from(49)), None);
}
