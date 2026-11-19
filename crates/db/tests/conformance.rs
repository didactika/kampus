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

//! The conformance suite of the database port against PostgreSQL: a module declared only with
//! the neutral schema is installed, and every type, key, reference behaviour, check, error class and
//! capability is exercised on it.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a failing setup must stop the test"
)]
#![allow(
    clippy::too_many_lines,
    reason = "each conformance scenario reads as one story"
)]

use kampus_common::ErrorKind;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};
use kampus_db::{Database, ModuleSteps, Prefix, Step, Upgrader, classify, lint};
use sqlx::Row;

/// The probe module: every portable type, a reserved-word table and every reference behaviour.
const PROBE: ModuleSteps = ModuleSteps {
    module: "probe",
    steps: &[
        Step {
            sequence: 1,
            name: "tables",
            revert: &[],
            action: None,
            changes: &[
                Change::CreateTable(Table {
                    name: "parent",
                    columns: &[
                        Column::new("id", Type::Uuid),
                        Column::new("code", Type::Varchar(20)),
                        Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                    ],
                    primary_key: &["id"],
                    unique: &[&["code"]],
                    ..Table::EMPTY
                }),
                Change::CreateTable(Table {
                    name: "group",
                    columns: &[
                        Column::new("id", Type::Uuid),
                        Column::new("parent_id", Type::Uuid),
                        Column::new("name", Type::Text),
                        Column::new("state", Type::Text).default(ColumnDefault::Text("active")),
                    ],
                    primary_key: &["id"],
                    unique: &[&["parent_id", "name"]],
                    foreign_keys: &[ForeignKey {
                        columns: &["parent_id"],
                        module: None,
                        table: "parent",
                        references: &["id"],
                        on_delete: OnDelete::Cascade,
                    }],
                    checks: &[Check::OneOf("state", &["active", "archived"])],
                    ..Table::EMPTY
                }),
                Change::CreateTable(Table {
                    name: "sample",
                    columns: &[
                        Column::new("id", Type::Uuid),
                        Column::new("owner_id", Type::Uuid),
                        Column::new("group_id", Type::Uuid).nullable(),
                        Column::new("label", Type::Text),
                        Column::new("count", Type::Integer),
                        Column::new("total", Type::BigInt),
                        Column::new("grade", Type::Numeric(12, 5)),
                        Column::new("enabled", Type::Boolean).default(ColumnDefault::Bool(true)),
                        Column::new("due_date", Type::Date),
                        Column::new("starts_at", Type::Timestamptz),
                        Column::new("ends_at", Type::Timestamptz).nullable(),
                        Column::new("duration", Type::Interval),
                        Column::new("digest", Type::Bytes),
                        Column::new("settings", Type::Json("probe.settings")),
                        Column::new("tags", Type::TextArray),
                    ],
                    primary_key: &["id"],
                    foreign_keys: &[
                        ForeignKey {
                            columns: &["owner_id"],
                            module: None,
                            table: "parent",
                            references: &["id"],
                            on_delete: OnDelete::Restrict,
                        },
                        ForeignKey {
                            columns: &["group_id"],
                            module: None,
                            table: "group",
                            references: &["id"],
                            on_delete: OnDelete::SetNull,
                        },
                    ],
                    checks: &[Check::Positive("count"), Check::After("ends_at", "starts_at")],
                    without_natural_key: Some("a probe of every type"),
                    ..Table::EMPTY
                }),
            ],
        },
        Step {
            sequence: 2,
            name: "expand",
            revert: &[],
            action: None,
            changes: &[
                Change::AddColumn {
                    table: "sample",
                    column: Column::new("note", Type::Text).default(ColumnDefault::Text("")),
                },
                Change::AddIndex {
                    table: "sample",
                    index: Index {
                        columns: &["starts_at"],
                        unique: false,
                    },
                },
            ],
        },
    ],
};

/// Installs the probe module in an installation of its own prefix.
async fn installation() -> Database {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("c{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    Upgrader::new(&db, &[PROBE], "0.1.0").apply(true).await.unwrap();
    db
}

/// Drops the schemas of a test installation.
async fn drop_installation(db: &Database) {
    for module in ["db", "probe"] {
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "drop schema if exists {} cascade",
            db.prefix().schema(module)
        )))
        .execute(db.pool())
        .await
        .unwrap();
    }
}

/// The quoted name of a probe table.
fn table(db: &Database, name: &str) -> String {
    format!("{}.\"{name}\"", db.prefix().schema("probe"))
}

/// Runs a statement and classifies its failure.
async fn run(db: &Database, sql: String, id: uuid::Uuid) -> kampus_common::Result<u64> {
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .execute(db.pool())
        .await
        .map(|r| r.rows_affected())
        .map_err(classify)
}

#[test]
fn the_probe_module_passes_the_linter() {
    assert_eq!(lint::lint(&[PROBE], "0.1.0"), []);
}

#[tokio::test]
async fn every_portable_type_round_trips() {
    let db = installation().await;
    let parent = uuid::Uuid::now_v7();
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "insert into {} (id, code) values ($1, 'p1')",
        table(&db, "parent")
    )))
    .bind(parent)
    .execute(db.pool())
    .await
    .unwrap();
    let id = uuid::Uuid::now_v7();
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "insert into {} (id, owner_id, label, count, total, grade, due_date, starts_at, ends_at, duration, digest, settings, tags)
         values ($1, $2, 'ñandú', 3, 9007199254740993, '87.12345', '2026-11-18', '2026-11-18T10:00:00Z', '2026-11-18T11:00:00Z',
                 '1 hour 30 minutes', $3, '{{\"a\": 1}}', $4)",
        table(&db, "sample")
    )))
    .bind(id)
    .bind(parent)
    .bind(vec![0_u8, 255, 7])
    .bind(vec!["one".to_owned(), "two".to_owned()])
    .execute(db.pool())
    .await
    .unwrap();

    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "select label, count, total, grade::text as grade, enabled, due_date::text as due_date, extract(epoch from ends_at - starts_at)::bigint as seconds,
                extract(epoch from duration)::bigint as duration, digest, settings->>'a' as a, tags, note from {} where id = $1",
        table(&db, "sample")
    )))
    .bind(id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>("label"), "ñandú");
    assert_eq!(row.get::<i32, _>("count"), 3);
    assert_eq!(row.get::<i64, _>("total"), 9_007_199_254_740_993);
    assert_eq!(row.get::<String, _>("grade"), "87.12345");
    assert!(row.get::<bool, _>("enabled"));
    assert_eq!(row.get::<String, _>("due_date"), "2026-11-18");
    assert_eq!(row.get::<i64, _>("seconds"), 3600);
    assert_eq!(row.get::<i64, _>("duration"), 5400);
    assert_eq!(row.get::<Vec<u8>, _>("digest"), vec![0, 255, 7]);
    assert_eq!(row.get::<String, _>("a"), "1");
    assert_eq!(row.get::<Vec<String>, _>("tags"), ["one", "two"]);
    assert_eq!(
        row.get::<String, _>("note"),
        "",
        "a column added later has its default on existing rows"
    );
    drop_installation(&db).await;
}

#[tokio::test]
async fn constraints_hold_and_their_failures_are_classified() {
    let db = installation().await;
    let (parent, group, sample) = (uuid::Uuid::now_v7(), uuid::Uuid::now_v7(), uuid::Uuid::now_v7());
    let parent_table = table(&db, "parent");
    let group_table = table(&db, "group");
    let sample_table = table(&db, "sample");
    run(
        &db,
        format!("insert into {parent_table} (id, code) values ($1, 'p1')"),
        parent,
    )
    .await
    .unwrap();

    let duplicate = run(
        &db,
        format!("insert into {parent_table} (id, code) values ($1, 'p1')"),
        uuid::Uuid::now_v7(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        (duplicate.kind(), duplicate.code()),
        (ErrorKind::Conflict, "db.unique_violation")
    );

    run(
        &db,
        format!("insert into {group_table} (id, parent_id, name) values ($1, '{parent}', 'g1')"),
        group,
    )
    .await
    .unwrap();
    let state = run(
        &db,
        format!(
            "insert into {group_table} (id, parent_id, name, state) values ($1, '{parent}', 'g2', 'gone')"
        ),
        uuid::Uuid::now_v7(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        (state.kind(), state.code()),
        (ErrorKind::Validation, "db.check_violation")
    );
    assert_eq!(state.fields()[0].field, "group_state_check");

    let values = "3, 1, '1', '2026-11-18', '2026-11-18T10:00:00Z', null, '1 hour', '\\x00', '{}', '{}'";
    let columns = "id, owner_id, group_id, label, count, total, grade, due_date, starts_at, ends_at, duration, digest, settings, tags";
    run(
        &db,
        format!("insert into {sample_table} ({columns}) values ($1, '{parent}', '{group}', 's', {values})"),
        sample,
    )
    .await
    .unwrap();

    let missing = run(
        &db,
        format!("insert into {sample_table} (id, owner_id) values ($1, '{parent}')"),
        uuid::Uuid::now_v7(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        (missing.kind(), missing.code()),
        (ErrorKind::Validation, "db.not_null_violation")
    );

    let window = run(
        &db,
        format!("insert into {sample_table} ({columns}) values ($1, '{parent}', null, 's', 3, 1, '1', '2026-11-18', '2026-11-18T10:00:00Z', '2026-11-18T09:00:00Z', '1 hour', '\\x00', '{{}}', '{{}}')"),
        uuid::Uuid::now_v7(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        window.code(),
        "db.check_violation",
        "an end before the start is refused"
    );

    let restricted = run(&db, format!("delete from {parent_table} where id = $1"), parent)
        .await
        .unwrap_err();
    assert_eq!(
        (restricted.kind(), restricted.code()),
        (ErrorKind::Conflict, "db.foreign_key_violation")
    );

    assert_eq!(
        run(&db, format!("delete from {group_table} where id = $1"), group)
            .await
            .unwrap(),
        1
    );
    let link: Option<uuid::Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select group_id from {sample_table} where id = $1"
    )))
    .bind(sample)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(link, None, "an optional link is cleared when its target goes");

    run(
        &db,
        format!("insert into {group_table} (id, parent_id, name) values ($1, '{parent}', 'g3')"),
        uuid::Uuid::now_v7(),
    )
    .await
    .unwrap();
    run(
        &db,
        format!("delete from {sample_table} where owner_id = $1"),
        parent,
    )
    .await
    .unwrap();
    assert_eq!(
        run(&db, format!("delete from {parent_table} where id = $1"), parent)
            .await
            .unwrap(),
        1
    );
    let groups: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("select count(*) from {group_table}")))
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(groups, 0, "owned children go with their parent");
    drop_installation(&db).await;
}

#[tokio::test]
async fn locked_rows_can_be_skipped() {
    let db = installation().await;
    assert!(db.capabilities().skip_locked);
    let parent = uuid::Uuid::now_v7();
    let parent_table = table(&db, "parent");
    run(
        &db,
        format!("insert into {parent_table} (id, code) values ($1, 'p1')"),
        parent,
    )
    .await
    .unwrap();

    let mut first = db.begin().await.unwrap();
    let claimed: Vec<uuid::Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select id from {parent_table} for update skip locked"
    )))
    .fetch_all(&mut *first)
    .await
    .unwrap();
    assert_eq!(claimed, [parent]);
    let mut second = db.begin().await.unwrap();
    let skipped: Vec<uuid::Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select id from {parent_table} for update skip locked"
    )))
    .fetch_all(&mut *second)
    .await
    .unwrap();
    assert_eq!(
        skipped.len(),
        0,
        "a row locked by another transaction is skipped, not waited for"
    );
    second.rollback().await.unwrap();
    first.rollback().await.unwrap();
    drop_installation(&db).await;
}
