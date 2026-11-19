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

//! Fractional sort keys: short texts that sort between their neighbours, so placing or moving a node
//! writes only that node's key. Keys use the 62 ASCII digits and letters, which sort the same way in
//! Rust and in PostgreSQL's `C` collation, and never end with the smallest digit, so there is always a
//! key between two others.

use kampus_common::{Error, Result};

/// The digits of a key, in sort order.
const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// The position of a digit.
fn index(digit: u8) -> Option<usize> {
    DIGITS.iter().position(|d| *d == digit)
}

/// Whether a text is a key: digits only, not empty, not ending with the smallest digit.
#[must_use]
pub fn is_key(key: &str) -> bool {
    !key.is_empty() && key.bytes().all(|b| index(b).is_some()) && !key.ends_with('0')
}

/// A key strictly between `before` and `after`; `None` stands for the start or the end of the list.
///
/// # Errors
///
/// `internal` (`courses.sort_key_invalid`) when a bound is not a key or `before` is not below `after`.
pub fn between(before: Option<&str>, after: Option<&str>) -> Result<String> {
    let invalid = || Error::internal("courses.sort_key_invalid");
    if before.is_some_and(|b| !is_key(b)) || after.is_some_and(|a| !is_key(a)) {
        return Err(invalid());
    }
    if let (Some(b), Some(a)) = (before, after)
        && b >= a
    {
        return Err(invalid());
    }
    midpoint(before.unwrap_or("").as_bytes(), after.map(str::as_bytes)).ok_or_else(invalid)
}

/// The midpoint of two digit strings, `low < high`, `high` absent for the end.
fn midpoint(low: &[u8], high: Option<&[u8]>) -> Option<String> {
    if let Some(high) = high {
        let common = (0..high.len())
            .take_while(|i| low.get(*i).copied().unwrap_or(DIGITS[0]) == high[*i])
            .count();
        if common > 0 {
            let rest = midpoint(low.get(common..).unwrap_or_default(), Some(&high[common..]))?;
            return Some(format!("{}{rest}", std::str::from_utf8(&high[..common]).ok()?));
        }
    }
    let first_low = low.first().map_or(Some(0), |d| index(*d))?;
    let first_high = match high {
        Some(high) => index(*high.first()?)?,
        None => DIGITS.len(),
    };
    if first_high - first_low > 1 {
        let middle = usize::midpoint(first_low, first_high);
        return Some(char::from(DIGITS[middle]).to_string());
    }
    if let Some(high) = high
        && high.len() > 1
    {
        return Some(char::from(high[0]).to_string());
    }
    let rest = midpoint(low.get(1..).unwrap_or_default(), None)?;
    Some(format!("{}{rest}", char::from(DIGITS[first_low])))
}

/// The key of a node placed among its siblings, given in their order with their keys; the node itself,
/// when it is moving, is not among them.
///
/// # Errors
///
/// `validation` (`courses.sibling_unknown`) for a sibling that is not among them.
pub fn place<T: PartialEq + Copy>(siblings: &[(T, String)], at: crate::Place<T>) -> Result<String> {
    let position = |sibling: T| {
        siblings.iter().position(|(id, _)| *id == sibling).ok_or_else(|| {
            Error::validation(
                "courses.sibling_unknown",
                vec![kampus_common::FieldError::new("place.sibling", "sibling")],
            )
        })
    };
    let key = |i: usize| siblings.get(i).map(|(_, k)| k.as_str());
    match at {
        crate::Place::First => between(None, key(0)),
        crate::Place::Last => between(siblings.last().map(|(_, k)| k.as_str()), None),
        crate::Place::After(sibling) => {
            let i = position(sibling)?;
            between(key(i), key(i + 1))
        }
        crate::Place::Before(sibling) => {
            let i = position(sibling)?;
            between(i.checked_sub(1).and_then(key), key(i))
        }
    }
}
