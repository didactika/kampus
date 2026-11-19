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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `enrol` schema, in the neutral schema. An enrolment owns the grants it gives,
//! so removing it removes exactly what it gave.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The statuses of an enrolment.
pub const STATUSES: &[&str] = &["active", "suspended"];

/// What happens when an enrolment ends.
pub const EXPIRY_ACTIONS: &[&str] = &["suspend", "remove", "keep_without_role"];

/// The states of a request to join.
pub const REQUEST_STATES: &[&str] = &["pending", "approved", "rejected", "withdrawn"];

/// A foreign key to a table of another module.
const fn other(
    columns: &'static [&'static str],
    module: &'static str,
    table: &'static str,
    references: &'static [&'static str],
    on_delete: OnDelete,
) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some(module),
        table,
        references,
        on_delete,
    }
}

/// A foreign key to a table of this module, which takes the row with it.
const fn own(columns: &'static [&'static str], table: &'static str) -> ForeignKey {
    ForeignKey {
        columns,
        module: None,
        table,
        references: &["id"],
        on_delete: OnDelete::Cascade,
    }
}

/// A foreign key to an account, which takes the row with it.
const ACCOUNT: ForeignKey = other(&["account_id"], "identity", "account", &["id"], OnDelete::Cascade);

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "method instances, enrolments and requests",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "instance",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_id", Type::Uuid),
                Column::new("method_code", Type::Text),
                Column::new("name", Type::Text),
                Column::new("role_id", Type::Uuid),
                Column::new("enabled", Type::Boolean).default(ColumnDefault::Bool(true)),
                Column::new("open_from", Type::Timestamptz).nullable(),
                Column::new("open_until", Type::Timestamptz).nullable(),
                Column::new("default_days", Type::Integer).nullable(),
                Column::new("capacity", Type::Integer).nullable(),
                Column::new("requires_approval", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("expiry_action", Type::Text).default(ColumnDefault::Text("suspend")),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                other(&["course_id"], "courses", "course", &["id"], OnDelete::Cascade),
                other(&["role_id"], "access", "role", &["id"], OnDelete::Restrict),
            ],
            checks: &[
                Check::After("open_until", "open_from"),
                Check::Positive("capacity"),
                Check::Positive("default_days"),
                Check::OneOf("expiry_action", EXPIRY_ACTIONS),
            ],
            indexes: &[Index {
                columns: &["course_id"],
                unique: false,
            }],
            without_natural_key: Some("named per course, unique by an index on the lower-case name"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "self_key",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("instance_id", Type::Uuid),
                Column::new("group_id", Type::Uuid).nullable(),
                Column::new("hash", Type::Text),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                own(&["instance_id"], "instance"),
                other(&["group_id"], "courses", "group", &["id"], OnDelete::Cascade),
            ],
            without_natural_key: Some("one key per instance and group, unique by an index where nulls are equal"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "linked_group",
            columns: &[
                Column::new("instance_id", Type::Uuid),
                Column::new("system_group_id", Type::Uuid),
            ],
            primary_key: &["instance_id"],
            foreign_keys: &[
                own(&["instance_id"], "instance"),
                other(&["system_group_id"], "users", "system_group", &["id"], OnDelete::Cascade),
            ],
            without_natural_key: Some("one source per instance"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "enrolment",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("instance_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("status", Type::Text).default(ColumnDefault::Text("active")),
                Column::new("starts_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("ends_at", Type::Timestamptz).nullable(),
                Column::new("expired_at", Type::Timestamptz).nullable(),
                Column::new("created_by", Type::Uuid).nullable(),
                Column::new("reason", Type::Text).nullable(),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["instance_id", "account_id"]],
            foreign_keys: &[own(&["instance_id"], "instance"), ACCOUNT],
            checks: &[Check::OneOf("status", STATUSES), Check::After("ends_at", "starts_at")],
            indexes: &[
                Index {
                    columns: &["account_id", "status"],
                    unique: false,
                },
                Index {
                    columns: &["instance_id", "status"],
                    unique: false,
                },
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "enrolment_grant",
            columns: &[
                Column::new("enrolment_id", Type::Uuid),
                Column::new("grant_id", Type::Uuid),
            ],
            primary_key: &["enrolment_id", "grant_id"],
            foreign_keys: &[
                own(&["enrolment_id"], "enrolment"),
                other(&["grant_id"], "access", "grant", &["id"], OnDelete::Cascade),
            ],
            without_natural_key: Some("keyed by the enrolment and the grant it owns"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "enrolment_group_member",
            columns: &[
                Column::new("instance_id", Type::Uuid),
                Column::new("group_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
            ],
            primary_key: &["instance_id", "group_id", "account_id"],
            foreign_keys: &[
                own(&["instance_id"], "instance"),
                other(&["group_id"], "courses", "group", &["id"], OnDelete::Cascade),
                ACCOUNT,
            ],
            without_natural_key: Some("the group memberships an instance owns"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "request",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("instance_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("message", Type::Text).nullable(),
                Column::new("group_id", Type::Uuid).nullable(),
                Column::new("state", Type::Text).default(ColumnDefault::Text("pending")),
                Column::new("decided_by", Type::Uuid).nullable(),
                Column::new("decided_at", Type::Timestamptz).nullable(),
                Column::new("reason", Type::Text).nullable(),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                own(&["instance_id"], "instance"),
                ACCOUNT,
                other(&["group_id"], "courses", "group", &["id"], OnDelete::SetNull),
            ],
            checks: &[Check::OneOf("state", REQUEST_STATES)],
            without_natural_key: Some("one pending request per instance and account, by a partial index"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "expiry_outcome",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("enrolment_id", Type::Uuid).nullable(),
                Column::new("instance_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("action", Type::Text),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                ForeignKey {
                    columns: &["enrolment_id"],
                    module: None,
                    table: "enrolment",
                    references: &["id"],
                    on_delete: OnDelete::SetNull,
                },
                own(&["instance_id"], "instance"),
                ACCOUNT,
            ],
            checks: &[Check::OneOf("action", EXPIRY_ACTIONS)],
            without_natural_key: Some("an append-only record of what each expiry did"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "self_attempt",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("instance_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("address", Type::Text).nullable(),
                Column::new("succeeded", Type::Boolean),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[own(&["instance_id"], "instance"), ACCOUNT],
            indexes: &[Index {
                columns: &["instance_id", "account_id", "at"],
                unique: false,
            }],
            without_natural_key: Some("an append-only log of key attempts, for throttling"),
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "create unique index \"instance_course_name_key\" on {schema}.\"instance\" (\"course_id\", lower(\"name\"));
create unique index \"self_key_instance_group_key\" on {schema}.\"self_key\" (\"instance_id\", \"group_id\") nulls not distinct;
create unique index \"request_pending_key\" on {schema}.\"request\" (\"instance_id\", \"account_id\") where \"state\" = 'pending';
alter table {schema}.\"instance\" add constraint \"instance_method_code_check\" check (\"method_code\" ~ '^[a-z][a-z0-9_]*$');",
            reason: "the neutral schema has no unique index on an expression, a partial one, or one where nulls are equal, and no pattern",
        },
    ],
}];
