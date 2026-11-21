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

//! Grades: idempotent upserts with their entries, the numbers a record accepts, overrides, locks, and what
//! each person sees of them.

use kampus_common::{ErrorKind, Timestamp};
use kampus_records::{GradeWrite, RecordFields, Source};
use rust_decimal::Decimal;

use crate::world::{World, who, world};

/// A numeric grade for a person.
fn write(_w: &World, who_is: &kampus_access::Actor, raw: i64) -> GradeWrite {
    GradeWrite {
        account: who(who_is),
        raw: Some(Decimal::from(raw)),
        text: None,
        feedback: None,
    }
}

/// A manual record graded out of a maximum.
async fn record(w: &World, max: i64) -> kampus_records::Record {
    w.records
        .define_record(
            &w.admin,
            w.course.id,
            None,
            RecordFields::new("Quiz", Decimal::from(max)),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn the_same_grade_twice_changes_nothing_and_every_change_leaves_an_entry() {
    let w = world().await;
    let quiz = record(&w, 10).await;
    let first = w
        .records
        .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, 7)], Source::Teacher)
        .await
        .unwrap();
    assert!(first[0].changed);
    let again = w
        .records
        .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, 7)], Source::Teacher)
        .await
        .unwrap();
    assert!(!again[0].changed, "the same grade changes nothing");
    assert_eq!(again[0].grade.version, first[0].grade.version);
    let better = w
        .records
        .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, 9)], Source::Teacher)
        .await
        .unwrap();
    assert!(better[0].changed);

    let detail = w
        .records
        .grade_detail(&w.admin, quiz.id, who(&w.lea))
        .await
        .unwrap();
    assert_eq!(detail.entries.len(), 2, "one entry per change");
    assert_eq!(detail.grade.raw, Some(Decimal::from(9)));
    assert_eq!(detail.grade.final_value, Some(Decimal::from(9)));

    let from_activity = w
        .records
        .submit_activity_grade(quiz.id, write(&w, &w.lea, 10))
        .await
        .unwrap();
    assert!(from_activity.changed);
    let entries = w
        .records
        .grade_detail(&w.admin, quiz.id, who(&w.lea))
        .await
        .unwrap()
        .entries;
    assert_eq!(entries.last().unwrap().source, Source::Activity);
    w.drop().await;
}

#[tokio::test]
async fn a_grade_must_fit_its_record_and_the_multiplier_and_offset_count() {
    let w = world().await;
    let quiz = record(&w, 10).await;
    for raw in [-1, 11] {
        let invalid = w
            .records
            .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, raw)], Source::Teacher)
            .await
            .unwrap_err();
        assert_eq!(invalid.kind(), ErrorKind::Validation, "{raw} is out of range");
    }
    let batch = w
        .records
        .set_grades(
            &w.admin,
            quiz.id,
            vec![write(&w, &w.lea, 5), write(&w, &w.max, 50)],
            Source::Teacher,
        )
        .await;
    assert!(batch.is_err());
    assert!(
        w.records
            .grade_detail(&w.admin, quiz.id, who(&w.lea))
            .await
            .is_err(),
        "a batch is all or nothing"
    );

    let mut fields = RecordFields::new("Quiz", Decimal::from(10));
    fields.multiplier = Decimal::from(2);
    fields.offset = Decimal::ONE;
    let scaled = w
        .records
        .update_record(&w.admin, quiz.id, fields, None)
        .await
        .unwrap();
    let set = w
        .records
        .set_grades(
            &w.admin,
            scaled.id,
            vec![write(&w, &w.lea, 3), write(&w, &w.max, 8)],
            Source::Teacher,
        )
        .await
        .unwrap();
    assert_eq!(
        set[0].grade.final_value,
        Some(Decimal::from(7)),
        "3 times 2 plus 1"
    );
    assert_eq!(
        set[1].grade.final_value,
        Some(Decimal::from(10)),
        "kept inside the maximum"
    );

    let stranger = w
        .records
        .set_grades(&w.lea, scaled.id, vec![write(&w, &w.lea, 10)], Source::Teacher)
        .await
        .unwrap_err();
    assert_eq!(stranger.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn an_override_wins_over_the_activity_until_it_is_removed() {
    let w = world().await;
    let quiz = record(&w, 10).await;
    w.records
        .submit_activity_grade(quiz.id, write(&w, &w.lea, 4))
        .await
        .unwrap();
    let overridden = w
        .records
        .override_grade(&w.admin, quiz.id, who(&w.lea), Decimal::from(8), Some("regraded"))
        .await
        .unwrap();
    assert!(overridden.grade.overridden);
    assert_eq!(overridden.grade.final_value, Some(Decimal::from(8)));
    let same = w
        .records
        .override_grade(&w.admin, quiz.id, who(&w.lea), Decimal::from(8), None)
        .await
        .unwrap();
    assert!(!same.changed);

    let later = w
        .records
        .submit_activity_grade(quiz.id, write(&w, &w.lea, 6))
        .await
        .unwrap();
    assert_eq!(later.grade.raw, Some(Decimal::from(6)));
    assert_eq!(
        later.grade.final_value,
        Some(Decimal::from(8)),
        "the override still counts"
    );

    let cleared = w
        .records
        .clear_override(&w.admin, quiz.id, who(&w.lea))
        .await
        .unwrap();
    assert!(!cleared.grade.overridden);
    assert_eq!(cleared.grade.final_value, Some(Decimal::from(6)));
    let detail = w
        .records
        .grade_detail(&w.admin, quiz.id, who(&w.lea))
        .await
        .unwrap();
    assert_eq!(detail.entries.len(), 4);
    assert_eq!(
        w.records
            .override_grade(&w.lea, quiz.id, who(&w.lea), Decimal::TEN, None)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}

#[tokio::test]
async fn a_locked_record_or_grade_does_not_change_until_it_is_unlocked() {
    let w = world().await;
    let quiz = record(&w, 10).await;
    w.records
        .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, 5)], Source::Teacher)
        .await
        .unwrap();

    w.records
        .set_grade_locked(&w.admin, quiz.id, who(&w.lea), true)
        .await
        .unwrap();
    let locked = w
        .records
        .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, 6)], Source::Teacher)
        .await
        .unwrap_err();
    assert_eq!(locked.code(), "records.locked");
    let same = w
        .records
        .set_grades(&w.admin, quiz.id, vec![write(&w, &w.lea, 5)], Source::Teacher)
        .await
        .unwrap();
    assert!(!same[0].changed, "the same grade is not a change, even locked");
    assert_eq!(
        w.records
            .override_grade(&w.admin, quiz.id, who(&w.lea), Decimal::TEN, None)
            .await
            .unwrap_err()
            .code(),
        "records.locked"
    );
    w.records
        .set_grade_locked(&w.admin, quiz.id, who(&w.lea), false)
        .await
        .unwrap();

    w.records
        .set_record_locked(&w.admin, quiz.id, true)
        .await
        .unwrap();
    assert_eq!(
        w.records
            .set_grades(&w.admin, quiz.id, vec![write(&w, &w.max, 3)], Source::Teacher)
            .await
            .unwrap_err()
            .code(),
        "records.locked"
    );
    assert_eq!(
        w.records
            .update_record(&w.admin, quiz.id, RecordFields::new("Quiz", Decimal::TEN), None)
            .await
            .unwrap_err()
            .code(),
        "records.locked"
    );
    w.records
        .set_record_locked(&w.admin, quiz.id, false)
        .await
        .unwrap();
    assert!(
        w.records
            .set_grades(&w.admin, quiz.id, vec![write(&w, &w.max, 3)], Source::Teacher)
            .await
            .unwrap()[0]
            .changed
    );
    w.drop().await;
}

#[tokio::test]
async fn everyone_sees_their_own_grades_and_hidden_ones_wait_for_their_date() {
    let w = world().await;
    let quiz = record(&w, 10).await;
    w.records
        .set_grades(
            &w.admin,
            quiz.id,
            vec![write(&w, &w.lea, 5), write(&w, &w.max, 9)],
            Source::Teacher,
        )
        .await
        .unwrap();

    let mine = w
        .records
        .grades_for(&w.lea, w.course.id, who(&w.lea))
        .await
        .unwrap();
    assert_eq!(mine.len(), 1);
    assert!(
        w.records
            .grades_for(&w.lea, w.course.id, who(&w.max))
            .await
            .unwrap()
            .is_empty(),
        "a learner does not see the grades of others"
    );
    assert_eq!(
        w.records
            .grade_detail(&w.lea, quiz.id, who(&w.max))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        w.records
            .grade_detail(&w.lea, quiz.id, who(&w.lea))
            .await
            .unwrap()
            .entries
            .len(),
        1
    );

    let later = Timestamp::parse_rfc3339("2099-01-01T00:00:00Z").unwrap();
    w.records
        .set_grade_hidden(&w.admin, quiz.id, who(&w.lea), Some(later))
        .await
        .unwrap();
    assert_eq!(
        w.records
            .grades_for(&w.lea, w.course.id, who(&w.lea))
            .await
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        w.records
            .grades_for(&w.admin, w.course.id, who(&w.lea))
            .await
            .unwrap()
            .len(),
        1,
        "who may see what is hidden still sees it"
    );
    w.records
        .set_grade_hidden(&w.admin, quiz.id, who(&w.lea), None)
        .await
        .unwrap();
    assert_eq!(
        w.records
            .grades_for(&w.lea, w.course.id, who(&w.lea))
            .await
            .unwrap()
            .len(),
        1
    );
    w.drop().await;
}

#[tokio::test]
async fn the_grader_report_pages_through_the_people_with_grades() {
    let w = world().await;
    let quiz = record(&w, 10).await;
    let other = record(&w, 20).await;
    w.records
        .set_grades(
            &w.admin,
            quiz.id,
            vec![write(&w, &w.lea, 5), write(&w, &w.max, 9)],
            Source::Teacher,
        )
        .await
        .unwrap();
    w.records
        .set_grades(&w.admin, other.id, vec![write(&w, &w.lea, 15)], Source::Teacher)
        .await
        .unwrap();

    let first = w
        .records
        .grader_report(&w.admin, w.course.id, 1, None, false)
        .await
        .unwrap();
    assert_eq!(first.rows.len(), 1);
    assert!(first.next.is_some());
    let second = w
        .records
        .grader_report(&w.admin, w.course.id, 1, first.next, false)
        .await
        .unwrap();
    assert_eq!(second.rows.len(), 1);
    assert!(second.next.is_none());
    let total: usize = first
        .rows
        .iter()
        .chain(&second.rows)
        .map(|r| r.grades.len())
        .sum();
    assert_eq!(total, 3);
    assert_ne!(first.rows[0].account, second.rows[0].account);
    assert_eq!(
        w.records
            .grader_report(&w.lea, w.course.id, 10, None, false)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}
