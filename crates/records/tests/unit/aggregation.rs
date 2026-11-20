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

//! Worked examples of every aggregation method: how weights, extra credit, empty grades, dropping and
//! keeping count, written as the arithmetic a teacher would do by hand.

use kampus_records::Method;
use kampus_records::aggregation::{Child, Rule, aggregate, natural_range};
use rust_decimal::Decimal;

/// A child graded out of a maximum, with a weight.
fn child(grade: Option<i64>, max: i64, weight: i64) -> Child {
    Child {
        grade: grade.map(Decimal::from),
        min: Decimal::ZERO,
        max: Decimal::from(max),
        weight: Decimal::from(weight),
        extra_credit: false,
    }
}

/// A rule without dropping that leaves empty grades out.
fn rule(method: Method) -> Rule {
    Rule {
        method,
        drop_lowest: 0,
        keep_highest: 0,
        exclude_empty: true,
    }
}

/// The range of a total graded out of 100.
const HUNDRED: (Decimal, Decimal) = (Decimal::ZERO, Decimal::ONE_HUNDRED);

/// A decimal from a text.
fn dec(text: &str) -> Decimal {
    text.parse().unwrap()
}

#[test]
fn the_mean_averages_the_shares_of_each_range() {
    let children = [
        child(Some(8), 10, 1),
        child(Some(15), 20, 1),
        child(Some(40), 50, 1),
    ];
    assert_eq!(
        aggregate(rule(Method::Mean), &children, HUNDRED),
        Some(dec("78.33333"))
    );
    let out_of_ten = (Decimal::ZERO, Decimal::TEN);
    assert_eq!(
        aggregate(rule(Method::Mean), &children, out_of_ten),
        Some(dec("7.83333"))
    );
}

#[test]
fn the_weighted_mean_uses_the_weight_and_extra_credit_only_adds() {
    let children = [child(Some(8), 10, 3), child(Some(15), 20, 1)];
    assert_eq!(
        aggregate(rule(Method::WeightedMean), &children, HUNDRED),
        Some(dec("78.75")),
        "(0.8 x 3 + 0.75 x 1) / 4"
    );
    let mut extra = child(Some(2), 10, 1);
    extra.extra_credit = true;
    let with_extra = [child(Some(5), 10, 1), extra];
    assert_eq!(
        aggregate(rule(Method::WeightedMean), &with_extra, HUNDRED),
        Some(dec("70")),
        "(0.5 + 0.2) / 1: extra credit is not in the denominator"
    );
    let capped = [child(Some(9), 10, 1), {
        let mut e = child(Some(9), 10, 1);
        e.extra_credit = true;
        e
    }];
    assert_eq!(
        aggregate(rule(Method::WeightedMean), &capped, HUNDRED),
        Some(dec("100"))
    );
}

#[test]
fn the_simple_weighted_mean_weighs_each_child_by_its_range() {
    let children = [child(Some(8), 10, 1), child(Some(15), 20, 1)];
    assert_eq!(
        aggregate(rule(Method::SimpleWeightedMean), &children, HUNDRED),
        Some(dec("76.66667")),
        "(0.8 x 10 + 0.75 x 20) / 30"
    );
}

#[test]
fn the_natural_method_adds_the_grades_and_its_range_is_the_sum_of_the_maximums() {
    let mut extra = child(Some(2), 5, 1);
    extra.extra_credit = true;
    let children = [child(Some(8), 10, 1), child(Some(15), 20, 1), extra];
    let range = natural_range(&children);
    assert_eq!(
        range,
        (Decimal::ZERO, Decimal::from(30)),
        "extra credit is not in the maximum"
    );
    assert_eq!(
        aggregate(rule(Method::Natural), &children, range),
        Some(Decimal::from(25))
    );
    let mut with_dropping = rule(Method::Natural);
    with_dropping.drop_lowest = 1;
    assert_eq!(
        aggregate(with_dropping, &children, range),
        Some(Decimal::from(25)),
        "the natural method ignores dropping"
    );
}

#[test]
fn dropping_the_lowest_removes_the_larger_maximum_first_among_equal_grades() {
    let children = [
        child(Some(5), 10, 1),
        child(Some(10), 20, 1),
        child(Some(10), 10, 1),
    ];
    let mut dropping = rule(Method::SimpleWeightedMean);
    dropping.drop_lowest = 1;
    assert_eq!(
        aggregate(dropping, &children, HUNDRED),
        Some(dec("75")),
        "the 10 out of 20 goes: (0.5 x 10 + 1 x 10) / 20"
    );
    let mut too_many = rule(Method::Mean);
    too_many.drop_lowest = 10;
    assert!(
        aggregate(too_many, &children, HUNDRED).is_some(),
        "one child always remains"
    );
}

#[test]
fn keeping_the_highest_ignores_the_rest() {
    let children = [
        child(Some(2), 10, 1),
        child(Some(6), 10, 1),
        child(Some(9), 10, 1),
    ];
    let mut keeping = rule(Method::Mean);
    keeping.keep_highest = 2;
    assert_eq!(aggregate(keeping, &children, HUNDRED), Some(dec("75")));
}

#[test]
fn empty_grades_are_left_out_or_count_as_zero() {
    let children = [child(Some(8), 10, 1), child(None, 10, 1)];
    assert_eq!(aggregate(rule(Method::Mean), &children, HUNDRED), Some(dec("80")));
    let mut counting = rule(Method::Mean);
    counting.exclude_empty = false;
    assert_eq!(aggregate(counting, &children, HUNDRED), Some(dec("40")));
    let nothing = [child(None, 10, 1)];
    assert_eq!(aggregate(rule(Method::Mean), &nothing, HUNDRED), None);
    assert_eq!(aggregate(counting, &nothing, HUNDRED), Some(Decimal::ZERO));
}

#[test]
fn median_mode_lowest_and_highest_pick_among_the_shares() {
    let odd = [
        child(Some(2), 10, 1),
        child(Some(6), 10, 1),
        child(Some(9), 10, 1),
    ];
    assert_eq!(aggregate(rule(Method::Median), &odd, HUNDRED), Some(dec("60")));
    let even = [
        child(Some(2), 10, 1),
        child(Some(6), 10, 1),
        child(Some(8), 10, 1),
        child(Some(9), 10, 1),
    ];
    assert_eq!(aggregate(rule(Method::Median), &even, HUNDRED), Some(dec("70")));
    assert_eq!(aggregate(rule(Method::Lowest), &odd, HUNDRED), Some(dec("20")));
    assert_eq!(aggregate(rule(Method::Highest), &odd, HUNDRED), Some(dec("90")));
    let repeated = [
        child(Some(5), 10, 1),
        child(Some(5), 10, 1),
        child(Some(9), 10, 1),
    ];
    assert_eq!(aggregate(rule(Method::Mode), &repeated, HUNDRED), Some(dec("50")));
    let tied = [child(Some(5), 10, 1), child(Some(9), 10, 1)];
    assert_eq!(
        aggregate(rule(Method::Mode), &tied, HUNDRED),
        Some(dec("90")),
        "the highest wins a tie"
    );
}
