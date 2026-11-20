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

//! The categories and grade records of a course: what a teacher defines, what an activity declares, who may
//! configure them, and what a learner sees of them.

use kampus_common::ErrorKind;
use kampus_courses::Place;
use kampus_records::{CategoryFields, Method, RecordFields, RecordKind};
use rust_decimal::Decimal;

use crate::world::world;

/// The fields of a category that adds up its records.
fn category(name: &str) -> CategoryFields {
    CategoryFields {
        name: name.to_owned(),
        method: Method::Natural,
        drop_lowest: 0,
        keep_highest: 0,
        exclude_empty: true,
    }
}

#[tokio::test]
async fn a_teacher_builds_the_tree_of_records_and_a_learner_only_sees_it() {
    let w = world().await;
    assert_eq!(
        w.records
            .tree(&w.admin, w.course.id)
            .await
            .unwrap()
            .categories
            .len(),
        0
    );

    let exams = w
        .records
        .define_category(&w.admin, w.course.id, None, category("Exams"), Place::Last)
        .await
        .unwrap();
    let midterm = w
        .records
        .define_record(
            &w.admin,
            w.course.id,
            Some(exams.id),
            RecordFields::new("Midterm", Decimal::from(50)),
        )
        .await
        .unwrap();
    assert_eq!(midterm.kind, RecordKind::Manual);
    assert_eq!(midterm.category, Some(exams.id));

    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    assert_eq!(
        tree.categories.len(),
        2,
        "the root of the course and the category"
    );
    assert!(tree.categories[0].parent.is_none());
    let kinds: Vec<RecordKind> = tree.records.iter().map(|r| r.kind).collect();
    assert!(kinds.contains(&RecordKind::CourseTotal));
    assert!(kinds.contains(&RecordKind::CategoryTotal));
    assert!(kinds.contains(&RecordKind::Manual));
    assert_eq!(
        w.records.tree(&w.lea, w.course.id).await.unwrap().records.len(),
        3
    );

    let denied = w
        .records
        .define_category(&w.lea, w.course.id, None, category("Mine"), Place::Last)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let denied = w
        .records
        .define_record(&w.lea, w.course.id, None, RecordFields::new("Mine", Decimal::TEN))
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn records_and_categories_change_on_their_version_and_are_deleted_only_when_allowed() {
    let w = world().await;
    let exams = w
        .records
        .define_category(&w.admin, w.course.id, None, category("Exams"), Place::Last)
        .await
        .unwrap();
    let record = w
        .records
        .define_record(
            &w.admin,
            w.course.id,
            Some(exams.id),
            RecordFields::new("Final", Decimal::from(100)),
        )
        .await
        .unwrap();

    let mut fields = RecordFields::new("Final exam", Decimal::from(80));
    fields.weight = Decimal::from(2);
    let updated = w
        .records
        .update_record(&w.admin, record.id, fields.clone(), Some(record.version))
        .await
        .unwrap();
    assert_eq!(
        (updated.title.as_str(), updated.max),
        ("Final exam", Decimal::from(80))
    );
    let stale = w
        .records
        .update_record(&w.admin, record.id, fields, Some(record.version))
        .await
        .unwrap_err();
    assert_eq!(stale.kind(), ErrorKind::PreconditionFailed);

    let mut bad = RecordFields::new("", Decimal::from(80));
    bad.min = Decimal::from(90);
    let invalid = w
        .records
        .update_record(&w.admin, record.id, bad, None)
        .await
        .unwrap_err();
    assert_eq!(invalid.kind(), ErrorKind::Validation);

    let not_empty = w.records.delete_category(&w.admin, exams.id).await.unwrap_err();
    assert_eq!(not_empty.code(), "records.category_not_empty");
    w.records.delete_record(&w.admin, record.id).await.unwrap();
    w.records.delete_category(&w.admin, exams.id).await.unwrap();
    let tree = w.records.tree(&w.admin, w.course.id).await.unwrap();
    assert_eq!(tree.categories.len(), 1, "only the root is left");
    assert_eq!(tree.records.len(), 1, "and the total of the course");

    let root = tree.categories[0].id;
    assert_eq!(
        w.records
            .delete_category(&w.admin, root)
            .await
            .unwrap_err()
            .code(),
        "records.category_is_root"
    );
    let total = tree.records[0].id;
    assert_eq!(
        w.records.delete_record(&w.admin, total).await.unwrap_err().code(),
        "records.record_not_deletable"
    );
    w.drop().await;
}

#[tokio::test]
async fn an_activity_declares_one_record_per_grade_and_a_teacher_keeps_what_they_set() {
    let w = world().await;
    let page = w.page("Lab report").await;
    let first = w
        .records
        .declare_activity_record(page.id, 0, RecordFields::new("Lab report", Decimal::from(10)))
        .await
        .unwrap();
    let mut tuned = RecordFields::new("Lab report", Decimal::from(10));
    tuned.weight = Decimal::from(3);
    w.records
        .update_record(&w.admin, first.id, tuned, None)
        .await
        .unwrap();

    let again = w
        .records
        .declare_activity_record(page.id, 0, RecordFields::new("Lab report v2", Decimal::from(20)))
        .await
        .unwrap();
    assert_eq!(again.id, first.id, "one record per course module and grade");
    assert_eq!(again.title, "Lab report v2");
    assert_eq!(again.max, Decimal::from(20));
    assert_eq!(again.weight, Decimal::from(3), "what the teacher set stays");
    let other = w
        .records
        .declare_activity_record(page.id, 1, RecordFields::new("Presentation", Decimal::from(5)))
        .await
        .unwrap();
    assert_ne!(other.id, first.id);

    let hidden_until = kampus_common::Timestamp::parse_rfc3339("2099-01-01T00:00:00Z").unwrap();
    let mut hidden = RecordFields::new("Secret", Decimal::from(5));
    hidden.hidden_until = Some(hidden_until);
    let secret = w
        .records
        .define_record(&w.admin, w.course.id, None, hidden)
        .await
        .unwrap();
    let seen_by_lea: Vec<_> = w
        .records
        .tree(&w.lea, w.course.id)
        .await
        .unwrap()
        .records
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert!(
        !seen_by_lea.contains(&secret.id),
        "a hidden record is not listed for learners"
    );
    assert!(w.records.record(&w.lea, secret.id).await.is_err());
    assert!(w.records.record(&w.admin, secret.id).await.is_ok());
    w.drop().await;
}
