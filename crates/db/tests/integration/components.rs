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
// @package   kampus-db
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The registry of components against PostgreSQL: versions and the history of runs, data actions inside
//! their steps, the way back to an older release, and uninstalling a component.

use kampus_common::{Error, ErrorKind};
use kampus_db::schema::Change;
use kampus_db::{
    Action, ActionContext, ActionFuture, ComponentRecord, Database, ModuleSteps, Prefix, Problem, Step,
    Upgrader,
};
use sqlx::PgConnection;

/// Creates the table of notes.
const CREATE: Step = Step {
    sequence: 1,
    name: "create note",
    revert: &[],
    action: None,
    changes: &[Change::Postgres {
        sql: "create table {schema}.note (id uuid primary key);",
        reason: "a sample module of the tests",
    }],
};

/// Adds the text of a note, and says how to remove it.
const TEXT: Step = Step {
    sequence: 2,
    name: "add note text",
    revert: &[Change::Postgres {
        sql: "alter table {schema}.note drop column body;",
        reason: "a sample module of the tests",
    }],
    action: None,
    changes: &[Change::Postgres {
        sql: "alter table {schema}.note add column body text;",
        reason: "a sample module of the tests",
    }],
};

/// Adds an index, and says how to remove it.
const INDEX: Step = Step {
    sequence: 3,
    name: "index note text",
    revert: &[Change::Postgres {
        sql: "drop index {schema}.note_body_idx;",
        reason: "a sample module of the tests",
    }],
    action: None,
    changes: &[Change::Postgres {
        sql: "create index note_body_idx on {schema}.note (body);",
        reason: "a sample module of the tests",
    }],
};

/// The same index, without a way back.
const INDEX_FOREVER: Step = Step { revert: &[], ..INDEX };

/// The first step with an action that fails.
const FAILING: &[Step] = &[Step {
    action: Some(Action {
        name: "refuse",
        run: refuse,
    }),
    ..CREATE
}];

/// The first step with an action that adds the first note.
const SEEDING: &[Step] = &[Step {
    action: Some(Action {
        name: "seed",
        run: seed,
    }),
    ..CREATE
}];

/// The first step with its action renamed after release.
const RENAMED: &[Step] = &[Step {
    action: Some(Action {
        name: "seed differently",
        run: seed,
    }),
    ..CREATE
}];

/// Installs the first note when the module is installed.
fn seed<'c>(conn: &'c mut PgConnection, context: &'c ActionContext) -> ActionFuture<'c> {
    Box::pin(async move {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into {} (id) values ($1)",
            context.table("note")
        )))
        .bind(uuid::Uuid::now_v7())
        .execute(conn)
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    })
}

/// An action that always fails.
fn refuse<'c>(_conn: &'c mut PgConnection, _context: &'c ActionContext) -> ActionFuture<'c> {
    Box::pin(async { Err(Error::conflict("test.refused")) })
}

/// A component of the build.
fn component(name: &str, version: &str, schema: Option<&'static str>) -> ComponentRecord {
    ComponentRecord {
        name: name.to_owned(),
        kind: if schema.is_some() { "module" } else { "adapter" },
        origin: "core",
        version: version.to_owned(),
        maturity: "alpha",
        schema,
    }
}

/// Connects to a fresh installation with a prefix of its own.
async fn installation() -> Database {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("c{}_", &simple[simple.len() - 6..]).parse().unwrap();
    Database::connect(&url, prefix, 4).await.unwrap()
}

/// Drops the schemas of a test installation.
async fn drop_installation(db: &Database) {
    for module in ["db", "notes"] {
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "drop schema if exists {} cascade",
            db.prefix().schema(module)
        )))
        .execute(db.pool())
        .await
        .unwrap();
    }
}

/// The notes module with these steps.
const fn notes(steps: &'static [Step]) -> ModuleSteps {
    ModuleSteps {
        module: "notes",
        steps,
    }
}

/// Whether a column of the notes table exists.
async fn has_column(db: &Database, column: &str) -> bool {
    sqlx::query_scalar(
        "select exists (select 1 from information_schema.columns where table_schema = $1 and table_name = 'note' and column_name = $2)",
    )
    .bind(db.prefix().schema_name("notes"))
    .bind(column)
    .fetch_one(db.pool())
    .await
    .unwrap()
}

#[tokio::test]
async fn versions_are_registered_and_every_run_is_kept() {
    let db = installation().await;
    Upgrader::new(&db, &[notes(&[CREATE])], "0.1.0")
        .with_components(vec![
            component("notes", "1.0.0", Some("notes")),
            component("cache_memory", "0.1.0", None),
        ])
        .apply(true)
        .await
        .unwrap();
    let installed = db.installed_components().await.unwrap();
    let versions: Vec<_> = installed
        .iter()
        .map(|c| (c.name.as_str(), c.version.as_str()))
        .collect();
    assert_eq!(versions, [("cache_memory", "0.1.0"), ("notes", "1.0.0")]);

    let newer = Upgrader::new(&db, &[notes(&[CREATE, TEXT, INDEX])], "0.2.0").with_components(vec![
        component("notes", "1.2.0", Some("notes")),
        component("cache_memory", "0.1.0", None),
    ]);
    let plan = newer.plan().await.unwrap();
    assert_eq!(plan.changes.len(), 1);
    assert_eq!(
        (plan.changes[0].from.as_deref(), plan.changes[0].steps),
        (Some("1.0.0"), 2)
    );
    newer.apply(false).await.unwrap();
    assert!(newer.plan().await.unwrap().is_current());

    let bumped = Upgrader::new(&db, &[notes(&[CREATE, TEXT, INDEX])], "0.2.0").with_components(vec![
        component("notes", "1.2.0", Some("notes")),
        component("cache_memory", "0.1.1", None),
    ]);
    assert!(
        !bumped.plan().await.unwrap().is_current(),
        "a new version without steps is an upgrade too"
    );
    bumped.apply(false).await.unwrap();

    let history = db.upgrade_history(10).await.unwrap();
    assert_eq!(history.len(), 3);
    assert!(
        history
            .iter()
            .all(|run| run.outcome == "succeeded" && run.operation == "upgrade")
    );
    let first = &history[2];
    assert_eq!(first.release_from, None);
    assert_eq!(first.changes.len(), 2);
    let second = &history[1];
    assert_eq!(second.release_from.as_deref(), Some("0.1.0"));
    assert_eq!(
        (
            second.changes[0].component.as_str(),
            second.changes[0].to.as_deref(),
            second.changes[0].steps
        ),
        ("notes", Some("1.2.0"), 2)
    );
    assert_eq!(history[0].changes[0].from.as_deref(), Some("0.1.0"));
    drop_installation(&db).await;
}

#[tokio::test]
async fn a_newer_or_a_missing_component_stops_the_start() {
    let db = installation().await;
    Upgrader::new(&db, &[notes(&[CREATE])], "0.1.0")
        .with_components(vec![
            component("notes", "1.1.0", Some("notes")),
            component("cache_memory", "0.1.0", None),
        ])
        .apply(true)
        .await
        .unwrap();
    let older = Upgrader::new(&db, &[notes(&[CREATE])], "0.1.0").with_components(vec![component(
        "notes",
        "1.0.0",
        Some("notes"),
    )]);
    let problems = older.plan().await.unwrap().problems;
    assert!(problems.contains(&Problem::NewerComponent {
        component: "notes".to_owned(),
        recorded: "1.1.0".to_owned(),
        code: "1.0.0".to_owned(),
    }));
    assert!(problems.contains(&Problem::MissingComponent {
        component: "cache_memory".to_owned(),
        version: "0.1.0".to_owned(),
    }));
    let refused = older.apply(false).await.unwrap_err();
    assert_eq!(
        (refused.kind(), refused.code()),
        (ErrorKind::Conflict, "db.upgrade_refused")
    );
    drop_installation(&db).await;
}

#[tokio::test]
async fn an_action_runs_once_inside_its_step() {
    let db = installation().await;
    let error = Upgrader::new(&db, &[notes(FAILING)], "0.1.0")
        .with_components(vec![component("notes", "1.0.0", Some("notes"))])
        .apply(true)
        .await
        .unwrap_err();
    assert_eq!(error.code(), "db.action_failed");
    let table: bool = sqlx::query_scalar("select to_regclass($1) is not null")
        .bind(format!("{}.note", db.prefix().schema("notes")))
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert!(!table, "the changes of a step go with its failed action");
    let failed = &db.upgrade_history(1).await.unwrap()[0];
    assert_eq!(failed.outcome, "failed");
    assert!(failed.error.as_deref().is_some_and(|e| e.contains("refuse")));
    assert_eq!(db.installed_components().await.unwrap(), []);

    let upgrader = Upgrader::new(&db, &[notes(SEEDING)], "0.1.0").with_components(vec![component(
        "notes",
        "1.0.0",
        Some("notes"),
    )]);
    upgrader.apply(false).await.unwrap();
    upgrader.apply(false).await.unwrap();
    let seeded: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {}.note",
        db.prefix().schema("notes")
    )))
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(seeded, 1);
    let renamed = Upgrader::new(&db, &[notes(RENAMED)], "0.1.0");
    assert!(
        renamed
            .plan()
            .await
            .unwrap()
            .problems
            .contains(&Problem::ChangedStep {
                module: "notes".to_owned(),
                sequence: 1,
            }),
        "the action is part of the step"
    );
    drop_installation(&db).await;
}

#[tokio::test]
async fn a_downgrade_undoes_newer_steps_with_what_they_left() {
    let db = installation().await;
    Upgrader::new(&db, &[notes(&[CREATE, TEXT, INDEX])], "0.3.0")
        .with_components(vec![component("notes", "1.2.0", Some("notes"))])
        .apply(true)
        .await
        .unwrap();
    let older = Upgrader::new(&db, &[notes(&[CREATE])], "0.1.0").with_components(vec![component(
        "notes",
        "1.0.0",
        Some("notes"),
    )]);
    assert!(older.apply(false).await.is_err(), "an upgrade never goes back");

    let report = older.downgrade().await.unwrap();
    assert_eq!(report.reverted, ["notes/3", "notes/2"]);
    assert!(!has_column(&db, "body").await);
    assert!(older.plan().await.unwrap().is_current());
    let installed = db.installed_components().await.unwrap();
    assert_eq!(installed[0].version, "1.0.0");
    let run = &db.upgrade_history(1).await.unwrap()[0];
    assert_eq!(
        (
            run.operation.as_str(),
            run.release_from.as_deref(),
            run.release_to.as_str()
        ),
        ("downgrade", Some("0.3.0"), "0.1.0")
    );
    assert_eq!(
        (
            run.changes[0].from.as_deref(),
            run.changes[0].to.as_deref(),
            run.changes[0].steps
        ),
        (Some("1.2.0"), Some("1.0.0"), 2)
    );

    let again = Upgrader::new(&db, &[notes(&[CREATE, TEXT, INDEX])], "0.3.0")
        .with_components(vec![component("notes", "1.2.0", Some("notes"))]);
    again.apply(false).await.unwrap();
    assert!(
        has_column(&db, "body").await,
        "the steps apply again after a downgrade"
    );
    drop_installation(&db).await;
}

#[tokio::test]
async fn a_step_without_a_way_back_cannot_be_downgraded() {
    let db = installation().await;
    Upgrader::new(&db, &[notes(&[CREATE, TEXT, INDEX_FOREVER])], "0.3.0")
        .with_components(vec![component("notes", "1.2.0", Some("notes"))])
        .apply(true)
        .await
        .unwrap();
    let older = Upgrader::new(&db, &[notes(&[CREATE])], "0.1.0").with_components(vec![component(
        "notes",
        "1.0.0",
        Some("notes"),
    )]);
    let refused = older.downgrade().await.unwrap_err();
    assert_eq!(
        (refused.kind(), refused.code()),
        (ErrorKind::Conflict, "db.downgrade_impossible")
    );
    assert!(
        refused
            .params()
            .iter()
            .any(|(k, v)| k == "steps" && v == "notes/3")
    );
    assert!(
        has_column(&db, "body").await,
        "nothing is undone when one step cannot be"
    );
    drop_installation(&db).await;
}

#[tokio::test]
async fn a_component_no_longer_in_the_build_can_be_uninstalled() {
    let db = installation().await;
    Upgrader::new(&db, &[notes(&[CREATE, TEXT])], "0.1.0")
        .with_components(vec![
            component("notes", "1.0.0", Some("notes")),
            component("cache_memory", "0.1.0", None),
        ])
        .apply(true)
        .await
        .unwrap();
    let without =
        Upgrader::new(&db, &[], "0.1.0").with_components(vec![component("cache_memory", "0.1.0", None)]);
    assert!(!without.plan().await.unwrap().is_current());
    let still_built = without.uninstall("cache_memory").await.unwrap_err();
    assert_eq!(still_built.code(), "db.component_in_build");
    assert_eq!(
        without.uninstall("nothing").await.unwrap_err().kind(),
        ErrorKind::NotFound
    );

    let change = without.uninstall("notes").await.unwrap();
    assert_eq!(
        (change.from.as_deref(), change.to, change.steps),
        (Some("1.0.0"), None, 2)
    );
    assert!(without.plan().await.unwrap().is_current());
    let schema: bool = sqlx::query_scalar("select exists (select 1 from pg_namespace where nspname = $1)")
        .bind(db.prefix().schema_name("notes"))
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert!(!schema);
    assert_eq!(db.upgrade_history(1).await.unwrap()[0].operation, "uninstall");
    drop_installation(&db).await;
}

#[tokio::test]
async fn an_installation_older_than_the_oldest_supported_release_is_refused() {
    let db = installation().await;
    Upgrader::new(&db, &[notes(&[CREATE])], "0.1.0")
        .apply(true)
        .await
        .unwrap();
    let later = Upgrader::new(&db, &[notes(&[CREATE, TEXT])], "0.3.0").upgrading_from("0.2.0");
    assert!(later.plan().await.unwrap().problems.contains(&Problem::TooOld {
        recorded: "0.1.0".to_owned(),
        oldest: "0.2.0".to_owned(),
    }));
    assert_eq!(later.apply(false).await.unwrap_err().code(), "db.upgrade_refused");
    let next = Upgrader::new(&db, &[notes(&[CREATE, TEXT])], "0.2.0").upgrading_from("0.1.0");
    next.apply(false).await.unwrap();
    assert!(
        later.plan().await.unwrap().problems.is_empty(),
        "0.2.0 upgrades to 0.3.0"
    );
    drop_installation(&db).await;
}
