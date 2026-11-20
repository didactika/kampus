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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `progress` schema: what each course module requires, what people have done,
//! their state on each course module, the exceptions teachers set, and the history of every change.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// What a course module can require.
pub const RULE_KINDS: &[&str] = &["manual", "view", "submit", "grade", "pass"];

/// What a person can have done.
pub const SIGNAL_KINDS: &[&str] = &["view", "submit", "manual"];

/// The states of a person on a course module.
pub const STATES: &[&str] = &["not_started", "in_progress", "done", "done_passed", "done_failed"];

/// Where a state came from.
pub const SOURCES: &[&str] = &["automatic", "exception"];

/// A foreign key to a table of another module, by its `id`.
const fn other(
    columns: &'static [&'static str],
    module: &'static str,
    table: &'static str,
    on_delete: OnDelete,
) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some(module),
        table,
        references: &["id"],
        on_delete,
    }
}

/// The course module and the person of a row, with the checks they share.
const fn module_and_person() -> [ForeignKey; 2] {
    [
        other(
            &["course_module_id"],
            "activities",
            "course_module",
            OnDelete::Cascade,
        ),
        other(&["account_id"], "identity", "account", OnDelete::Cascade),
    ]
}

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "rules, signals, states, exceptions, history and the queue of rule changes",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "rule",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_module_id", Type::Uuid),
                Column::new("kind", Type::Text),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["course_module_id", "kind"]],
            foreign_keys: &[other(
                &["course_module_id"],
                "activities",
                "course_module",
                OnDelete::Cascade,
            )],
            checks: &[Check::OneOf("kind", RULE_KINDS)],
            without_natural_key: Some("a rule is named by its course module and kind"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "signal",
            columns: &[
                Column::new("course_module_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("kind", Type::Text),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["course_module_id", "account_id", "kind"],
            foreign_keys: &module_and_person(),
            checks: &[Check::OneOf("kind", SIGNAL_KINDS)],
            indexes: &[Index {
                columns: &["account_id"],
                unique: false,
            }],
            without_natural_key: Some("what a person did on a course module, once per kind"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "activity_state",
            columns: &[
                Column::new("course_module_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("state", Type::Text),
                Column::new("source", Type::Text),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["course_module_id", "account_id"],
            foreign_keys: &module_and_person(),
            checks: &[Check::OneOf("state", STATES), Check::OneOf("source", SOURCES)],
            indexes: &[Index {
                columns: &["account_id"],
                unique: false,
            }],
            without_natural_key: Some("the state of a person on a course module"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "activity_exception",
            columns: &[
                Column::new("course_module_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("state", Type::Text),
                Column::new("reason", Type::Text),
                Column::new("author", Type::Uuid).nullable(),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["course_module_id", "account_id"],
            foreign_keys: &module_and_person(),
            checks: &[Check::OneOf("state", STATES)],
            without_natural_key: Some("the decision of a teacher about a person on a course module"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "activity_history",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_module_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("from_state", Type::Text).nullable(),
                Column::new("to_state", Type::Text),
                Column::new("cause", Type::Text),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &module_and_person(),
            checks: &[Check::OneOf("to_state", STATES)],
            indexes: &[Index {
                columns: &["course_module_id", "account_id", "at"],
                unique: false,
            }],
            without_natural_key: Some("append-only history: one row per change of state"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "pending",
            columns: &[
                Column::new("course_module_id", Type::Uuid),
                Column::new("queued_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["course_module_id"],
            foreign_keys: &[other(
                &["course_module_id"],
                "activities",
                "course_module",
                OnDelete::Cascade,
            )],
            without_natural_key: Some(
                "one row per course module whose rules changed and waits to be re-evaluated",
            ),
            ..Table::EMPTY
        }),
    ],
}];
