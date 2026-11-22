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

//! How a category turns the grades of its children into its total. Each method has its own typed
//! definition of how weights, extra credit and empty grades count; nothing here reads the database, so
//! every method has worked examples as tests.
//!
//! Grades are first **normalised** to the share of their own range, `(grade - min) / (max - min)`, so that
//! records with different maximums can be combined; the result is scaled back to the range of the total.
//! The natural method is the exception: it adds the grades as they are.
//!
//! | Method | Weight of a child | Extra credit | Drop lowest, keep highest |
//! | --- | --- | --- | --- |
//! | mean | the same for all | counts as any child | yes |
//! | weighted mean | its weight | adds to the numerator, not to the denominator | yes |
//! | simple weighted mean | its range (maximum minus minimum) | as weighted mean | yes |
//! | natural | none: the grades are added | adds to the sum, not to the maximum | ignored |
//! | median, mode | the same for all | counts as any child | yes |
//! | lowest, highest | none | counts as any child | ignored |

use rust_decimal::Decimal;

use crate::Method;

/// One child of a category as the aggregation sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, schemars::JsonSchema)]
pub struct Child {
    /// What counts of the grade the person has on it; none when they have none, or it is excluded.
    pub grade: Option<Decimal>,
    /// The lowest grade of the child.
    pub min: Decimal,
    /// The highest grade of the child.
    pub max: Decimal,
    /// Its weight in a weighted mean.
    pub weight: Decimal,
    /// Whether it counts on top of the maximum.
    pub extra_credit: bool,
}

/// How a category aggregates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    /// The method.
    pub method: Method,
    /// How many of the lowest grades are dropped.
    pub drop_lowest: usize,
    /// How many of the highest grades are kept; zero keeps all.
    pub keep_highest: usize,
    /// Whether a child without a grade is left out instead of counting as zero.
    pub exclude_empty: bool,
}

/// A child that takes part: its share of its range and the child itself.
#[derive(Debug, Clone, Copy, schemars::JsonSchema)]
struct Entry {
    /// The share of its range the grade reaches.
    share: Decimal,
    /// The grade as it is.
    grade: Decimal,
    /// The child.
    child: Child,
}

/// Whether a method drops the lowest or keeps the highest grades; the others ignore both.
#[must_use]
pub const fn supports_dropping(method: Method) -> bool {
    !matches!(method, Method::Natural | Method::Lowest | Method::Highest)
}

/// The range of the total of a natural category: from zero to the sum of the maximums of its children, not
/// counting extra credit.
#[must_use]
pub fn natural_range(children: &[Child]) -> (Decimal, Decimal) {
    let max = children
        .iter()
        .filter(|c| !c.extra_credit)
        .map(|c| c.max)
        .sum::<Decimal>();
    (Decimal::ZERO, max)
}

/// The share of its range a grade reaches.
fn share(child: &Child, grade: Decimal) -> Decimal {
    let range = child.max - child.min;
    if range <= Decimal::ZERO {
        Decimal::ZERO
    } else {
        (grade - child.min) / range
    }
}

/// The children that take part, each with its share.
fn entries(rule: Rule, children: &[Child]) -> Vec<Entry> {
    children
        .iter()
        .filter_map(|child| match child.grade {
            Some(grade) => Some(Entry {
                share: share(child, grade),
                grade,
                child: *child,
            }),
            None if rule.exclude_empty => None,
            None => Some(Entry {
                share: Decimal::ZERO,
                grade: Decimal::ZERO,
                child: *child,
            }),
        })
        .collect()
}

/// Drops the lowest and keeps the highest of the regular children; the larger maximum goes first among
/// equal grades, and extra credit is never dropped.
fn dropped(rule: Rule, all: Vec<Entry>) -> Vec<Entry> {
    if !supports_dropping(rule.method) || (rule.drop_lowest == 0 && rule.keep_highest == 0) {
        return all;
    }
    let (extra, mut regular): (Vec<Entry>, Vec<Entry>) = all.into_iter().partition(|e| e.child.extra_credit);
    regular.sort_by(|a, b| a.share.cmp(&b.share).then(b.child.max.cmp(&a.child.max)));
    let drop = rule.drop_lowest.min(regular.len().saturating_sub(1));
    regular.drain(..drop);
    if rule.keep_highest > 0 && regular.len() > rule.keep_highest {
        let excess = regular.len() - rule.keep_highest;
        regular.drain(..excess);
    }
    regular.extend(extra);
    regular
}

/// The median of shares.
fn median(mut shares: Vec<Decimal>) -> Decimal {
    shares.sort();
    let middle = shares.len() / 2;
    if shares.len() % 2 == 1 {
        shares[middle]
    } else {
        (shares[middle - 1] + shares[middle]) / Decimal::TWO
    }
}

/// The most frequent share; the highest when several are equally frequent.
fn mode(mut shares: Vec<Decimal>) -> Decimal {
    shares.sort();
    let mut best = (shares[0], 0usize);
    let mut run = (shares[0], 0usize);
    for share in shares {
        run = if share == run.0 {
            (run.0, run.1 + 1)
        } else {
            (share, 1)
        };
        if run.1 >= best.1 {
            best = run;
        }
    }
    best.0
}

/// The weighted mean of shares, where `weight` gives the weight of each entry; extra credit adds to the
/// numerator only. None when the weights add up to nothing.
fn weighted(entries: &[Entry], weight: impl Fn(&Entry) -> Decimal) -> Option<Decimal> {
    let numerator: Decimal = entries.iter().map(|e| e.share * weight(e)).sum();
    let denominator: Decimal = entries
        .iter()
        .filter(|e| !e.child.extra_credit)
        .map(&weight)
        .sum();
    (denominator > Decimal::ZERO).then(|| numerator / denominator)
}

/// The total of a category for one person: what the children give, in the range of the total. None when no
/// child takes part.
#[must_use]
pub fn aggregate(rule: Rule, children: &[Child], total: (Decimal, Decimal)) -> Option<Decimal> {
    let taking = dropped(rule, entries(rule, children));
    if taking.is_empty() {
        return None;
    }
    let (low, high) = total;
    let shares = || taking.iter().map(|e| e.share).collect::<Vec<_>>();
    let value = match rule.method {
        Method::Natural => taking.iter().map(|e| e.grade).sum::<Decimal>(),
        Method::Mean => scaled(
            taking.iter().map(|e| e.share).sum::<Decimal>() / Decimal::from(taking.len()),
            total,
        ),
        Method::WeightedMean => scaled(weighted(&taking, |e| e.child.weight)?, total),
        Method::SimpleWeightedMean => scaled(weighted(&taking, |e| e.child.max - e.child.min)?, total),
        Method::Median => scaled(median(shares()), total),
        Method::Lowest => scaled(taking.iter().map(|e| e.share).min()?, total),
        Method::Highest => scaled(taking.iter().map(|e| e.share).max()?, total),
        Method::Mode => scaled(mode(shares()), total),
    };
    Some(value.clamp(low, high).round_dp(5))
}

/// A share of the range of the total, as a grade.
fn scaled(share: Decimal, (low, high): (Decimal, Decimal)) -> Decimal {
    low + share * (high - low)
}
