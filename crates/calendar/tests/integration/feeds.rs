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

//! Feeds and subscriptions: a person reads what they see through a secret address until they revoke it, and
//! a course shows the events of another calendar, read again with what changed.

use std::fmt::Write;
use std::sync::{Arc, Mutex};

use kampus_calendar::{EntryFields, ItemKind};
use kampus_common::{ErrorKind, Timestamp};

use crate::world::world;

/// A moment from its text.
fn at(text: &str) -> Timestamp {
    Timestamp::parse_rfc3339(text).unwrap()
}

/// The fields of an entry within the next weeks.
fn soon(title: &str) -> EntryFields {
    let start = time::OffsetDateTime::now_utc() + time::Duration::days(7);
    EntryFields {
        title: title.to_owned(),
        description: None,
        starts_at: Timestamp::from_offset_date_time(start),
        ends_at: Timestamp::from_offset_date_time(start + time::Duration::hours(1)),
        time_zone: None,
        all_day: false,
        recurrence: None,
        group: None,
    }
}

#[tokio::test]
async fn a_feed_gives_what_its_owner_sees_until_it_is_revoked() {
    let w = world().await;
    w.calendar
        .create_entry(&w.admin, Some(w.course.scope), soon("Exam"))
        .await
        .unwrap();
    w.calendar
        .create_entry(&w.lea, None, soon("Study"))
        .await
        .unwrap();
    w.calendar
        .create_entry(&w.max, None, soon("Not hers"))
        .await
        .unwrap();
    let (feed, secret) = w.calendar.create_feed(&w.lea, "Phone").await.unwrap();
    assert_eq!(secret.len(), 64);
    let text = w.calendar.feed_calendar(&secret).await.unwrap();
    assert!(text.contains("SUMMARY:Exam"), "{text}");
    assert!(text.contains("SUMMARY:Study"));
    assert!(!text.contains("Not hers"));
    assert!(w.calendar.feeds(&w.lea).await.unwrap()[0].last_used_at.is_some());
    assert_eq!(
        w.calendar.revoke_feed(&w.max, feed.id).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    w.calendar.revoke_feed(&w.lea, feed.id).await.unwrap();
    assert_eq!(
        w.calendar.feed_calendar(&secret).await.unwrap_err().kind(),
        ErrorKind::NotFound
    );
    w.drop().await;
}

/// Serves a calendar whose text the test changes; returns its address and the text.
async fn serve() -> (String, Arc<Mutex<String>>) {
    let text = Arc::new(Mutex::new(String::new()));
    let served = text.clone();
    let app = axum::Router::new().route(
        "/school.ics",
        axum::routing::get(move || {
            let body = served.lock().unwrap().clone();
            async move { body }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}/school.ics"), text)
}

/// A calendar with events of an hour, by UID and title.
fn calendar(events: &[(&str, &str, &str)]) -> String {
    let mut text = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n");
    for (uid, title, start) in events {
        let _ = write!(
            text,
            "BEGIN:VEVENT\r\nUID:{uid}\r\nSUMMARY:{title}\r\nDTSTART:{start}\r\nEND:VEVENT\r\n"
        );
    }
    text.push_str("END:VCALENDAR\r\n");
    text
}

#[tokio::test]
async fn a_subscription_shows_the_events_of_another_calendar_and_follows_its_changes() {
    let w = world().await;
    let scope = w.course.scope;
    let (url, text) = serve().await;
    *text.lock().unwrap() = calendar(&[
        ("a", "Holiday", "20300101T090000Z"),
        ("b", "Open day", "20300102T090000Z"),
    ]);
    assert_eq!(
        w.calendar
            .subscribe(&w.lea, scope, &url, "School", None)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    let refused = w
        .calendar
        .subscribe(&w.admin, scope, "ftp://example.org/x.ics", "School", None)
        .await
        .unwrap_err();
    assert_eq!(refused.code(), "calendar.subscription_invalid");
    let subscription = w
        .calendar
        .subscribe(&w.admin, scope, &url, "School", None)
        .await
        .unwrap();
    let first = w.calendar.refresh_now(&w.admin, subscription.id).await.unwrap();
    assert_eq!((first.added, first.changed, first.removed), (2, 0, 0));
    let (from, to) = (at("2029-12-01T00:00:00Z"), at("2030-02-01T00:00:00Z"));
    let seen = w
        .calendar
        .entries_for(&w.lea, Some(scope), from, to)
        .await
        .unwrap();
    assert_eq!(seen.len(), 2);
    assert!(seen.iter().all(|i| i.kind == ItemKind::Imported));

    *text.lock().unwrap() = calendar(&[
        ("a", "Holiday (moved)", "20300103T090000Z"),
        ("c", "Exam week", "20300110T090000Z"),
    ]);
    let second = w.calendar.refresh_now(&w.admin, subscription.id).await.unwrap();
    assert_eq!((second.added, second.changed, second.removed), (1, 1, 1));
    let third = w.calendar.refresh_now(&w.admin, subscription.id).await.unwrap();
    assert_eq!(
        (third.added, third.changed, third.removed),
        (0, 0, 0),
        "nothing changed"
    );
    let titles: Vec<String> = w
        .calendar
        .entries_for(&w.lea, Some(scope), from, to)
        .await
        .unwrap()
        .into_iter()
        .map(|i| i.title)
        .collect();
    assert_eq!(titles, ["Holiday (moved)", "Exam week"]);

    w.calendar.unsubscribe(&w.admin, subscription.id).await.unwrap();
    assert_eq!(
        w.calendar
            .entries_for(&w.lea, Some(scope), from, to)
            .await
            .unwrap()
            .len(),
        0
    );
    w.drop().await;
}
