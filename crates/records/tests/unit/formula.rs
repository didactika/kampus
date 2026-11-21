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

//! Formulas: parsing, the records they read, exact results and what gives no result.

use std::collections::BTreeMap;

use kampus_records::formula::{Expr, parse};
use rust_decimal::Decimal;

/// Two records, with the grades a person has on them.
fn grades() -> (uuid::Uuid, uuid::Uuid, BTreeMap<uuid::Uuid, Decimal>) {
    let quiz = uuid::Uuid::now_v7();
    let essay = uuid::Uuid::now_v7();
    let held = BTreeMap::from([(quiz, Decimal::from(8)), (essay, Decimal::new(65, 1))]);
    (quiz, essay, held)
}

/// The value of an expression with those grades.
fn value(text: &str, held: &BTreeMap<uuid::Uuid, Decimal>) -> Option<Decimal> {
    parse(text).unwrap().evaluate(&|id| held.get(&id).copied())
}

#[test]
fn operators_follow_their_precedence_and_results_are_exact() {
    let none = BTreeMap::new();
    assert_eq!(value("1 + 2 * 3", &none), Some(Decimal::from(7)));
    assert_eq!(value("(1 + 2) * 3", &none), Some(Decimal::from(9)));
    assert_eq!(value("-2 - -3", &none), Some(Decimal::ONE));
    assert_eq!(
        value("0.1 + 0.2", &none),
        Some(Decimal::new(3, 1)),
        "no binary rounding"
    );
    assert_eq!(value("10 / 4", &none), Some(Decimal::new(25, 1)));
}

#[test]
fn references_read_the_grades_of_other_records() {
    let (quiz, essay, held) = grades();
    let text = format!("([[{quiz}]] + [[{essay}]]) / 2");
    assert_eq!(value(&text, &held), Some(Decimal::new(725, 2)));
    assert_eq!(
        parse(&text)
            .unwrap()
            .references()
            .into_iter()
            .collect::<Vec<_>>()
            .len(),
        2
    );
    let missing = format!("[[{quiz}]] + [[{}]]", uuid::Uuid::now_v7());
    assert_eq!(
        value(&missing, &held),
        None,
        "a record without a grade gives no result"
    );
}

#[test]
fn functions_compute_and_check_their_arguments() {
    let (quiz, essay, held) = grades();
    assert_eq!(
        value(&format!("max([[{quiz}]], [[{essay}]])"), &held),
        Some(Decimal::from(8))
    );
    assert_eq!(
        value(&format!("min([[{quiz}]], [[{essay}]])"), &held),
        Some(Decimal::new(65, 1))
    );
    assert_eq!(value("sum(1, 2, 3)", &held), Some(Decimal::from(6)));
    assert_eq!(value("average(1, 2)", &held), Some(Decimal::new(15, 1)));
    assert_eq!(value("round(2.345, 2)", &held), Some(Decimal::new(235, 2)));
    assert_eq!(value("round(2.5)", &held), Some(Decimal::from(3)));
    assert_eq!(
        value("floor(2.7) + ceil(2.1) + abs(-1)", &held),
        Some(Decimal::from(6))
    );
    assert_eq!(parse("abs(1, 2)").unwrap_err().rule, "arguments");
    assert_eq!(parse("sqrt(4)").unwrap_err().rule, "unknown_function");
}

#[test]
fn wrong_expressions_say_what_and_where_and_nothing_divides_by_zero() {
    assert_eq!(parse("1 +").unwrap_err().rule, "unexpected");
    assert_eq!(parse("(1 + 2").unwrap_err().rule, "unexpected");
    assert_eq!(parse("1 2").unwrap_err().at, 2);
    assert_eq!(parse("[[not an id]]").unwrap_err().rule, "reference");
    assert_eq!(parse("2 % 3").unwrap_err().rule, "unexpected");
    assert_eq!(parse(&"(".repeat(40)).unwrap_err().rule, "too_deep");
    assert_eq!(parse(&"1+".repeat(1500)).unwrap_err().rule, "too_long");
    assert_eq!(value("1 / 0", &BTreeMap::new()), None);
    assert!(matches!(parse("2").unwrap(), Expr::Number(_)));
}
