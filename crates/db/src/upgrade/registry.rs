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

//! The registry of an installation: every component with its installed version, and the history of
//! every upgrade, downgrade and uninstall, in order.

use std::collections::BTreeMap;

use kampus_common::{Result, Timestamp};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use crate::Database;
use crate::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The tables of the registry, created by the second step of this crate.
pub(crate) const TABLES: &[Change] = &[
    Change::CreateTable(Table {
        name: "component",
        columns: &[
            Column::new("name", Type::Text),
            Column::new("kind", Type::Text),
            Column::new("origin", Type::Text),
            Column::new("version", Type::Text),
            Column::new("maturity", Type::Text),
            Column::new("installed_at", Type::Timestamptz).default(ColumnDefault::Now),
            Column::new("upgraded_at", Type::Timestamptz).default(ColumnDefault::Now),
        ],
        primary_key: &["name"],
        checks: &[
            Check::OneOf("kind", &["module", "plugin", "adapter"]),
            Check::OneOf("origin", &["core", "external"]),
            Check::OneOf("maturity", &["alpha", "beta", "rc", "stable"]),
        ],
        without_natural_key: Some("keyed by the component name, its natural key"),
        ..Table::EMPTY
    }),
    Change::CreateTable(Table {
        name: "upgrade_run",
        columns: &[
            Column::new("id", Type::Uuid),
            Column::new("operation", Type::Text),
            Column::new("release_from", Type::Text).nullable(),
            Column::new("release_to", Type::Text),
            Column::new("started_at", Type::Timestamptz),
            Column::new("finished_at", Type::Timestamptz),
            Column::new("outcome", Type::Text),
            Column::new("error", Type::Text).nullable(),
        ],
        primary_key: &["id"],
        checks: &[
            Check::OneOf("operation", &["upgrade", "downgrade", "uninstall"]),
            Check::OneOf("outcome", &["succeeded", "failed"]),
            Check::After("finished_at", "started_at"),
        ],
        indexes: &[Index {
            columns: &["started_at"],
            unique: false,
        }],
        without_natural_key: Some("an append-only history of the runs"),
        ..Table::EMPTY
    }),
    Change::CreateTable(Table {
        name: "component_change",
        columns: &[
            Column::new("id", Type::Uuid),
            Column::new("run_id", Type::Uuid),
            Column::new("component", Type::Text),
            Column::new("version_from", Type::Text).nullable(),
            Column::new("version_to", Type::Text).nullable(),
            Column::new("steps", Type::Integer),
        ],
        primary_key: &["id"],
        foreign_keys: &[ForeignKey {
            columns: &["run_id"],
            module: None,
            table: "upgrade_run",
            references: &["id"],
            on_delete: OnDelete::Cascade,
        }],
        checks: &[Check::NonNegative("steps")],
        without_natural_key: Some("an append-only history of what each run did to each component"),
        ..Table::EMPTY
    }),
];

/// A component of the build, as the registry records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentRecord {
    /// Its name (`identity`, `cache_redis`).
    pub name: String,
    /// `platform`, `module`, `plugin` or `adapter`.
    pub kind: &'static str,
    /// `core` or `external`.
    pub origin: &'static str,
    /// Its version.
    pub version: String,
    /// `alpha`, `beta`, `rc` or `stable`.
    pub maturity: &'static str,
    /// The schema its steps create, when it has one.
    pub schema: Option<&'static str>,
}

/// A component as installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledComponent {
    /// Its name.
    pub name: String,
    /// `module`, `plugin` or `adapter`.
    pub kind: String,
    /// `core` or `external`.
    pub origin: String,
    /// The installed version.
    pub version: String,
    /// Its maturity when it was installed or upgraded.
    pub maturity: String,
    /// When it was installed.
    pub installed_at: Timestamp,
    /// When its version last changed.
    pub upgraded_at: Timestamp,
}

/// What one run did to one component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentChange {
    /// The component.
    pub component: String,
    /// The version before, absent when the run installed it.
    pub from: Option<String>,
    /// The version after, absent when the run uninstalled it.
    pub to: Option<String>,
    /// How many steps of it the run applied or reverted.
    pub steps: i32,
}

/// One upgrade, downgrade or uninstall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// `upgrade`, `downgrade` or `uninstall`.
    pub operation: String,
    /// The release before, absent for a new installation.
    pub release_from: Option<String>,
    /// The release after.
    pub release_to: String,
    /// When it started.
    pub started_at: Timestamp,
    /// When it finished.
    pub finished_at: Timestamp,
    /// `succeeded` or `failed`.
    pub outcome: String,
    /// Why it failed.
    pub error: Option<String>,
    /// What it did to each component.
    pub changes: Vec<ComponentChange>,
}

/// Marks SQL of the registry as audited: only the quoted ledger schema is interpolated.
fn audited(sql: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(sql)
}

/// Whether the registry exists in an installation.
pub(crate) async fn exists(conn: &mut PgConnection, ledger: &str) -> Result<bool> {
    sqlx::query_scalar("select to_regclass($1) is not null")
        .bind(format!("{ledger}.upgrade_run"))
        .fetch_one(conn)
        .await
        .map_err(crate::classify)
}

/// The installed version of every component.
pub(crate) async fn versions(conn: &mut PgConnection, ledger: &str) -> Result<BTreeMap<String, String>> {
    if !exists(conn, ledger).await? {
        return Ok(BTreeMap::new());
    }
    let rows = sqlx::query(audited(format!("select name, version from {ledger}.component")))
        .fetch_all(conn)
        .await
        .map_err(crate::classify)?;
    Ok(rows.iter().map(|r| (r.get("name"), r.get("version"))).collect())
}

/// Records a finished run with what it did, and brings the components up to date.
pub(crate) async fn record(
    conn: &mut PgConnection,
    ledger: &str,
    run: &Run,
    components: &[ComponentRecord],
) -> Result<()> {
    let id = uuid::Uuid::now_v7();
    sqlx::query(audited(format!(
        "insert into {ledger}.upgrade_run (id, operation, release_from, release_to, started_at, finished_at, outcome, error)
         values ($1, $2, $3, $4, $5, $6, $7, $8)"
    )))
    .bind(id)
    .bind(&run.operation)
    .bind(&run.release_from)
    .bind(&run.release_to)
    .bind(run.started_at.as_offset_date_time())
    .bind(run.finished_at.as_offset_date_time())
    .bind(&run.outcome)
    .bind(&run.error)
    .execute(&mut *conn)
    .await
    .map_err(crate::classify)?;
    for change in &run.changes {
        sqlx::query(audited(format!(
            "insert into {ledger}.component_change (id, run_id, component, version_from, version_to, steps)
             values ($1, $2, $3, $4, $5, $6)"
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(id)
        .bind(&change.component)
        .bind(&change.from)
        .bind(&change.to)
        .bind(change.steps)
        .execute(&mut *conn)
        .await
        .map_err(crate::classify)?;
        if run.outcome != "succeeded" {
            continue;
        }
        match components.iter().find(|c| c.name == change.component) {
            Some(component) if change.to.is_some() => {
                sqlx::query(audited(format!(
                    "insert into {ledger}.component (name, kind, origin, version, maturity, installed_at, upgraded_at)
                     values ($1, $2, $3, $4, $5, $6, $6)
                     on conflict (name) do update set kind = excluded.kind, origin = excluded.origin,
                     version = excluded.version, maturity = excluded.maturity, upgraded_at = excluded.upgraded_at"
                )))
                .bind(&component.name)
                .bind(component.kind)
                .bind(component.origin)
                .bind(&component.version)
                .bind(component.maturity)
                .bind(run.finished_at.as_offset_date_time())
                .execute(&mut *conn)
                .await
                .map_err(crate::classify)?;
            }
            _ => {
                if change.to.is_none() {
                    sqlx::query(audited(format!("delete from {ledger}.component where name = $1")))
                        .bind(&change.component)
                        .execute(&mut *conn)
                        .await
                        .map_err(crate::classify)?;
                }
            }
        }
    }
    Ok(())
}

impl Database {
    /// The components installed, by name; empty before the registry exists.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn installed_components(&self) -> Result<Vec<InstalledComponent>> {
        let ledger = self.prefix().schema(crate::MODULE);
        let mut conn = self.pool().acquire().await.map_err(crate::classify)?;
        if !exists(&mut conn, &ledger).await? {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(audited(format!(
            "select name, kind, origin, version, maturity, installed_at, upgraded_at from {ledger}.component order by name"
        )))
        .fetch_all(&mut *conn)
        .await
        .map_err(crate::classify)?;
        Ok(rows
            .iter()
            .map(|r| InstalledComponent {
                name: r.get("name"),
                kind: r.get("kind"),
                origin: r.get("origin"),
                version: r.get("version"),
                maturity: r.get("maturity"),
                installed_at: Timestamp::from_offset_date_time(r.get("installed_at")),
                upgraded_at: Timestamp::from_offset_date_time(r.get("upgraded_at")),
            })
            .collect())
    }

    /// The latest runs, newest first, with what each did.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn upgrade_history(&self, limit: i64) -> Result<Vec<Run>> {
        let ledger = self.prefix().schema(crate::MODULE);
        let mut conn = self.pool().acquire().await.map_err(crate::classify)?;
        if !exists(&mut conn, &ledger).await? {
            return Ok(Vec::new());
        }
        let runs = sqlx::query(audited(format!(
            "select id, operation, release_from, release_to, started_at, finished_at, outcome, error
             from {ledger}.upgrade_run order by started_at desc, id desc limit $1"
        )))
        .bind(limit)
        .fetch_all(&mut *conn)
        .await
        .map_err(crate::classify)?;
        let mut history = Vec::with_capacity(runs.len());
        for row in &runs {
            let id: uuid::Uuid = row.get("id");
            let changes = sqlx::query(audited(format!(
                "select component, version_from, version_to, steps from {ledger}.component_change
                 where run_id = $1 order by id"
            )))
            .bind(id)
            .fetch_all(&mut *conn)
            .await
            .map_err(crate::classify)?;
            history.push(run_of(row, changes.iter().map(change_of).collect()));
        }
        Ok(history)
    }
}

/// A run from its row.
fn run_of(row: &PgRow, changes: Vec<ComponentChange>) -> Run {
    Run {
        operation: row.get("operation"),
        release_from: row.get("release_from"),
        release_to: row.get("release_to"),
        started_at: Timestamp::from_offset_date_time(row.get("started_at")),
        finished_at: Timestamp::from_offset_date_time(row.get("finished_at")),
        outcome: row.get("outcome"),
        error: row.get("error"),
        changes,
    }
}

/// A change from its row.
fn change_of(row: &PgRow) -> ComponentChange {
    ComponentChange {
        component: row.get("component"),
        from: row.get("version_from"),
        to: row.get("version_to"),
        steps: row.get("steps"),
    }
}
