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
// @package   kampus-settings
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `settings` schema, in the neutral schema: the definitions modules declare, the
//! values set in each scope, and the append-only record of every change.

use kampus_db::Step;
use kampus_db::schema::{Change, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// A foreign key to a definition, which takes the row with it.
const DEFINITION: ForeignKey = ForeignKey {
    columns: &["definition_key"],
    module: None,
    table: "definition",
    references: &["key"],
    on_delete: OnDelete::Cascade,
};

/// A foreign key to a scope of access, which takes the row with it.
const SCOPE: ForeignKey = ForeignKey {
    columns: &["scope_id"],
    module: Some("access"),
    table: "scope",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "definitions, values and changes",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "definition",
            columns: &[
                Column::new("key", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("kind", Type::Json("settings.kind")),
                Column::new("default_value", Type::Json("settings.value")),
                Column::new("levels", Type::Json("settings.levels")),
                Column::new("permission", Type::Text),
                Column::new("restart_required", Type::Boolean),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["key"],
            without_natural_key: Some("keyed by the key its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "value",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("definition_key", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("value", Type::Json("settings.value")),
                Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["definition_key", "scope_id"]],
            foreign_keys: &[DEFINITION, SCOPE],
            indexes: &[Index {
                columns: &["scope_id"],
                unique: false,
            }],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "change",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("definition_key", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("actor", Type::Uuid).nullable(),
                Column::new("before", Type::Json("settings.value")).nullable(),
                Column::new("after", Type::Json("settings.value")).nullable(),
                Column::new("locked", Type::Boolean),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[DEFINITION, SCOPE],
            indexes: &[Index {
                columns: &["definition_key", "at"],
                unique: false,
            }],
            without_natural_key: Some("an append-only record of changes; its actor outlives the account"),
            ..Table::EMPTY
        }),
    ],
}];
