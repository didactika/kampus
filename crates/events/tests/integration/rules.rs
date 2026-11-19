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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of events, against PostgreSQL: publishing in a transaction, declared payloads,
//! deliveries with retries, dead letters and replay, order per object, the audit log, the limit of chains,
//! and listeners turned off.

use std::sync::Mutex;

use kampus_common::Error;
use kampus_db::{Database, Prefix, Upgrader};
use kampus_events::{Dispatcher, Events, MAX_DEPTH, Publish};
use kampus_module::{EventDeclaration, EventRun, JobFuture, ListenerDeclaration, Module, ServiceContext};
use serde_json::json;

/// What the listeners of these tests saw, in order: the object and the value of `n` of each event.
static SEEN: Mutex<Vec<(uuid::Uuid, i64)>> = Mutex::new(Vec::new());

/// The events of these tests.
const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: "testing.happened",
        version: 1,
        summary: "Something happened",
        fields: &["n"],
        audited: false,
        personal_fields: &[],
    },
    EventDeclaration {
        code: "testing.audited",
        version: 1,
        summary: "Something worth auditing happened",
        fields: &[],
        audited: true,
        personal_fields: &[],
    },
];

/// Records what it saw; fails for a negative `n`.
fn record(run: EventRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let n = run.event.payload["n"].as_i64().unwrap_or_default();
        if n < 0 {
            return Err(Error::unavailable("testing.refused"));
        }
        SEEN.lock()
            .unwrap()
            .push((run.event.object.unwrap_or_default(), n));
        Ok(())
    })
}

/// The listeners of these tests.
const LISTENERS: &[ListenerDeclaration] = &[ListenerDeclaration {
    event: "testing.happened",
    name: "record",
    run: record,
}];

/// A module that declares the events and listeners of these tests.
const TESTING: Module = Module {
    name: "testing",
    events: EVENTS,
    listeners: LISTENERS,
    jobs: &[],
    schedules: &[],
    ..kampus_events::MODULE
};

/// An installation with the events of these tests.
struct World {
    /// The events.
    events: Events,
    /// The dispatcher of the listeners of these tests.
    dispatcher: Dispatcher,
    /// The database.
    db: Database,
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("v{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    Upgrader::new(&db, &[kampus_events::MODULE.schema()], "0.1.0")
        .apply(true)
        .await
        .unwrap();
    let events = Events::new(db.clone());
    events.sync_declarations(&[&TESTING]).await.unwrap();
    let dispatcher = Dispatcher::new(events.clone(), &[&TESTING], ServiceContext::default());
    World {
        events,
        dispatcher,
        db,
    }
}

impl World {
    /// Publishes an event about an object, committed.
    async fn publish(&self, object: uuid::Uuid, n: i64) {
        let mut tx = self.db.begin().await.unwrap();
        self.events
            .publish_in(
                &mut tx,
                Publish::new("testing.happened", Some(object), json!({ "n": n })),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    /// Delivers until nothing is left to do now.
    async fn drain(&self) {
        while self.dispatcher.run_once().await.unwrap() {}
    }

    /// Makes every pending delivery due now, so retries need no waiting.
    async fn hurry(&self) {
        self.sql("update {schema}.\"delivery\" set next_attempt_at = now() where state = 'pending'")
            .await;
    }

    /// Runs SQL in the schema of events.
    async fn sql(&self, text: &str) {
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            text.replace("{schema}", &self.db.prefix().schema("events")),
        ))
        .execute(self.db.pool())
        .await
        .unwrap();
    }

    /// Drops the installation.
    async fn drop(self) {
        for module in ["events", "db"] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
    }
}

/// What the listener saw about these objects since the last call, in order.
fn seen(objects: &[uuid::Uuid]) -> Vec<i64> {
    let mut all = SEEN.lock().unwrap();
    let (mine, rest): (Vec<_>, Vec<_>) = all.drain(..).partition(|(o, _)| objects.contains(o));
    *all = rest;
    mine.into_iter().map(|(_, n)| n).collect()
}

#[tokio::test]
async fn events_are_published_with_their_change_and_delivered_after_commit_in_order() {
    let w = world().await;
    let object = uuid::Uuid::now_v7();
    let mut tx = w.db.begin().await.unwrap();
    w.events
        .publish_in(
            &mut tx,
            Publish::new("testing.happened", Some(object), json!({ "n": 99 })),
        )
        .await
        .unwrap();
    drop(tx);
    w.drain().await;
    assert_eq!(
        seen(&[object]),
        Vec::<i64>::new(),
        "a rolled-back change publishes nothing"
    );

    let mut tx = w.db.begin().await.unwrap();
    assert_eq!(
        w.events
            .publish_in(&mut tx, Publish::new("testing.unknown", None, json!({})))
            .await
            .unwrap_err()
            .code(),
        "events.type_unknown"
    );
    assert_eq!(
        w.events
            .publish_in(&mut tx, Publish::new("testing.happened", None, json!({ "m": 1 })))
            .await
            .unwrap_err()
            .code(),
        "events.payload_invalid",
        "every declared field is in the payload"
    );
    drop(tx);

    for n in 1..=3 {
        w.publish(object, n).await;
    }
    w.drain().await;
    assert_eq!(seen(&[object]), [1, 2, 3]);
    w.drop().await;
}

#[tokio::test]
async fn a_failing_delivery_holds_later_events_of_its_object_dies_and_can_be_replayed() {
    let w = world().await;
    let object = uuid::Uuid::now_v7();
    let other = uuid::Uuid::now_v7();
    w.publish(object, -1).await;
    w.publish(object, 2).await;
    w.publish(other, 7).await;
    w.drain().await;
    assert_eq!(
        seen(&[object, other]),
        [7],
        "the second event of the object waits for the first; other objects do not"
    );
    for _ in 0..kampus_events::service::delivery::MAX_ATTEMPTS {
        w.hurry().await;
        w.drain().await;
    }
    let dead = w.events.deliveries("dead", 10).await.unwrap();
    assert_eq!(dead.len(), 1);
    assert!(dead[0].last_error.as_deref().unwrap().contains("testing.refused"));
    w.drain().await;
    assert_eq!(seen(&[object]), [2], "once the first is dead, the next one goes");
    assert_eq!(w.events.replay(Some(dead[0].event)).await.unwrap(), 1);
    assert_eq!(w.events.deliveries("pending", 10).await.unwrap().len(), 1);
    w.drop().await;
}

#[tokio::test]
async fn an_audited_event_keeps_an_entry_that_cannot_be_changed() {
    let w = world().await;
    let actor = uuid::Uuid::now_v7();
    let mut tx = w.db.begin().await.unwrap();
    let mut event = Publish::new("testing.audited", None, json!({}));
    event.actor = Some(actor);
    event.summary = Some("something worth auditing".to_owned());
    w.events.publish_in(&mut tx, event).await.unwrap();
    tx.commit().await.unwrap();
    let entries = w.events.audit(None, Some(actor), 10).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].summary, "something worth auditing");
    let changed = sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {}.\"audit_entry\" set summary = 'rewritten'",
        w.db.prefix().schema("events")
    )))
    .execute(w.db.pool())
    .await;
    assert!(changed.is_err(), "the audit log is append-only");
    let deleted = sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "delete from {}.\"audit_entry\"",
        w.db.prefix().schema("events")
    )))
    .execute(w.db.pool())
    .await;
    assert!(deleted.is_err(), "only its retention removes entries");
    assert_eq!(
        w.events.purge().await.unwrap().audit_entries,
        0,
        "recent entries stay"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_chain_of_events_stops_at_its_limit() {
    let w = world().await;
    let deep = kampus_module::Event {
        id: uuid::Uuid::now_v7(),
        code: "testing.happened".to_owned(),
        version: 1,
        scope: None,
        actor: None,
        object: None,
        payload: json!({ "n": 0 }),
        occurred_at: kampus_common::Timestamp::parse_rfc3339("2026-11-19T00:00:00Z").unwrap(),
        depth: MAX_DEPTH,
    };
    let mut tx = w.db.begin().await.unwrap();
    let mut event = Publish::new("testing.happened", None, json!({ "n": 1 }));
    event.cause = Some(&deep);
    assert_eq!(
        w.events.publish_in(&mut tx, event).await.unwrap_err().code(),
        "events.chain_limit_reached"
    );
    drop(tx);
    w.drop().await;
}

#[tokio::test]
async fn the_deliveries_of_a_listener_turned_off_wait_and_are_not_lost() {
    let w = world().await;
    w.events.set_listener("testing", "record", false).await.unwrap();
    let object = uuid::Uuid::now_v7();
    w.publish(object, 5).await;
    w.drain().await;
    assert_eq!(seen(&[object]), Vec::<i64>::new());
    assert_eq!(w.events.deliveries("pending", 10).await.unwrap().len(), 1);
    w.events.set_listener("testing", "record", true).await.unwrap();
    w.drain().await;
    assert_eq!(seen(&[object]), [5]);
    w.drop().await;
}
