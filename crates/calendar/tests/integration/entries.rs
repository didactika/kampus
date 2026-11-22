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
// @package   kampus-calendar
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Entries and calendars: who sees and changes the entries of a scope and the personal ones, the window of
//! a calendar, and the dates of activities with the exceptions of the person who asks.

use std::collections::BTreeMap;

use kampus_activities::{ModuleFields, Target, Values};
use kampus_calendar::{EntryFields, ItemKind};
use kampus_common::{ErrorKind, Timestamp};
use kampus_courses::{Place, Visibility};
use kampus_module::{ActivityKind, ActivitySetting, ActivityTypeDeclaration, Module, ValueKind};
use serde_json::json;

use crate::world::{who, world};

/// A moment from its text.
fn at(text: &str) -> Timestamp {
    Timestamp::parse_rfc3339(text).unwrap()
}

/// The fields of an entry between two moments.
fn fields(title: &str, from: &str, to: &str) -> EntryFields {
    EntryFields {
        title: title.to_owned(),
        description: None,
        starts_at: at(from),
        ends_at: at(to),
        time_zone: None,
        all_day: false,
        recurrence: None,
        group: None,
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one scenario told from start to end")]
async fn a_teacher_adds_entries_to_a_course_and_participants_see_them_in_a_window() {
    let w = world().await;
    let scope = w.course.scope;
    let exam = w
        .calendar
        .create_entry(
            &w.admin,
            Some(scope),
            fields("Midterm", "2030-03-01T09:00:00Z", "2030-03-01T11:00:00Z"),
        )
        .await
        .unwrap();
    w.calendar
        .create_entry(
            &w.admin,
            Some(scope),
            fields("Review", "2030-04-10T09:00:00Z", "2030-04-10T10:00:00Z"),
        )
        .await
        .unwrap();

    let march = w
        .calendar
        .entries_for(
            &w.lea,
            Some(scope),
            at("2030-03-01T00:00:00Z"),
            at("2030-03-31T00:00:00Z"),
        )
        .await
        .unwrap();
    assert_eq!(march.len(), 1);
    assert_eq!(
        (march[0].kind, march[0].title.as_str()),
        (ItemKind::Entry, "Midterm")
    );
    let both = w
        .calendar
        .entries_for(
            &w.lea,
            Some(scope),
            at("2030-03-01T00:00:00Z"),
            at("2030-04-30T00:00:00Z"),
        )
        .await
        .unwrap();
    assert_eq!(both.len(), 2);

    let denied = w
        .calendar
        .create_entry(
            &w.lea,
            Some(scope),
            fields("Mine", "2030-03-02T09:00:00Z", "2030-03-02T10:00:00Z"),
        )
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    assert_eq!(
        w.calendar.delete_entry(&w.lea, exam.id).await.unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );

    let moved = w
        .calendar
        .update_entry(
            &w.admin,
            exam.id,
            fields("Midterm exam", "2030-03-02T09:00:00Z", "2030-03-02T11:00:00Z"),
            Some(exam.version),
        )
        .await
        .unwrap();
    assert_eq!(moved.title, "Midterm exam");
    let stale = w
        .calendar
        .update_entry(
            &w.admin,
            exam.id,
            fields("Again", "2030-03-02T09:00:00Z", "2030-03-02T11:00:00Z"),
            Some(exam.version),
        )
        .await
        .unwrap_err();
    assert_eq!(stale.kind(), ErrorKind::PreconditionFailed);
    w.calendar.delete_entry(&w.admin, exam.id).await.unwrap();
    assert_eq!(
        w.calendar.entry(&w.lea, exam.id).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );

    let backwards = w
        .calendar
        .create_entry(
            &w.admin,
            Some(scope),
            fields("Bad", "2030-03-02T11:00:00Z", "2030-03-02T09:00:00Z"),
        )
        .await
        .unwrap_err();
    assert_eq!(backwards.kind(), ErrorKind::Validation);
    let too_long = w
        .calendar
        .entries_for(
            &w.admin,
            Some(scope),
            at("2030-01-01T00:00:00Z"),
            at("2032-01-01T00:00:00Z"),
        )
        .await
        .unwrap_err();
    assert_eq!(too_long.code(), "calendar.window_invalid");
    w.drop().await;
}

#[tokio::test]
async fn personal_entries_are_seen_and_changed_only_by_their_owner() {
    let w = world().await;
    let mine = w
        .calendar
        .create_entry(
            &w.lea,
            None,
            fields("Study", "2030-05-01T18:00:00Z", "2030-05-01T20:00:00Z"),
        )
        .await
        .unwrap();
    assert_eq!(mine.owner, Some(who(&w.lea)));

    let own = w
        .calendar
        .entries_for(
            &w.lea,
            None,
            at("2030-05-01T00:00:00Z"),
            at("2030-05-02T00:00:00Z"),
        )
        .await
        .unwrap();
    assert_eq!(own.len(), 1);
    let others = w
        .calendar
        .entries_for(
            &w.max,
            None,
            at("2030-05-01T00:00:00Z"),
            at("2030-05-02T00:00:00Z"),
        )
        .await
        .unwrap();
    assert_eq!(others.len(), 0);
    assert_eq!(
        w.calendar.entry(&w.max, mine.id).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        w.calendar.entry(&w.admin, mine.id).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        w.calendar.delete_entry(&w.max, mine.id).await.unwrap_err().kind(),
        ErrorKind::NotFound,
        "others do not learn that it exists"
    );
    w.calendar.delete_entry(&w.lea, mine.id).await.unwrap();
    w.drop().await;
}

/// A module that provides an activity with dates and exceptions.
pub(crate) static QUIZ_MODULE: Module = Module {
    name: "quiz",
    steps: &[],
    permissions: &[],
    templates: &[],
    terms: &[],
    file_areas: &[],
    events: &[],
    jobs: &[],
    schedules: &[],
    routes: None,
    operations: &[],
    plugin_types: &[],
    activity_types: &[ActivityTypeDeclaration {
        code: "quiz.quiz",
        kind: ActivityKind::Activity,
        summary: "A quiz with dates",
        features: &["exceptions"],
        settings: &[
            ActivitySetting {
                key: "opens_at",
                kind: ValueKind::Text { max: 40, default: "" },
            },
            ActivitySetting {
                key: "due_at",
                kind: ValueKind::Text { max: 40, default: "" },
            },
        ],
    }],
    ..kampus_activities::MODULE
};

#[tokio::test]
async fn the_calendar_of_a_course_shows_the_dates_of_its_activities_as_they_apply_to_each_person() {
    let w = world().await;
    let section = w.courses.sections(&w.admin, w.course.id).await.unwrap().remove(0);
    let quiz = w
        .activities
        .create_module(
            &w.admin,
            section.id,
            "quiz.quiz",
            ModuleFields {
                title: "Quiz 1".to_owned(),
                on_course_page: true,
                visibility: Visibility::Shown,
                settings: BTreeMap::from([
                    ("opens_at".to_owned(), json!("2030-06-03T08:00:00Z")),
                    ("due_at".to_owned(), json!("2030-06-10T20:00:00Z")),
                ]),
            },
            Place::Last,
        )
        .await
        .unwrap();
    let from = at("2030-06-01T00:00:00Z");
    let to = at("2030-07-01T00:00:00Z");
    let shown = w
        .calendar
        .course_calendar(&w.lea, w.course.id, from, to)
        .await
        .unwrap();
    let kinds: Vec<_> = shown.iter().map(|i| (i.kind, i.starts_at)).collect();
    assert_eq!(
        kinds,
        [
            (ItemKind::Opens, at("2030-06-03T08:00:00Z")),
            (ItemKind::Due, at("2030-06-10T20:00:00Z"))
        ]
    );
    assert!(
        shown
            .iter()
            .all(|i| i.title == "Quiz 1" && i.course_module == Some(quiz.id))
    );

    w.activities
        .set_exception(
            &w.admin,
            quiz.id,
            Target::Person(who(&w.lea)),
            Values {
                due_at: Some(at("2030-06-15T20:00:00Z")),
                ..Values::default()
            },
            Some("extension"),
        )
        .await
        .unwrap();
    let lea = w
        .calendar
        .course_calendar(&w.lea, w.course.id, from, to)
        .await
        .unwrap();
    assert_eq!(
        lea[1].starts_at,
        at("2030-06-15T20:00:00Z"),
        "the exception of the person applies"
    );
    let max = w
        .calendar
        .course_calendar(&w.max, w.course.id, from, to)
        .await
        .unwrap();
    assert_eq!(max[1].starts_at, at("2030-06-10T20:00:00Z"), "and only to them");
    let june_first_week = w
        .calendar
        .course_calendar(&w.max, w.course.id, from, at("2030-06-05T00:00:00Z"))
        .await
        .unwrap();
    assert_eq!(june_first_week.len(), 1, "only what falls in the window");
    w.drop().await;
}

#[tokio::test]
async fn erasing_the_data_of_a_person_deletes_their_personal_entries_only() {
    let w = world().await;
    let course = w
        .calendar
        .create_entry(
            &w.admin,
            Some(w.course.scope),
            fields("Midterm", "2030-03-01T09:00:00Z", "2030-03-01T11:00:00Z"),
        )
        .await
        .unwrap();
    for (actor, title) in [(&w.lea, "Study"), (&w.max, "Sleep")] {
        w.calendar
            .create_entry(
                actor,
                None,
                fields(title, "2030-05-01T18:00:00Z", "2030-05-01T20:00:00Z"),
            )
            .await
            .unwrap();
    }
    let report = w.calendar.erase_personal_data(who(&w.lea)).await.unwrap();
    assert_eq!(report["deleted"]["entry"], 1);
    assert_eq!(
        w.calendar.entry(&w.admin, course.id).await.unwrap().title,
        "Midterm"
    );
    w.drop().await;
}
