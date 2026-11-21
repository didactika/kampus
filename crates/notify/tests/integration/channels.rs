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

//! Channels: a person who turns email or push on receives a notification there through a delivery job, with
//! the template of their campus; a campus default that is locked cannot be changed; a delivery without an
//! address is given up.

use kampus_common::ErrorKind;
use kampus_module::ServiceContext;
use kampus_notify::service::channel::{CampusDefault, PushSubscription, Template};
use serde_json::json;

use crate::world::{World, who, world};

/// The type these tests notify.
const GRADE: &str = "records.grade_posted";

/// The services a delivery reads.
fn services(w: &World) -> ServiceContext {
    ServiceContext::default().with(w.identity.clone())
}

#[tokio::test]
async fn an_email_goes_out_with_the_template_of_the_campus() {
    let w = world().await;
    w.notify
        .set_preference(&w.lea, GRADE, "email", true)
        .await
        .unwrap();
    w.notify
        .set_template(
            &w.admin,
            w.campus,
            Template {
                type_code: GRADE.to_owned(),
                channel: "email".to_owned(),
                locale: "EN".to_owned(),
                subject: "New grade in {{course}}".to_owned(),
                body: "{{summary}}: {{record}}".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        w.notify.templates(&w.lea, w.campus).await.unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
    w.notify
        .notify(
            GRADE,
            who(&w.lea),
            json!({ "course": "Physics", "record": "Quiz" }),
            Some("e1"),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(w.notify.deliver(&services(&w)).await.unwrap(), 1);
    let sent = w.email.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, ["lea@example.org"]);
    assert_eq!(sent[0].subject, "New grade in Physics");
    assert!(sent[0].body.ends_with(": Quiz"), "{}", sent[0].body);
    assert_eq!(w.notify.deliver(&services(&w)).await.unwrap(), 0, "sent once");
    w.drop().await;
}

#[tokio::test]
async fn push_reaches_the_devices_of_a_person_and_waits_for_none_without_them() {
    let w = world().await;
    w.notify
        .set_preference(&w.lea, GRADE, "push", true)
        .await
        .unwrap();
    w.notify
        .notify(GRADE, who(&w.lea), json!({}), Some("e1"))
        .await
        .unwrap();
    w.notify.deliver(&services(&w)).await.unwrap();
    assert!(w.push.sent().is_empty(), "no device: the delivery is given up");

    let refused = w
        .notify
        .register_push(
            &w.lea,
            PushSubscription {
                id: None,
                endpoint: "http://push.example.org/1".to_owned(),
                p256dh: "key".to_owned(),
                auth: "secret".to_owned(),
            },
        )
        .await
        .unwrap_err();
    assert_eq!(refused.code(), "notify.push_invalid");
    let phone = w
        .notify
        .register_push(
            &w.lea,
            PushSubscription {
                id: None,
                endpoint: "https://push.example.org/1".to_owned(),
                p256dh: "key".to_owned(),
                auth: "secret".to_owned(),
            },
        )
        .await
        .unwrap();
    w.notify
        .notify(GRADE, who(&w.lea), json!({}), Some("e2"))
        .await
        .unwrap();
    w.notify.deliver(&services(&w)).await.unwrap();
    assert_eq!(w.push.sent()[0].to, ["https://push.example.org/1"]);
    w.notify.remove_push(&w.lea, phone.id.unwrap()).await.unwrap();
    assert_eq!(w.notify.push_subscriptions(&w.lea).await.unwrap().len(), 0);
    w.drop().await;
}

#[tokio::test]
async fn a_locked_default_of_the_campus_wins_over_the_person() {
    let w = world().await;
    w.notify
        .set_campus_default(
            &w.admin,
            w.campus,
            CampusDefault {
                type_code: GRADE.to_owned(),
                channel: "email".to_owned(),
                enabled: true,
                locked: true,
            },
        )
        .await
        .unwrap();
    let email = |prefs: &[kampus_notify::Preference]| {
        prefs
            .iter()
            .find(|p| p.type_code == GRADE && p.channel == "email")
            .cloned()
            .unwrap()
    };
    let seen = email(&w.notify.preferences(&w.max).await.unwrap());
    assert!(seen.enabled && seen.locked);
    assert_eq!(
        w.notify
            .set_preference(&w.max, GRADE, "email", false)
            .await
            .unwrap_err()
            .code(),
        "notify.preference_locked"
    );
    w.notify
        .set_campus_default(
            &w.admin,
            w.campus,
            CampusDefault {
                type_code: GRADE.to_owned(),
                channel: "email".to_owned(),
                enabled: false,
                locked: false,
            },
        )
        .await
        .unwrap();
    assert!(!email(&w.notify.preferences(&w.max).await.unwrap()).enabled);
    w.notify
        .set_preference(&w.max, GRADE, "email", true)
        .await
        .unwrap();
    assert!(
        email(&w.notify.preferences(&w.max).await.unwrap()).enabled,
        "the choice of the person wins over a default that is not locked"
    );
    assert_eq!(
        w.notify
            .set_campus_default(
                &w.lea,
                w.campus,
                CampusDefault {
                    type_code: GRADE.to_owned(),
                    channel: "email".to_owned(),
                    enabled: true,
                    locked: true,
                },
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}
