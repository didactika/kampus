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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The inbox and the preferences: a notification reaches a person when they receive its type, once for each
//! cause, and a required type cannot be turned off.

use kampus_common::ErrorKind;
use kampus_module::{Module, NotificationTypeDeclaration};
use serde_json::json;

use crate::world::{who, world};

/// A type nobody can turn off.
const ANNOUNCEMENT: &str = "news.announcement";

/// A module that declares one required type.
pub(crate) static NEWS_MODULE: Module = Module {
    name: "news",
    notification_types: &[NotificationTypeDeclaration {
        code: ANNOUNCEMENT,
        summary: "An announcement of the campus",
        channels: &["inbox"],
        required: true,
    }],
    ..kampus_records::MODULE
};

#[tokio::test]
async fn a_notification_reaches_the_inbox_once_per_cause_and_is_read_newest_first() {
    let w = world().await;
    let lea = who(&w.lea);
    let first = w
        .notify
        .notify("records.grade_posted", lea, json!({ "n": 1 }), Some("event-1"))
        .await
        .unwrap()
        .unwrap();
    assert!(
        w.notify
            .notify("records.grade_posted", lea, json!({ "n": 1 }), Some("event-1"))
            .await
            .unwrap()
            .is_none(),
        "the same cause notifies once"
    );
    w.notify
        .notify("records.grade_posted", lea, json!({ "n": 2 }), Some("event-2"))
        .await
        .unwrap();
    w.notify
        .notify("progress.state_decided", lea, json!({ "n": 3 }), None)
        .await
        .unwrap();

    let page = w.notify.inbox(&w.lea, false, 2, None).await.unwrap();
    assert_eq!(page.notifications.len(), 2);
    assert_eq!(page.unread, 3);
    assert_eq!(page.notifications[0].payload["n"], 3, "newest first");
    let next = w.notify.inbox(&w.lea, false, 2, page.next).await.unwrap();
    assert_eq!(next.notifications.len(), 1);
    assert_eq!(next.notifications[0].id, first.id);
    assert!(next.next.is_none());

    let read = w.notify.mark_read(&w.lea, first.id).await.unwrap();
    assert!(read.read_at.is_some());
    assert_eq!(
        w.notify.mark_read(&w.lea, first.id).await.unwrap().read_at,
        read.read_at
    );
    let unread = w.notify.inbox(&w.lea, true, 10, None).await.unwrap();
    assert_eq!(unread.notifications.len(), 2);
    assert_eq!(unread.unread, 2);
    assert_eq!(w.notify.mark_all_read(&w.lea).await.unwrap(), 2);
    assert_eq!(w.notify.inbox(&w.lea, true, 10, None).await.unwrap().unread, 0);

    let theirs = w.notify.mark_read(&w.max, first.id).await.unwrap_err();
    assert_eq!(
        theirs.kind(),
        ErrorKind::NotFound,
        "nobody reads the notifications of others"
    );
    assert_eq!(
        w.notify
            .inbox(&w.max, false, 10, None)
            .await
            .unwrap()
            .notifications
            .len(),
        0
    );
    let unknown = w
        .notify
        .notify("nobody.declared_this", lea, json!({}), None)
        .await
        .unwrap_err();
    assert_eq!(unknown.code(), "notify.type_unknown");
    w.drop().await;
}

#[tokio::test]
async fn a_person_turns_types_off_except_the_required_ones() {
    let w = world().await;
    let lea = who(&w.lea);
    let all = w.notify.preferences(&w.lea).await.unwrap();
    let codes: Vec<_> = all
        .iter()
        .map(|p| (p.type_code.as_str(), p.enabled, p.required))
        .collect();
    assert_eq!(
        codes,
        [
            (ANNOUNCEMENT, true, true),
            ("progress.state_decided", true, false),
            ("records.grade_posted", true, false),
        ]
    );

    let off = w
        .notify
        .set_preference(&w.lea, "records.grade_posted", "inbox", false)
        .await
        .unwrap();
    assert!(!off.enabled);
    assert!(
        w.notify
            .notify("records.grade_posted", lea, json!({}), None)
            .await
            .unwrap()
            .is_none(),
        "a type turned off is not delivered"
    );
    assert!(
        w.notify
            .notify("records.grade_posted", who(&w.max), json!({}), None)
            .await
            .unwrap()
            .is_some(),
        "and only for the person who turned it off"
    );
    w.notify
        .set_preference(&w.lea, "records.grade_posted", "inbox", true)
        .await
        .unwrap();
    assert!(
        w.notify
            .notify("records.grade_posted", lea, json!({}), None)
            .await
            .unwrap()
            .is_some()
    );

    let required = w
        .notify
        .set_preference(&w.lea, ANNOUNCEMENT, "inbox", false)
        .await
        .unwrap_err();
    assert_eq!(required.code(), "notify.preference_invalid");
    let channel = w
        .notify
        .set_preference(&w.lea, "records.grade_posted", "carrier_pigeon", true)
        .await
        .unwrap_err();
    assert_eq!(channel.kind(), ErrorKind::Validation);
    let unknown = w
        .notify
        .set_preference(&w.lea, "nobody.declared_this", "inbox", true)
        .await
        .unwrap_err();
    assert_eq!(unknown.kind(), ErrorKind::NotFound);
    w.drop().await;
}
