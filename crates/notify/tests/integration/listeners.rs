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

//! What other modules cause: a grade the person may see is announced, a hidden one is not, a state a teacher
//! decides is announced, and a listener that runs twice for the same event notifies once.

use kampus_common::Timestamp;
use kampus_module::{Event, EventRun, ServiceContext};
use kampus_notify::observers::LISTENERS;
use kampus_progress::{RuleKind, State};
use kampus_records::{GradeWrite, RecordFields, Source};
use rust_decimal::Decimal;
use serde_json::json;

use crate::world::{World, who, world};

/// An event as a listener receives it.
fn event(code: &str, payload: serde_json::Value) -> Event {
    Event {
        id: uuid::Uuid::now_v7(),
        code: code.to_owned(),
        version: 1,
        scope: None,
        actor: None,
        object: None,
        payload,
        occurred_at: Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc()),
        depth: 0,
    }
}

/// The services listeners read.
fn services(w: &World) -> ServiceContext {
    ServiceContext::default()
        .with(w.records.clone())
        .with(w.notify.clone())
        .with(w.progress.clone())
}

/// Runs the listener of an event name.
async fn run(w: &World, name: &str, event: &Event) {
    let listener = LISTENERS.iter().find(|l| l.name == name).unwrap();
    let services = services(w);
    (listener.run)(EventRun {
        event,
        services: &services,
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_new_grade_is_announced_only_when_the_person_may_see_it() {
    let w = world().await;
    let lea = who(&w.lea);
    let record = w
        .records
        .define_record(
            &w.admin,
            w.course.id,
            None,
            RecordFields::new("Quiz", Decimal::TEN),
        )
        .await
        .unwrap();
    w.records
        .set_grades(
            &w.admin,
            record.id,
            vec![GradeWrite {
                account: lea,
                raw: Some(Decimal::from(7)),
                text: None,
                feedback: None,
            }],
            Source::Teacher,
        )
        .await
        .unwrap();
    let grade_event = event(
        "records.grade_changed",
        json!({ "record": record.id, "course": w.course.id, "account": lea }),
    );
    run(&w, "notify_grade_posted", &grade_event).await;
    run(&w, "notify_grade_posted", &grade_event).await;
    let inbox = w.notify.inbox(&w.lea, false, 10, None).await.unwrap();
    assert_eq!(inbox.notifications.len(), 1, "the same event notifies once");
    assert_eq!(inbox.notifications[0].type_code, "records.grade_posted");
    assert_eq!(inbox.notifications[0].payload["course"], json!(w.course.id));

    let later = Timestamp::parse_rfc3339("2099-01-01T00:00:00Z").unwrap();
    w.records
        .set_grade_hidden(&w.admin, record.id, lea, Some(later))
        .await
        .unwrap();
    let hidden_event = event(
        "records.grade_changed",
        json!({ "record": record.id, "course": w.course.id, "account": lea }),
    );
    run(&w, "notify_grade_posted", &hidden_event).await;
    assert_eq!(
        w.notify
            .inbox(&w.lea, false, 10, None)
            .await
            .unwrap()
            .notifications
            .len(),
        1,
        "a hidden grade is not announced"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_state_decided_by_a_teacher_is_announced() {
    let w = world().await;
    let lea = who(&w.lea);
    let page = w.page("Lab").await;
    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::Submit])
        .await
        .unwrap();
    w.progress
        .set_exception(&w.admin, page.id, lea, State::Done, "Handed in on paper")
        .await
        .unwrap();
    let decided = event(
        "progress.exception_changed",
        json!({ "module": page.id, "course": w.course.id, "account": lea, "state": "done" }),
    );
    run(&w, "notify_state_decided", &decided).await;
    let inbox = w.notify.inbox(&w.lea, false, 10, None).await.unwrap();
    assert_eq!(inbox.notifications.len(), 1);
    assert_eq!(inbox.notifications[0].type_code, "progress.state_decided");
    assert_eq!(inbox.notifications[0].payload["state"], "done");
    w.drop().await;
}
