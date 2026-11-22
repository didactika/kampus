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

//! The upgrade runner against PostgreSQL: fresh installations, upgrades from any earlier state in one run,
//! refused databases and concurrent upgrades. Each test runs in an installation of its own prefix.

use kampus_common::ErrorKind;
use kampus_db::schema::Change;
use kampus_db::{Database, ModuleSteps, Prefix, Problem, Step, Upgrader};

/// The first step of a sample module.
const FIRST: Step = Step {
    sequence: 1,
    name: "create note",
    revert: &[],
    action: None,
    changes: &[Change::Postgres {
        sql: "create table {schema}.note (id uuid primary key);",
        reason: "a sample module of the tests",
    }],
};

/// The second step of a sample module.
const SECOND: Step = Step {
    sequence: 2,
    name: "add note text",
    revert: &[],
    action: None,
    changes: &[Change::Postgres {
        sql: "alter table {schema}.note add column body text;",
        reason: "a sample module of the tests",
    }],
};

/// The third step of a sample module, referencing another module's schema.
const THIRD: Step = Step {
    sequence: 3,
    name: "add note author",
    revert: &[],
    action: None,
    changes: &[Change::Postgres {
        sql: "alter table {schema}.note add column author_id uuid references {schema:people}.person (id);",
        reason: "a sample module of the tests",
    }],
};

/// The first step as it would look if someone edited it after release.
const FIRST_EDITED: Step = Step {
    sequence: 1,
    name: "create note",
    revert: &[],
    action: None,
    changes: &[Change::Postgres {
        sql: "create table {schema}.note (id uuid);",
        reason: "a sample module of the tests",
    }],
};

/// A module the sample module depends on.
const PEOPLE: ModuleSteps = ModuleSteps {
    module: "people",
    steps: &[Step {
        sequence: 1,
        name: "create person",
        revert: &[],
        action: None,
        changes: &[Change::Postgres {
            sql: "create table {schema}.person (id uuid primary key);",
            reason: "a sample module of the tests",
        }],
    }],
};

/// The sample module with one released step.
const NOTES_V1: ModuleSteps = ModuleSteps {
    module: "notes",
    steps: &[FIRST],
};

/// The sample module three releases later.
const NOTES_V3: ModuleSteps = ModuleSteps {
    module: "notes",
    steps: &[FIRST, SECOND, THIRD],
};

/// Connects to a fresh installation with a prefix of its own.
async fn installation() -> Database {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("t{}_", &simple[simple.len() - 6..]).parse().unwrap();
    Database::connect(&url, prefix, 4).await.unwrap()
}

/// Drops the schemas of a test installation.
async fn drop_installation(db: &Database) {
    for module in ["db", "people", "notes"] {
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "drop schema if exists {} cascade",
            db.prefix().schema(module)
        )))
        .execute(db.pool())
        .await
        .unwrap();
    }
    db.close().await;
}

#[tokio::test]
async fn a_fresh_installation_applies_every_step_once() {
    let db = installation().await;
    let upgrader = Upgrader::new(&db, &[PEOPLE, NOTES_V3], "0.1.0");
    let plan = upgrader.plan().await.unwrap();
    assert!(!plan.installed);
    assert_eq!(plan.pending.len(), 7);

    let report = upgrader.apply(true).await.unwrap();
    assert!(report.installed);
    assert_eq!(report.applied.len(), 7);
    assert!(upgrader.plan().await.unwrap().is_current());

    let again = upgrader.apply(false).await.unwrap();
    assert_eq!(again.applied.len(), 0);
    drop_installation(&db).await;
}

#[tokio::test]
async fn an_old_installation_reaches_the_latest_steps_in_one_run() {
    let db = installation().await;
    Upgrader::new(&db, &[PEOPLE, NOTES_V1], "0.1.0")
        .apply(true)
        .await
        .unwrap();

    let latest = Upgrader::new(&db, &[PEOPLE, NOTES_V3], "0.3.0");
    let report = latest.apply(false).await.unwrap();
    let applied: Vec<_> = report.applied.iter().map(|s| (s.module, s.sequence)).collect();
    assert_eq!(applied, [("notes", 2), ("notes", 3)]);
    assert!(latest.plan().await.unwrap().is_current());

    let release: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select upgraded_version from {}.installation",
        db.prefix().schema("db")
    )))
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(release, "0.3.0");
    drop_installation(&db).await;
}

#[tokio::test]
async fn a_step_changed_after_release_is_refused() {
    let db = installation().await;
    Upgrader::new(&db, &[PEOPLE, NOTES_V1], "0.1.0")
        .apply(true)
        .await
        .unwrap();

    let edited = Upgrader::new(
        &db,
        &[
            PEOPLE,
            ModuleSteps {
                module: "notes",
                steps: &[FIRST_EDITED],
            },
        ],
        "0.1.1",
    );
    let plan = edited.plan().await.unwrap();
    assert_eq!(
        plan.problems,
        [Problem::ChangedStep {
            module: "notes".into(),
            sequence: 1
        }]
    );
    assert!(!plan.is_current());
    let error = edited.apply(false).await.unwrap_err();
    assert_eq!(
        (error.kind(), error.code()),
        (ErrorKind::Conflict, "db.upgrade_refused")
    );
    drop_installation(&db).await;
}

#[tokio::test]
async fn code_older_than_the_database_is_refused() {
    let db = installation().await;
    Upgrader::new(&db, &[PEOPLE, NOTES_V3], "0.3.0")
        .apply(true)
        .await
        .unwrap();

    let older = Upgrader::new(&db, &[PEOPLE, NOTES_V1], "0.1.0");
    let problems = older.plan().await.unwrap().problems;
    assert!(problems.contains(&Problem::UnknownStep {
        module: "notes".into(),
        sequence: 2
    }));
    assert!(problems.contains(&Problem::NewerRelease {
        recorded: "0.3.0".into()
    }));
    assert!(older.apply(false).await.is_err());
    drop_installation(&db).await;
}

#[tokio::test]
async fn a_new_installation_next_to_another_one_must_be_asked_for() {
    let first = installation().await;
    Upgrader::new(&first, &[PEOPLE], "0.1.0")
        .apply(true)
        .await
        .unwrap();

    let second = installation().await;
    let upgrader = Upgrader::new(&second, &[PEOPLE], "0.1.0");
    assert!(
        upgrader
            .plan()
            .await
            .unwrap()
            .problems
            .iter()
            .any(|p| matches!(p, Problem::OtherInstallations { .. }))
    );
    assert_eq!(
        upgrader.apply(false).await.unwrap_err().code(),
        "db.other_installations"
    );
    assert!(upgrader.apply(true).await.unwrap().installed);
    drop_installation(&first).await;
    drop_installation(&second).await;
}

#[tokio::test]
async fn concurrent_upgrades_apply_each_step_once() {
    let db = installation().await;
    let (a, b) = tokio::join!(
        async { Upgrader::new(&db, &[PEOPLE, NOTES_V3], "0.1.0").apply(true).await },
        async { Upgrader::new(&db, &[PEOPLE, NOTES_V3], "0.1.0").apply(true).await },
    );
    let total = a.unwrap().applied.len() + b.unwrap().applied.len();
    assert_eq!(total, 7);
    let rows: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {}.migration",
        db.prefix().schema("db")
    )))
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(rows, 7);
    drop_installation(&db).await;
}

#[tokio::test]
async fn modules_must_declare_their_steps_in_order() {
    let db = installation().await;
    let gap = ModuleSteps {
        module: "notes",
        steps: &[FIRST, THIRD],
    };
    let error = Upgrader::new(&db, &[gap], "0.1.0").plan().await.unwrap_err();
    assert_eq!(
        (error.kind(), error.code()),
        (ErrorKind::Internal, "db.steps_invalid")
    );
}
