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

//! Scales and letter sets: defined in a scope and available below it, versioned, used by records that grade
//! with items, and turned into letters for display.

use kampus_common::ErrorKind;
use kampus_records::{Band, GradeType, GradeWrite, RecordFields, Source};
use rust_decimal::Decimal;

use crate::world::{who, world};

/// The items of a scale.
fn items(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

/// A band from a percentage and a letter.
fn band(percent: i64, letter: &str) -> Band {
    Band {
        min_percent: Decimal::from(percent),
        letter: letter.to_owned(),
    }
}

#[tokio::test]
async fn a_scale_has_versions_and_the_nearest_definition_wins() {
    let w = world().await;
    let campus = w.course.campus;
    let first = w
        .records
        .define_scale(
            &w.admin,
            campus,
            "quality",
            items(&["Poor", "Fair", "Good", "Excellent"]),
        )
        .await
        .unwrap();
    assert_eq!(first.version, 1);
    let second = w
        .records
        .define_scale(&w.admin, campus, "quality", items(&["Poor", "Good", "Excellent"]))
        .await
        .unwrap();
    assert_eq!(second.version, 2);
    let available = w.records.scales_of_course(&w.lea, w.course.id).await.unwrap();
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].id, second.id, "only the latest version is offered");

    let own = w
        .records
        .define_scale(&w.admin, w.course.scope, "quality", items(&["No", "Yes"]))
        .await
        .unwrap();
    assert_eq!(own.version, 1, "versions count per scope");
    let available = w.records.scales_in(&w.admin, w.course.scope).await.unwrap();
    assert_eq!(available.len(), 1);
    assert_eq!(
        available[0].id, own.id,
        "the course definition hides the campus one"
    );
    assert_eq!(
        w.records.scales_in(&w.admin, campus).await.unwrap()[0].id,
        second.id
    );

    for bad in [items(&["Only"]), items(&["a", "a"]), items(&["a", ""])] {
        let invalid = w
            .records
            .define_scale(&w.admin, campus, "bad", bad)
            .await
            .unwrap_err();
        assert_eq!(invalid.code(), "records.scale_invalid");
    }
    let wrong_code = w
        .records
        .define_scale(&w.admin, campus, "Not Valid", items(&["a", "b"]))
        .await
        .unwrap_err();
    assert_eq!(wrong_code.kind(), ErrorKind::Validation);
    let denied = w
        .records
        .define_scale(&w.lea, campus, "mine", items(&["a", "b"]))
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn a_record_graded_with_a_scale_takes_its_items_as_range_and_keeps_its_version() {
    let w = world().await;
    let scale = w
        .records
        .define_scale(
            &w.admin,
            w.course.campus,
            "quality",
            items(&["Poor", "Fair", "Good", "Excellent"]),
        )
        .await
        .unwrap();
    let mut fields = RecordFields::new("Presentation", Decimal::from(100));
    fields.grade_type = GradeType::Scale;
    fields.scale = Some(scale.id);
    let record = w
        .records
        .define_record(&w.admin, w.course.id, None, fields)
        .await
        .unwrap();
    assert_eq!((record.min, record.max), (Decimal::ONE, Decimal::from(4)));
    assert_eq!(record.scale, Some(scale.id));

    let write = |raw: &str| GradeWrite {
        account: who(&w.lea),
        raw: Some(raw.parse().unwrap()),
        text: None,
        feedback: None,
    };
    let set = w
        .records
        .set_grades(&w.admin, record.id, vec![write("3")], Source::Teacher)
        .await
        .unwrap();
    assert_eq!(set[0].grade.final_value, Some(Decimal::from(3)));
    for raw in ["2.5", "5", "0"] {
        let invalid = w
            .records
            .set_grades(&w.admin, record.id, vec![write(raw)], Source::Teacher)
            .await
            .unwrap_err();
        assert_eq!(invalid.kind(), ErrorKind::Validation, "{raw} is not an item");
    }

    w.records
        .define_scale(&w.admin, w.course.campus, "quality", items(&["No", "Yes"]))
        .await
        .unwrap();
    let again = w.records.record(&w.admin, record.id).await.unwrap();
    assert_eq!(
        (again.scale, again.max),
        (Some(scale.id), Decimal::from(4)),
        "a new version changes nothing"
    );

    let mut plain = RecordFields::new("Plain", Decimal::TEN);
    plain.scale = Some(scale.id);
    assert_eq!(
        w.records
            .define_record(&w.admin, w.course.id, None, plain)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Validation,
        "only a scale record has a scale"
    );
    let mut missing = RecordFields::new("Missing", Decimal::TEN);
    missing.grade_type = GradeType::Scale;
    assert_eq!(
        w.records
            .define_record(&w.admin, w.course.id, None, missing)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Validation
    );
    w.drop().await;
}

#[tokio::test]
async fn letters_follow_the_nearest_set_and_the_percentage_picks_the_band() {
    let w = world().await;
    assert_eq!(
        w.records
            .letter_for_course(w.course.id, Decimal::from(95))
            .await
            .unwrap(),
        None
    );
    let campus = w
        .records
        .define_letters(
            &w.admin,
            w.course.campus,
            vec![band(0, "F"), band(60, "D"), band(90, "A"), band(75, "B")],
        )
        .await
        .unwrap();
    assert_eq!(campus.bands[0].letter, "A", "from the highest percentage down");
    assert_eq!(
        w.records
            .letter_for_course(w.course.id, Decimal::from(95))
            .await
            .unwrap()
            .as_deref(),
        Some("A")
    );
    assert_eq!(
        w.records
            .letter_for_course(w.course.id, Decimal::from(80))
            .await
            .unwrap()
            .as_deref(),
        Some("B")
    );

    w.records
        .define_letters(&w.admin, w.course.scope, vec![band(0, "Fail"), band(50, "Pass")])
        .await
        .unwrap();
    assert_eq!(
        w.records
            .letter_for_course(w.course.id, Decimal::from(95))
            .await
            .unwrap()
            .as_deref(),
        Some("Pass"),
        "the course set hides the campus one"
    );
    let shown = w
        .records
        .letters_of_course(&w.lea, w.course.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(shown.scope, w.course.scope);
    let again = w
        .records
        .define_letters(&w.admin, w.course.scope, vec![band(0, "N"), band(70, "Y")])
        .await
        .unwrap();
    assert_eq!(again.version, 2);

    for bad in [
        vec![],
        vec![band(10, "A")],
        vec![band(0, "A"), band(0, "B")],
        vec![band(0, "A"), band(101, "B")],
    ] {
        let invalid = w
            .records
            .define_letters(&w.admin, w.course.scope, bad)
            .await
            .unwrap_err();
        assert_eq!(invalid.code(), "records.letters_invalid");
    }
    assert_eq!(
        w.records
            .define_letters(&w.lea, w.course.scope, vec![band(0, "A")])
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}
