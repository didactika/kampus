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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `notify` schema: the types modules declare, the notifications of each person and
//! what each person chose to receive.

use kampus_db::Step;
use kampus_db::schema::{Change, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

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
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "notification types, notifications and preferences",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "notification_type",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("channels", Type::Json("notify.channels")),
                Column::new("required", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "notification",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("type_code", Type::Text),
                Column::new("recipient_id", Type::Uuid),
                Column::new("dedupe", Type::Text).nullable(),
                Column::new("payload", Type::Json("notify.payload")),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("read_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            unique: &[&["recipient_id", "type_code", "dedupe"]],
            foreign_keys: &[
                ForeignKey {
                    columns: &["type_code"],
                    module: None,
                    table: "notification_type",
                    references: &["code"],
                    on_delete: OnDelete::Restrict,
                },
                other(&["recipient_id"], "identity", "account", OnDelete::Cascade),
            ],
            indexes: &[Index {
                columns: &["recipient_id", "created_at"],
                unique: false,
            }],
            without_natural_key: Some(
                "a notification is identified by the event that caused it when there is one",
            ),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "preference",
            columns: &[
                Column::new("account_id", Type::Uuid),
                Column::new("type_code", Type::Text),
                Column::new("channel", Type::Text),
                Column::new("enabled", Type::Boolean),
            ],
            primary_key: &["account_id", "type_code", "channel"],
            foreign_keys: &[
                other(&["account_id"], "identity", "account", OnDelete::Cascade),
                ForeignKey {
                    columns: &["type_code"],
                    module: None,
                    table: "notification_type",
                    references: &["code"],
                    on_delete: OnDelete::Cascade,
                },
            ],
            without_natural_key: Some("one choice per person, type and channel"),
            ..Table::EMPTY
        }),
    ],
}];
