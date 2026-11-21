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

//! Recurring, all-day and group entries: a weekly class stored once and shown in every week of a window,
//! one occurrence cancelled or moved, whole days that must start at midnight, and entries only a group sees.

use kampus_calendar::{EntryFields, OccurrenceChange};
use kampus_common::{ErrorKind, Timestamp};
use kampus_courses::{GroupFields, GroupVisibility};

use crate::world::{who, world};

/// A moment from its text.
fn at(text: &str) -> Timestamp {
    Timestamp::parse_rfc3339(text).unwrap()
}

/// The fields of a weekly class on Mondays at nine in Madrid.
fn weekly() -> EntryFields {
    EntryFields {
        title: "Lecture".to_owned(),
        description: None,
        starts_at: at("2030-03-18T08:00:00Z"),
        ends_at: at("2030-03-18T09:00:00Z"),
        time_zone: Some("Europe/Madrid".to_owned()),
        all_day: false,
        recurrence: Some("FREQ=WEEKLY;BYDAY=MO;COUNT=4".to_owned()),
        group: None,
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one scenario told from start to end")]
async fn a_weekly_entry_is_stored_once_and_shown_in_each_week_with_its_changes() {
    let w = world().await;
    let scope = w.course.scope;
    let lecture = w
        .calendar
        .create_entry(&w.admin, Some(scope), weekly())
        .await
        .unwrap();
    let window = |from: &str, to: &str| (at(from), at(to));
    let (from, to) = window("2030-03-01T00:00:00Z", "2030-05-01T00:00:00Z");
    let starts =
        |items: &[kampus_calendar::Item]| items.iter().map(|i| i.starts_at.to_string()).collect::<Vec<_>>();
    let all = w
        .calendar
        .entries_for(&w.lea, Some(scope), from, to)
        .await
        .unwrap();
    assert_eq!(
        starts(&all),
        [
            "2030-03-18T08:00:00Z",
            "2030-03-25T08:00:00Z",
            "2030-04-01T07:00:00Z",
            "2030-04-08T07:00:00Z"
        ],
        "nine in Madrid before and after the change of time"
    );

    w.calendar
        .change_occurrence(
            &w.admin,
            lecture.id,
            at("2030-03-25T08:00:00Z"),
            OccurrenceChange {
                cancelled: true,
                title: None,
                starts_at: None,
                ends_at: None,
            },
        )
        .await
        .unwrap();
    w.calendar
        .change_occurrence(
            &w.admin,
            lecture.id,
            at("2030-04-01T07:00:00Z"),
            OccurrenceChange {
                cancelled: false,
                title: Some("Lecture in room 2".to_owned()),
                starts_at: Some(at("2030-04-02T07:00:00Z")),
                ends_at: Some(at("2030-04-02T08:00:00Z")),
            },
        )
        .await
        .unwrap();
    let changed = w
        .calendar
        .entries_for(&w.lea, Some(scope), from, to)
        .await
        .unwrap();
    assert_eq!(
        starts(&changed),
        [
            "2030-03-18T08:00:00Z",
            "2030-04-02T07:00:00Z",
            "2030-04-08T07:00:00Z"
        ]
    );
    assert_eq!(changed[1].title, "Lecture in room 2");
    assert_eq!(changed[1].occurrence, Some(at("2030-04-01T07:00:00Z")));

    let wrong = w
        .calendar
        .change_occurrence(
            &w.admin,
            lecture.id,
            at("2030-03-26T08:00:00Z"),
            OccurrenceChange {
                cancelled: true,
                title: None,
                starts_at: None,
                ends_at: None,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        wrong.code(),
        "calendar.occurrence_invalid",
        "not one of its occurrences"
    );
    w.calendar
        .restore_occurrence(&w.admin, lecture.id, at("2030-03-25T08:00:00Z"))
        .await
        .unwrap();
    assert_eq!(
        w.calendar
            .entries_for(&w.lea, Some(scope), from, to)
            .await
            .unwrap()
            .len(),
        4
    );
    w.drop().await;
}

#[tokio::test]
async fn rules_time_zones_and_all_day_entries_are_checked() {
    let w = world().await;
    let scope = w.course.scope;
    for (fields, field) in [
        (
            EntryFields {
                recurrence: Some("FREQ=HOURLY".to_owned()),
                ..weekly()
            },
            "recurrence",
        ),
        (
            EntryFields {
                time_zone: Some("Mars/Olympus".to_owned()),
                ..weekly()
            },
            "time_zone",
        ),
        (
            EntryFields {
                all_day: true,
                ..weekly()
            },
            "all_day",
        ),
    ] {
        let refused = w
            .calendar
            .create_entry(&w.admin, Some(scope), fields)
            .await
            .unwrap_err();
        assert_eq!(refused.kind(), ErrorKind::Validation);
        assert!(format!("{refused:?}").contains(field), "{refused:?}");
    }
    let holiday = w
        .calendar
        .create_entry(
            &w.admin,
            Some(scope),
            EntryFields {
                title: "Holiday".to_owned(),
                starts_at: at("2030-03-31T22:00:00Z"),
                ends_at: at("2030-04-01T22:00:00Z"),
                all_day: true,
                recurrence: None,
                ..weekly()
            },
        )
        .await
        .unwrap();
    assert!(
        holiday.all_day,
        "the first of April from midnight to midnight in Madrid"
    );
    w.drop().await;
}

#[tokio::test]
async fn an_entry_for_a_group_is_seen_by_its_members_and_who_manages_the_calendar() {
    let w = world().await;
    let scope = w.course.scope;
    let group = w
        .courses
        .create_group(
            &w.admin,
            w.course.id,
            GroupFields {
                name: "Lab A".to_owned(),
                code: None,
                description: None,
                section: None,
                visibility: GroupVisibility::Everyone,
            },
        )
        .await
        .unwrap();
    w.courses
        .add_member(&w.admin, group.id, who(&w.lea))
        .await
        .unwrap();
    let lab = w
        .calendar
        .create_entry(
            &w.admin,
            Some(scope),
            EntryFields {
                title: "Lab session".to_owned(),
                recurrence: None,
                group: Some(group.id),
                ..weekly()
            },
        )
        .await
        .unwrap();
    let (from, to) = (at("2030-03-01T00:00:00Z"), at("2030-04-01T00:00:00Z"));
    assert_eq!(
        w.calendar
            .entries_for(&w.lea, Some(scope), from, to)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        w.calendar
            .entries_for(&w.admin, Some(scope), from, to)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        w.calendar
            .entries_for(&w.max, Some(scope), from, to)
            .await
            .unwrap()
            .len(),
        0,
        "not a member"
    );
    assert_eq!(
        w.calendar.entry(&w.max, lab.id).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    w.drop().await;
}
