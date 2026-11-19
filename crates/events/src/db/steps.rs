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

//! The upgrade steps of the `events` schema, in the neutral schema: event types and listeners declared by
//! modules, the events with their outbox, the deliveries of each event to each listener, and the
//! append-only audit log.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The states of a delivery.
pub const STATES: &[&str] = &["pending", "done", "dead"];

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
    name: "event types, events, outbox, listeners, deliveries and the audit log",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "event_type",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("version", Type::Integer),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("fields", Type::Json("events.type_fields")),
                Column::new("audited", Type::Boolean),
                Column::new("personal_fields", Type::Json("events.personal_fields")),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            checks: &[Check::Positive("version")],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "event",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("type_code", Type::Text),
                Column::new("type_version", Type::Integer),
                Column::new("scope", Type::Uuid).nullable(),
                Column::new("actor", Type::Uuid).nullable(),
                Column::new("object", Type::Uuid).nullable(),
                Column::new("payload", Type::Json("events.payload")),
                Column::new("depth", Type::Integer).default(ColumnDefault::Int(0)),
                Column::new("request", Type::Text).nullable(),
                Column::new("occurred_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[own(&["type_code"], "event_type", &["code"], OnDelete::Restrict)],
            checks: &[Check::NonNegative("depth")],
            indexes: &[
                Index {
                    columns: &["object", "occurred_at"],
                    unique: false,
                },
                Index {
                    columns: &["occurred_at"],
                    unique: false,
                },
            ],
            without_natural_key: Some("one row per occurrence, identified by its id"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "outbox",
            columns: &[
                Column::new("event_id", Type::Uuid),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("dispatched_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["event_id"],
            foreign_keys: &[own(&["event_id"], "event", &["id"], OnDelete::Cascade)],
            without_natural_key: Some("one entry per event waiting for its dispatch"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "listener",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("module", Type::Text),
                Column::new("type_code", Type::Text),
                Column::new("name", Type::Text),
                Column::new("enabled", Type::Boolean).default(ColumnDefault::Bool(true)),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["module", "type_code", "name"]],
            foreign_keys: &[own(&["type_code"], "event_type", &["code"], OnDelete::Cascade)],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "delivery",
            columns: &[
                Column::new("listener_id", Type::Uuid),
                Column::new("event_id", Type::Uuid),
                Column::new("object", Type::Uuid).nullable(),
                Column::new("occurred_at", Type::Timestamptz),
                Column::new("state", Type::Text).default(ColumnDefault::Text("pending")),
                Column::new("attempts", Type::Integer).default(ColumnDefault::Int(0)),
                Column::new("next_attempt_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("lease_until", Type::Timestamptz).nullable(),
                Column::new("last_error", Type::Text).nullable(),
                Column::new("done_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["listener_id", "event_id"],
            foreign_keys: &[
                own(&["listener_id"], "listener", &["id"], OnDelete::Cascade),
                own(&["event_id"], "event", &["id"], OnDelete::Cascade),
            ],
            checks: &[Check::OneOf("state", STATES), Check::NonNegative("attempts")],
            indexes: &[
                Index {
                    columns: &["state", "next_attempt_at"],
                    unique: false,
                },
                Index {
                    columns: &["listener_id", "object", "occurred_at"],
                    unique: false,
                },
            ],
            without_natural_key: Some("one delivery per listener and event"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "audit_entry",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("event_id", Type::Uuid),
                Column::new("campus", Type::Uuid).nullable(),
                Column::new("scope", Type::Uuid).nullable(),
                Column::new("actor", Type::Uuid).nullable(),
                Column::new("object", Type::Uuid).nullable(),
                Column::new("summary", Type::Text),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["event_id"]],
            foreign_keys: &[own(&["event_id"], "event", &["id"], OnDelete::Restrict)],
            indexes: &[
                Index {
                    columns: &["scope", "at"],
                    unique: false,
                },
                Index {
                    columns: &["actor", "at"],
                    unique: false,
                },
            ],
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "create index \"outbox_waiting_index\" on {schema}.\"outbox\" (\"created_at\") where \"dispatched_at\" is null;
create function {schema}.\"audit_entry_append_only\"() returns trigger language plpgsql as $$
begin
  if tg_op = 'UPDATE' then
    raise exception 'audit entries are append-only';
  end if;
  if current_setting('kampus.audit_retention', true) is distinct from 'on' then
    raise exception 'audit entries are removed only by their retention';
  end if;
  return old;
end $$;
create trigger \"audit_entry_append_only\" before update or delete on {schema}.\"audit_entry\"
  for each row execute function {schema}.\"audit_entry_append_only\"();",
            reason: "the neutral schema has no partial index and no trigger; the audit log refuses updates and deletes outside its retention",
        },
    ],
}];
