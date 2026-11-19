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
// @package   kampus-tasks
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `tasks` schema, in the neutral schema: job types declared by modules, the
//! durable queue of jobs, schedules with their unique ticks, and the attempts of every job.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The states of a job.
pub const STATES: &[&str] = &["queued", "running", "succeeded", "failed", "cancelled"];

/// The outcomes of an attempt.
pub const OUTCOMES: &[&str] = &["succeeded", "failed", "timed_out", "lost", "cancelled"];

/// A foreign key to a table of this module.
const fn own(
    columns: &'static [&'static str],
    table: &'static str,
    references: &'static [&'static str],
    on_delete: OnDelete,
) -> ForeignKey {
    ForeignKey {
        columns,
        module: None,
        table,
        references,
        on_delete,
    }
}

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "job types, jobs, schedules and attempts",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "job_type",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("max_attempts", Type::Integer),
                Column::new("backoff_seconds", Type::Integer),
                Column::new("timeout_seconds", Type::Integer),
                Column::new("concurrency", Type::Text).nullable(),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            checks: &[
                Check::Positive("max_attempts"),
                Check::NonNegative("backoff_seconds"),
                Check::Positive("timeout_seconds"),
            ],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "job",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("type_code", Type::Text),
                Column::new("payload", Type::Json("tasks.job_payload")),
                Column::new("state", Type::Text).default(ColumnDefault::Text("queued")),
                Column::new("attempts", Type::Integer).default(ColumnDefault::Int(0)),
                Column::new("available_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("lease_until", Type::Timestamptz).nullable(),
                Column::new("worker", Type::Text).nullable(),
                Column::new("concurrency_key", Type::Text).nullable(),
                Column::new("progress", Type::Integer).default(ColumnDefault::Int(0)),
                Column::new("cancel_requested", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("last_error", Type::Text).nullable(),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("finished_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            foreign_keys: &[own(&["type_code"], "job_type", &["code"], OnDelete::Restrict)],
            checks: &[
                Check::OneOf("state", STATES),
                Check::NonNegative("attempts"),
                Check::NonNegative("progress"),
            ],
            indexes: &[Index {
                columns: &["type_code", "state"],
                unique: false,
            }],
            without_natural_key: Some("one row per unit of work, identified by its id"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "schedule",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("type_code", Type::Text),
                Column::new("cron", Type::Text),
                Column::new("time_zone", Type::Text),
                Column::new("enabled", Type::Boolean).default(ColumnDefault::Bool(true)),
                Column::new("next_run_at", Type::Timestamptz),
                Column::new("last_run_at", Type::Timestamptz).nullable(),
                Column::new("changed_by", Type::Text).nullable(),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["id"],
            unique: &[&["type_code"]],
            foreign_keys: &[own(&["type_code"], "job_type", &["code"], OnDelete::Cascade)],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "schedule_tick",
            columns: &[
                Column::new("schedule_id", Type::Uuid),
                Column::new("at", Type::Timestamptz),
                Column::new("job_id", Type::Uuid).nullable(),
            ],
            primary_key: &["schedule_id", "at"],
            foreign_keys: &[
                own(&["schedule_id"], "schedule", &["id"], OnDelete::Cascade),
                own(&["job_id"], "job", &["id"], OnDelete::SetNull),
            ],
            without_natural_key: Some("one tick per schedule and instant, so a run is enqueued once"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "attempt",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("job_id", Type::Uuid),
                Column::new("worker", Type::Text),
                Column::new("started_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("ended_at", Type::Timestamptz).nullable(),
                Column::new("outcome", Type::Text).nullable(),
                Column::new("error", Type::Text).nullable(),
            ],
            primary_key: &["id"],
            foreign_keys: &[own(&["job_id"], "job", &["id"], OnDelete::Cascade)],
            checks: &[Check::OneOf("outcome", OUTCOMES)],
            indexes: &[Index {
                columns: &["job_id"],
                unique: false,
            }],
            without_natural_key: Some("one row per execution of a job"),
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "create index \"job_queued_available_at_index\" on {schema}.\"job\" (\"available_at\") where \"state\" = 'queued';
create unique index \"job_running_concurrency_key\" on {schema}.\"job\" (\"concurrency_key\") where \"state\" = 'running';
alter table {schema}.\"job\" add constraint \"job_progress_at_most_check\" check (\"progress\" <= 100);",
            reason: "the neutral schema has no partial index and no upper bound",
        },
    ],
}];
