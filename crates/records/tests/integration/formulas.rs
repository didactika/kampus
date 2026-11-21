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

//! Formulas: a manual record or a total computes its grades from other records when the queue runs, within
//! its range; what it reads is kept, a cycle or an unknown record is refused, a computed record takes no
//! grades by hand, and a record a formula reads cannot be deleted.

use kampus_access::Actor;
use kampus_common::ErrorKind;
use kampus_courses::Place;
use kampus_records::{CategoryFields, GradeWrite, Method, Record, RecordFields, RecordId, Source};
use rust_decimal::Decimal;

use crate::world::{World, who, world};

/// A manual record of the course with a maximum.
async fn item(w: &World, title: &str, max: i64) -> Record {
    w.records
        .define_record(
            &w.admin,
            w.course.id,
            None,
            RecordFields::new(title, Decimal::from(max)),
        )
        .await
        .unwrap()
}

/// Gives a person a grade.
async fn grade(w: &World, record: RecordId, person: &Actor, raw: i64) {
    w.records
        .set_grades(
            &w.admin,
            record,
            vec![GradeWrite {
                account: who(person),
                raw: Some(Decimal::from(raw)),
                text: None,
                feedback: None,
            }],
            Source::Teacher,
        )
        .await
        .unwrap();
}

/// Runs the queue until nothing is left.
async fn run_queue(w: &World) {
    while w.records.recalculate().await.unwrap().people > 0 {}
}

/// What a person has on a record.
async fn final_of(w: &World, person: &Actor, record: RecordId) -> Option<Decimal> {
    w.records
        .grades_for(&w.admin, w.course.id, who(person))
        .await
        .unwrap()
        .into_iter()
        .find(|g| g.record == record)
        .and_then(|g| g.final_value)
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one scenario told from start to end")]
async fn a_formula_computes_from_other_records_within_its_range() {
    let w = world().await;
    let quiz = item(&w, "Quiz", 10).await;
    let essay = item(&w, "Essay", 10).await;
    let mark = item(&w, "Mark", 10).await;
    let bonus = item(&w, "Bonus", 10).await;
    let set = w
        .records
        .set_formula(
            &w.admin,
            mark.id,
            &format!("round(([[{}]] * 2 + [[{}]]) / 3, 1)", quiz.id, essay.id),
        )
        .await
        .unwrap();
    assert_eq!(set.depends_on.len(), 2);
    w.records
        .set_formula(&w.admin, bonus.id, &format!("[[{}]] + 5", mark.id))
        .await
        .unwrap();

    grade(&w, quiz.id, &w.lea, 8).await;
    grade(&w, essay.id, &w.lea, 6).await;
    grade(&w, quiz.id, &w.max, 9).await;
    run_queue(&w).await;
    assert_eq!(final_of(&w, &w.lea, mark.id).await, Some(Decimal::new(73, 1)));
    assert_eq!(
        final_of(&w, &w.lea, bonus.id).await,
        Some(Decimal::TEN),
        "a formula reading another one comes after it, and stays within the range"
    );
    assert_eq!(
        final_of(&w, &w.max, mark.id).await,
        None,
        "a record without a grade gives no result"
    );

    let by_hand = w
        .records
        .set_grades(
            &w.admin,
            mark.id,
            vec![GradeWrite {
                account: who(&w.lea),
                raw: Some(Decimal::ONE),
                text: None,
                feedback: None,
            }],
            Source::Teacher,
        )
        .await
        .unwrap_err();
    assert_eq!(by_hand.code(), "records.record_derived");
    assert_eq!(
        w.records
            .delete_record(&w.admin, quiz.id)
            .await
            .unwrap_err()
            .code(),
        "records.record_used_by_formula"
    );

    w.records.remove_formula(&w.admin, bonus.id).await.unwrap();
    assert!(w.records.formula(&w.admin, bonus.id).await.unwrap().is_none());
    w.records.delete_record(&w.admin, bonus.id).await.unwrap();
    w.drop().await;
}

#[tokio::test]
async fn cycles_unknown_records_and_wrong_expressions_are_refused() {
    let w = world().await;
    let first = item(&w, "First", 10).await;
    let second = item(&w, "Second", 10).await;
    w.records
        .set_formula(&w.admin, first.id, &format!("[[{}]]", second.id))
        .await
        .unwrap();
    let cycle = w
        .records
        .set_formula(&w.admin, second.id, &format!("[[{}]] + 1", first.id))
        .await
        .unwrap_err();
    assert_eq!(cycle.code(), "records.formula_cycle");
    let itself = w
        .records
        .set_formula(&w.admin, second.id, &format!("[[{}]]", second.id))
        .await
        .unwrap_err();
    assert_eq!(itself.code(), "records.formula_cycle");
    let unknown = w
        .records
        .set_formula(&w.admin, second.id, &format!("[[{}]]", uuid::Uuid::now_v7()))
        .await
        .unwrap_err();
    assert_eq!(unknown.code(), "records.formula_reference_unknown");
    let wrong = w
        .records
        .set_formula(&w.admin, second.id, "1 +")
        .await
        .unwrap_err();
    assert_eq!(wrong.code(), "records.formula_invalid");
    assert_eq!(
        w.records
            .set_formula(&w.lea, second.id, "1")
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}

#[tokio::test]
async fn a_formula_on_the_total_of_a_category_replaces_its_aggregation_and_respects_the_tree() {
    let w = world().await;
    let exams = w
        .records
        .define_category(
            &w.admin,
            w.course.id,
            None,
            CategoryFields {
                name: "Exams".to_owned(),
                method: Method::Mean,
                drop_lowest: 0,
                keep_highest: 0,
                exclude_empty: true,
            },
            Place::Last,
        )
        .await
        .unwrap();
    let midterm = w
        .records
        .define_record(
            &w.admin,
            w.course.id,
            Some(exams.id),
            RecordFields::new("Midterm", Decimal::from(100)),
        )
        .await
        .unwrap();
    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    let total = tree
        .records
        .iter()
        .find(|r| r.total_of == Some(exams.id))
        .unwrap()
        .id;
    let outside = item(&w, "Outside", 10).await;
    w.records
        .set_formula(&w.admin, total, &format!("[[{}]] * 0.5", midterm.id))
        .await
        .unwrap();
    assert_eq!(
        w.records
            .set_formula(&w.admin, midterm.id, &format!("[[{total}]]"))
            .await
            .unwrap_err()
            .code(),
        "records.formula_cycle",
        "a child read by the total of its category cannot read that total"
    );
    w.records
        .set_formula(&w.admin, outside.id, &format!("[[{total}]] / 10"))
        .await
        .unwrap();
    grade(&w, midterm.id, &w.lea, 80).await;
    run_queue(&w).await;
    assert_eq!(final_of(&w, &w.lea, total).await, Some(Decimal::from(40)));
    assert_eq!(final_of(&w, &w.lea, outside.id).await, Some(Decimal::from(4)));
    w.drop().await;
}
