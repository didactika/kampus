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
// @package   kampus-calendar
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `calendar` schema: the entries people create in a scope or for themselves.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

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

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[
    Step {
        sequence: 1,
        name: "entries",
        revert: &[],
        action: None,
        changes: &[Change::CreateTable(Table {
            name: "entry",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("scope_id", Type::Uuid).nullable(),
                Column::new("owner_id", Type::Uuid).nullable(),
                Column::new("author", Type::Uuid).nullable(),
                Column::new("title", Type::Text),
                Column::new("description", Type::Text).nullable(),
                Column::new("starts_at", Type::Timestamptz),
                Column::new("ends_at", Type::Timestamptz),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                other(&["scope_id"], "access", "scope", OnDelete::Cascade),
                other(&["owner_id"], "identity", "account", OnDelete::Cascade),
            ],
            checks: &[
                Check::ExactlyOne(&["scope_id", "owner_id"]),
                Check::After("ends_at", "starts_at"),
            ],
            indexes: &[
                Index {
                    columns: &["scope_id", "starts_at"],
                    unique: false,
                },
                Index {
                    columns: &["owner_id", "starts_at"],
                    unique: false,
                },
            ],
            without_natural_key: Some("an entry has no key of its own: people create as many as they need"),
            ..Table::EMPTY
        })],
    },
    Step {
        sequence: 2,
        name: "recurrence, time zones, all-day and group entries, and exceptions of occurrences",
        revert: &[],
        action: None,
        changes: &[
            Change::AddColumn {
                table: "entry",
                column: Column::new("time_zone", Type::Text).default(ColumnDefault::Text("UTC")),
            },
            Change::AddColumn {
                table: "entry",
                column: Column::new("all_day", Type::Boolean).default(ColumnDefault::Bool(false)),
            },
            Change::AddColumn {
                table: "entry",
                column: Column::new("recurrence", Type::Text).nullable(),
            },
            Change::AddColumn {
                table: "entry",
                column: Column::new("group_id", Type::Uuid).nullable(),
            },
            Change::AddForeignKey {
                table: "entry",
                foreign_key: other(&["group_id"], "courses", "group", OnDelete::Cascade),
            },
            Change::CreateTable(Table {
                name: "entry_exception",
                columns: &[
                    Column::new("entry_id", Type::Uuid),
                    Column::new("occurrence_at", Type::Timestamptz),
                    Column::new("cancelled", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("title", Type::Text).nullable(),
                    Column::new("starts_at", Type::Timestamptz).nullable(),
                    Column::new("ends_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["entry_id", "occurrence_at"],
                foreign_keys: &[ForeignKey {
                    columns: &["entry_id"],
                    module: None,
                    table: "entry",
                    references: &["id"],
                    on_delete: OnDelete::Cascade,
                }],
                checks: &[Check::After("ends_at", "starts_at")],
                without_natural_key: Some("one change per occurrence of a recurring entry"),
                ..Table::EMPTY
            }),
        ],
    },
];
