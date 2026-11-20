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

//! Totals: categories and the course add up the grades of their children when the recalculation runs, an
//! override of a total stays, the range of a natural total follows its children, excluded grades are left out,
//! and a total that adds up something hidden is hidden too.

use kampus_access::Actor;
use kampus_common::Timestamp;
use kampus_courses::Place;
use kampus_records::{
    CategoryFields, CategoryId, GradeWrite, Method, Record, RecordFields, RecordId, Source,
};
use rust_decimal::Decimal;

use crate::world::{World, who, world};

/// A category aggregating with a method.
fn category(name: &str, method: Method) -> CategoryFields {
    CategoryFields {
        name: name.to_owned(),
        method,
        drop_lowest: 0,
        keep_highest: 0,
        exclude_empty: true,
    }
}

/// A manual record with a maximum and a weight.
async fn item(w: &World, category: CategoryId, title: &str, max: i64, weight: i64) -> Record {
    let mut fields = RecordFields::new(title, Decimal::from(max));
    fields.weight = Decimal::from(weight);
    w.records
        .define_record(&w.admin, w.course.id, Some(category), fields)
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
async fn run_queue(w: &World) -> u64 {
    let mut totals = 0;
    loop {
        let done = w.records.recalculate().await.unwrap();
        if done.people == 0 {
            return totals;
        }
        totals += done.totals_changed;
    }
}

/// What a person has on a record, as the actor sees it.
async fn final_of(w: &World, viewer: &Actor, person: &Actor, record: RecordId) -> Option<Decimal> {
    w.records
        .grades_for(viewer, w.course.id, who(person))
        .await
        .unwrap()
        .into_iter()
        .find(|g| g.record == record)
        .and_then(|g| g.final_value)
}

/// The total records of a category and of the course.
async fn totals(w: &World, category: CategoryId) -> (RecordId, RecordId) {
    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    let of = |wanted: CategoryId| {
        tree.records
            .iter()
            .find(|r| r.total_of == Some(wanted))
            .unwrap()
            .id
    };
    let root = tree.categories.iter().find(|c| c.parent.is_none()).unwrap().id;
    (of(category), of(root))
}

/// A decimal from a text.
fn dec(text: &str) -> Decimal {
    text.parse().unwrap()
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one scenario told from start to end")]
async fn totals_follow_the_grades_when_the_queue_runs_and_an_override_of_a_total_stays() {
    let w = world().await;
    let exams = w
        .records
        .define_category(
            &w.admin,
            w.course.id,
            None,
            category("Exams", Method::WeightedMean),
            Place::Last,
        )
        .await
        .unwrap();
    let first = item(&w, exams.id, "First", 10, 3).await;
    let second = item(&w, exams.id, "Second", 20, 1).await;
    let (exams_total, course_total) = totals(&w, exams.id).await;

    grade(&w, first.id, &w.lea, 8).await;
    grade(&w, second.id, &w.lea, 15).await;
    grade(&w, first.id, &w.max, 5).await;
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, exams_total).await,
        None,
        "nothing is derived before the job runs"
    );
    assert!(run_queue(&w).await >= 4);

    assert_eq!(
        final_of(&w, &w.admin, &w.lea, exams_total).await,
        Some(dec("78.75"))
    );
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, course_total).await,
        Some(dec("78.75"))
    );
    assert_eq!(final_of(&w, &w.admin, &w.max, exams_total).await, Some(dec("50")));
    assert_eq!(run_queue(&w).await, 0, "running again changes nothing");

    grade(&w, first.id, &w.lea, 10).await;
    run_queue(&w).await;
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, exams_total).await,
        Some(dec("93.75"))
    );
    assert_eq!(
        final_of(&w, &w.lea, &w.lea, exams_total).await,
        Some(dec("93.75")),
        "a learner sees their totals"
    );

    w.records
        .override_grade(&w.admin, exams_total, who(&w.lea), Decimal::from(60), None)
        .await
        .unwrap();
    run_queue(&w).await;
    assert_eq!(final_of(&w, &w.admin, &w.lea, exams_total).await, Some(dec("60")));
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, course_total).await,
        Some(dec("60")),
        "the course adds the override"
    );
    grade(&w, first.id, &w.lea, 2).await;
    run_queue(&w).await;
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, exams_total).await,
        Some(dec("60")),
        "the override still counts"
    );
    w.records
        .clear_override(&w.admin, exams_total, who(&w.lea))
        .await
        .unwrap();
    run_queue(&w).await;
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, exams_total).await,
        Some(dec("33.75"))
    );

    let derived = w
        .records
        .set_grades(
            &w.admin,
            exams_total,
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
    assert_eq!(
        derived.code(),
        "records.record_derived",
        "a total is not graded by hand"
    );
    w.drop().await;
}

#[tokio::test]
async fn the_total_of_a_natural_category_follows_the_maximums_and_leaves_out_excluded_grades() {
    let w = world().await;
    let labs = w
        .records
        .define_category(
            &w.admin,
            w.course.id,
            None,
            category("Labs", Method::Natural),
            Place::Last,
        )
        .await
        .unwrap();
    let first = item(&w, labs.id, "Lab 1", 10, 1).await;
    let second = item(&w, labs.id, "Lab 2", 20, 1).await;
    let (labs_total, _) = totals(&w, labs.id).await;
    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    let total = tree.records.iter().find(|r| r.id == labs_total).unwrap();
    assert_eq!(
        (total.min, total.max),
        (Decimal::ZERO, Decimal::from(30)),
        "the maximums add up"
    );

    grade(&w, first.id, &w.lea, 4).await;
    grade(&w, second.id, &w.lea, 12).await;
    run_queue(&w).await;
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, labs_total).await,
        Some(Decimal::from(16))
    );

    w.records
        .set_grade_excluded(&w.admin, first.id, who(&w.lea), true)
        .await
        .unwrap();
    run_queue(&w).await;
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, labs_total).await,
        Some(Decimal::from(12))
    );

    let third = item(&w, labs.id, "Lab 3", 5, 1).await;
    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    assert_eq!(
        tree.records.iter().find(|r| r.id == labs_total).unwrap().max,
        Decimal::from(35)
    );
    w.records.delete_record(&w.admin, third.id).await.unwrap();
    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    assert_eq!(
        tree.records.iter().find(|r| r.id == labs_total).unwrap().max,
        Decimal::from(30)
    );
    w.drop().await;
}

#[tokio::test]
async fn a_total_that_adds_up_something_hidden_is_hidden_with_it() {
    let w = world().await;
    let exams = w
        .records
        .define_category(
            &w.admin,
            w.course.id,
            None,
            category("Exams", Method::Mean),
            Place::Last,
        )
        .await
        .unwrap();
    let first = item(&w, exams.id, "First", 10, 1).await;
    let second = item(&w, exams.id, "Second", 10, 1).await;
    let (exams_total, course_total) = totals(&w, exams.id).await;
    grade(&w, first.id, &w.lea, 8).await;
    grade(&w, second.id, &w.lea, 6).await;
    run_queue(&w).await;
    assert_eq!(final_of(&w, &w.lea, &w.lea, exams_total).await, Some(dec("70")));

    let later = Timestamp::parse_rfc3339("2099-01-01T00:00:00Z").unwrap();
    w.records
        .set_grade_hidden(&w.admin, second.id, who(&w.lea), Some(later))
        .await
        .unwrap();
    assert_eq!(final_of(&w, &w.lea, &w.lea, second.id).await, None);
    assert_eq!(
        final_of(&w, &w.lea, &w.lea, exams_total).await,
        None,
        "the total would reveal it"
    );
    assert_eq!(
        final_of(&w, &w.lea, &w.lea, course_total).await,
        None,
        "and so would the course total"
    );
    assert_eq!(
        final_of(&w, &w.lea, &w.lea, first.id).await,
        Some(Decimal::from(8))
    );
    assert_eq!(
        final_of(&w, &w.admin, &w.lea, exams_total).await,
        Some(dec("70")),
        "graders see everything"
    );
    w.drop().await;
}
