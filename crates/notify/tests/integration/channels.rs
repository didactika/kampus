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
//! address, or one the mail server refuses for good, is given up; devices their push service says are gone
//! are forgotten.

use std::sync::Arc;

use kampus_common::ErrorKind;
use kampus_mail::{Mail, Mailer};
use kampus_module::ServiceContext;
use kampus_notify::ports::transport::{MailTransport, PushTransport};
use kampus_notify::service::channel::{CampusDefault, PushSubscription, Template};
use kampus_push::{Device, Outcome, Pusher};
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

/// A mail server that refuses every message for good.
#[derive(Debug)]
struct Refusing;

#[async_trait::async_trait]
impl Mailer for Refusing {
    async fn send(&self, _: &Mail) -> kampus_common::Result<()> {
        Err(kampus_common::Error::validation(
            "mail.rejected",
            vec![kampus_common::FieldError::new("to", "rejected")],
        ))
    }

    async fn check(&self) -> kampus_common::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn email_goes_through_the_mail_port_and_a_refusal_for_good_is_not_tried_again() {
    let w = world().await;
    let notify = w
        .notify
        .clone()
        .with_transports(Arc::new(MailTransport::new(Arc::new(Refusing))), w.push.clone());
    notify.set_preference(&w.lea, GRADE, "email", true).await.unwrap();
    notify
        .notify(GRADE, who(&w.lea), json!({}), Some("e1"))
        .await
        .unwrap();
    assert_eq!(notify.deliver(&services(&w)).await.unwrap(), 1);
    let (state, error): (String, Option<String>) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select state, last_error from {}.\"delivery\" where channel = 'email'",
        w.db.prefix().schema("notify")
    )))
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(
        (state.as_str(), error.as_deref()),
        ("dead", Some("mail.rejected"))
    );
    w.drop().await;
}

/// Push services where every device whose endpoint ends in `gone` is gone, and the others take messages.
#[derive(Debug, Default)]
struct Services {
    /// The endpoints delivered to, oldest first.
    delivered: std::sync::Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl Pusher for Services {
    async fn push(&self, device: &Device, payload: &[u8]) -> kampus_common::Result<Outcome> {
        let message: serde_json::Value = serde_json::from_slice(payload).unwrap();
        assert!(
            message["title"].is_string() && message["body"].is_string(),
            "{message}"
        );
        if device.endpoint.ends_with("gone") {
            return Ok(Outcome::Gone);
        }
        self.delivered.lock().unwrap().push(device.endpoint.clone());
        Ok(Outcome::Delivered)
    }

    fn public_key(&self) -> String {
        "the key".to_owned()
    }
}

/// Registers a device of a person.
async fn device(w: &World, notify: &kampus_notify::Notify, endpoint: &str) {
    notify
        .register_push(
            &w.lea,
            PushSubscription {
                id: None,
                endpoint: endpoint.to_owned(),
                p256dh: "key".to_owned(),
                auth: "secret".to_owned(),
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn push_goes_through_the_push_port_and_devices_that_are_gone_are_forgotten() {
    let w = world().await;
    assert_eq!(w.notify.push_key().unwrap_err().code(), "notify.push_unavailable");
    let pusher = Arc::new(Services::default());
    let notify = w
        .notify
        .clone()
        .with_transports(w.email.clone(), Arc::new(PushTransport::new(pusher.clone())));
    assert_eq!(notify.push_key().unwrap(), "the key");
    notify.set_preference(&w.lea, GRADE, "push", true).await.unwrap();
    device(&w, &notify, "https://push.example.org/phone").await;
    device(&w, &notify, "https://push.example.org/old-gone").await;
    notify
        .notify(GRADE, who(&w.lea), json!({}), Some("e1"))
        .await
        .unwrap();
    notify.deliver(&services(&w)).await.unwrap();
    assert_eq!(
        *pusher.delivered.lock().unwrap(),
        ["https://push.example.org/phone"]
    );
    let left: Vec<_> = notify
        .push_subscriptions(&w.lea)
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.endpoint)
        .collect();
    assert_eq!(
        left,
        ["https://push.example.org/phone"],
        "the device that is gone is forgotten"
    );

    w.notify
        .remove_push(
            &w.lea,
            notify.push_subscriptions(&w.lea).await.unwrap()[0].id.unwrap(),
        )
        .await
        .unwrap();
    device(&w, &notify, "https://push.example.org/last-gone").await;
    notify
        .notify(GRADE, who(&w.lea), json!({}), Some("e2"))
        .await
        .unwrap();
    notify.deliver(&services(&w)).await.unwrap();
    let error: Option<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select d.last_error from {s}.\"delivery\" d join {s}.\"notification\" n on n.id = d.notification_id
         where d.channel = 'push' and d.state = 'dead'",
        s = w.db.prefix().schema("notify")
    )))
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(
        error.as_deref(),
        Some("notify.no_address"),
        "nobody is left to receive it"
    );
    assert_eq!(notify.push_subscriptions(&w.lea).await.unwrap().len(), 0);
    w.drop().await;
}
