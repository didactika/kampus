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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `users` schema, in the neutral schema. The language and the picture of a
//! profile gain their foreign keys in later steps, once the modules that own languages and files exist.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, OnDelete, Table, Type};

/// A foreign key to the account of identity, which takes the row with it.
const ACCOUNT: ForeignKey = ForeignKey {
    columns: &["account_id"],
    module: Some("identity"),
    table: "account",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

/// A foreign key to a scope of access.
const fn scope(columns: &'static [&'static str]) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some("access"),
        table: "scope",
        references: &["id"],
        on_delete: OnDelete::Cascade,
    }
}

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[
    Step {
        sequence: 1,
        name: "profiles, preferences and system groups",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "profile",
                columns: &[
                    Column::new("account_id", Type::Uuid),
                    Column::new("campus_id", Type::Uuid),
                    Column::new("given_name", Type::Text).nullable(),
                    Column::new("family_name", Type::Text).nullable(),
                    Column::new("display_name", Type::Text).nullable(),
                    Column::new("language_code", Type::Text).nullable(),
                    Column::new("time_zone", Type::Text).nullable(),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["account_id"],
                foreign_keys: &[ACCOUNT, scope(&["campus_id"])],
                without_natural_key: Some("one row per account, keyed by the account"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "preference",
                columns: &[
                    Column::new("account_id", Type::Uuid),
                    Column::new("key", Type::Text),
                    Column::new("value", Type::Json("users.preference_value")),
                    Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["account_id", "key"],
                foreign_keys: &[ACCOUNT],
                without_natural_key: Some("keyed by the account and the declared key"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "system_group",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("scope_id", Type::Uuid),
                    Column::new("code", Type::Text),
                    Column::new("name", Type::Text),
                    Column::new("kind", Type::Text),
                    Column::new("rule", Type::Json("users.group_rule")).nullable(),
                    Column::new("owner_module", Type::Text).nullable(),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["scope_id", "code"]],
                foreign_keys: &[scope(&["scope_id"])],
                checks: &[Check::OneOf("kind", &["manual", "rule"])],
                ..Table::EMPTY
            }),
            Change::Postgres {
                sql: "alter table {schema}.\"system_group\" add constraint \"system_group_rule_check\" check ((\"kind\" = 'rule') = (\"rule\" is not null));",
                reason: "the neutral schema has no condition between two columns",
            },
            Change::CreateTable(Table {
                name: "system_group_member",
                columns: &[
                    Column::new("group_id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("origin", Type::Text),
                    Column::new("added_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["group_id", "account_id", "origin"],
                foreign_keys: &[
                    ForeignKey {
                        columns: &["group_id"],
                        module: None,
                        table: "system_group",
                        references: &["id"],
                        on_delete: OnDelete::Cascade,
                    },
                    ACCOUNT,
                ],
                checks: &[Check::OneOf("origin", &["manual", "rule", "sync"])],
                without_natural_key: Some("a membership is its group, its account and its origin"),
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 2,
        name: "external references of system groups",
        revert: &[],
        action: None,
        changes: &[kampus_db::reference_table!(
            "system_group_reference",
            "system_group"
        )],
    },
];
