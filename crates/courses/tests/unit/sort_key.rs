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

//! Sort keys: always between their neighbours, so placing a node never touches its siblings.

use kampus_courses::Place;
use kampus_courses::support::sort_key::{between, is_key, place};

#[test]
fn a_key_sorts_between_its_bounds() {
    let first = between(None, None).unwrap();
    assert!(is_key(&first));
    let after = between(Some(&first), None).unwrap();
    let before = between(None, Some(&first)).unwrap();
    assert!(before < first && first < after);
    let middle = between(Some(&before), Some(&first)).unwrap();
    assert!(before < middle && middle < first);
}

#[test]
fn keys_between_neighbours_that_differ_by_one_digit_grow_by_one() {
    let key = between(Some("a"), Some("b")).unwrap();
    assert!("a" < key.as_str() && key.as_str() < "b");
    assert_eq!(key.len(), 2);
    let tighter = between(Some("az"), Some("b")).unwrap();
    assert!("az" < tighter.as_str() && tighter.as_str() < "b");
}

#[test]
fn inserting_many_times_at_the_same_place_keeps_the_order() {
    let mut low = between(None, None).unwrap();
    let high = between(Some(&low), None).unwrap();
    let mut keys = vec![low.clone(), high.clone()];
    for _ in 0..200 {
        let next = between(Some(&low), Some(&high)).unwrap();
        assert!(is_key(&next), "{next}");
        assert!(low < next && next < high);
        keys.push(next.clone());
        low = next;
    }
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), keys.len(), "every key is unique");
    assert!(low.len() < 40, "keys stay short: {}", low.len());
}

#[test]
fn bounds_that_are_not_keys_or_out_of_order_are_refused() {
    assert!(
        between(Some("a0"), None).is_err(),
        "a key never ends with the smallest digit"
    );
    assert!(between(Some("b"), Some("a")).is_err());
    assert!(between(Some("a"), Some("a")).is_err());
    assert!(between(Some("a-"), None).is_err());
}

#[test]
fn a_node_is_placed_first_last_after_or_before_a_sibling() {
    let siblings = vec![(1, "F".to_owned()), (2, "V".to_owned()), (3, "k".to_owned())];
    let first = place(&siblings, Place::First).unwrap();
    assert!(first.as_str() < "F");
    let last = place(&siblings, Place::Last).unwrap();
    assert!(last.as_str() > "k");
    let after = place(&siblings, Place::After(1)).unwrap();
    assert!("F" < after.as_str() && after.as_str() < "V");
    let before = place(&siblings, Place::Before(1)).unwrap();
    assert!(before.as_str() < "F");
    assert_eq!(
        place(&siblings, Place::After(9)).unwrap_err().code(),
        "courses.sibling_unknown"
    );
    assert_ne!(place::<i32>(&[], Place::Last).unwrap(), "");
}
