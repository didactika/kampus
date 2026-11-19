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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `access` schema, in the neutral schema. Scope kinds and their
//! parents, and the lowest kind of an override, are rules of the service.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, OnDelete, Table, Type};

/// The kinds of scope, as stored.
const KINDS: &[&str] = &[
    "platform", "tenant", "campus", "category", "course", "section", "activity", "widget", "user",
];

/// The values of a permission, as stored.
const VALUES: &[&str] = &["allow", "deny", "forbid"];

/// A foreign key to a table of this module.
const fn local(columns: &'static [&'static str], table: &'static str, on_delete: OnDelete) -> ForeignKey {
    ForeignKey {
        columns,
        module: None,
        table,
        references: &["id"],
        on_delete,
    }
}

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[
    Step {
        sequence: 1,
        name: "scopes, permissions, roles and grants",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "scope",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("kind", Type::Text),
                    Column::new("parent_id", Type::Uuid).nullable(),
                    Column::new("campus_id", Type::Uuid).nullable(),
                    Column::new("frozen", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                foreign_keys: &[
                    local(&["parent_id"], "scope", OnDelete::Cascade),
                    local(&["campus_id"], "scope", OnDelete::Cascade),
                ],
                checks: &[Check::OneOf("kind", KINDS)],
                without_natural_key: Some(
                    "a scope is identified by the entity it belongs to, which holds the natural key",
                ),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "scope_closure",
                columns: &[
                    Column::new("ancestor_id", Type::Uuid),
                    Column::new("descendant_id", Type::Uuid),
                    Column::new("depth", Type::Integer),
                ],
                primary_key: &["ancestor_id", "descendant_id"],
                foreign_keys: &[
                    local(&["ancestor_id"], "scope", OnDelete::Cascade),
                    local(&["descendant_id"], "scope", OnDelete::Cascade),
                ],
                checks: &[Check::NonNegative("depth")],
                indexes: &[kampus_db::schema::Index {
                    columns: &["descendant_id", "depth"],
                    unique: false,
                }],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "permission",
                columns: &[
                    Column::new("code", Type::Text),
                    Column::new("module", Type::Text),
                    Column::new("kind", Type::Text),
                    Column::new("risks", Type::TextArray),
                    Column::new("lowest_kind", Type::Text),
                    Column::new("retired_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["code"],
                checks: &[
                    Check::OneOf("kind", &["read", "write"]),
                    Check::OneOf("lowest_kind", KINDS),
                ],
                without_natural_key: Some("the primary key is the natural code"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "role_template",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("code", Type::Text),
                    Column::new("version", Type::Integer),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["code", "version"]],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "role_template_value",
                columns: &[
                    Column::new("template_id", Type::Uuid),
                    Column::new("permission_code", Type::Text),
                    Column::new("value", Type::Text),
                ],
                primary_key: &["template_id", "permission_code"],
                foreign_keys: &[
                    local(&["template_id"], "role_template", OnDelete::Cascade),
                    ForeignKey {
                        columns: &["permission_code"],
                        module: None,
                        table: "permission",
                        references: &["code"],
                        on_delete: OnDelete::Restrict,
                    },
                ],
                checks: &[Check::OneOf("value", VALUES)],
                without_natural_key: Some("the primary key is the natural key"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "role",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("campus_id", Type::Uuid),
                    Column::new("short_name", Type::Varchar(100)),
                    Column::new("template_id", Type::Uuid),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["campus_id", "short_name"]],
                foreign_keys: &[
                    local(&["campus_id"], "scope", OnDelete::Cascade),
                    local(&["template_id"], "role_template", OnDelete::Restrict),
                ],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "role_scope_kind",
                columns: &[
                    Column::new("role_id", Type::Uuid),
                    Column::new("kind", Type::Text),
                ],
                primary_key: &["role_id", "kind"],
                foreign_keys: &[local(&["role_id"], "role", OnDelete::Cascade)],
                checks: &[Check::OneOf("kind", KINDS)],
                without_natural_key: Some("the primary key is the natural key"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "role_permission",
                columns: &[
                    Column::new("role_id", Type::Uuid),
                    Column::new("permission_code", Type::Text),
                    Column::new("value", Type::Text),
                ],
                primary_key: &["role_id", "permission_code"],
                foreign_keys: &[
                    local(&["role_id"], "role", OnDelete::Cascade),
                    ForeignKey {
                        columns: &["permission_code"],
                        module: None,
                        table: "permission",
                        references: &["code"],
                        on_delete: OnDelete::Restrict,
                    },
                ],
                checks: &[Check::OneOf("value", VALUES)],
                without_natural_key: Some("the primary key is the natural key"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "role_allowance",
                columns: &[
                    Column::new("role_id", Type::Uuid),
                    Column::new("target_role_id", Type::Uuid),
                    Column::new("kind", Type::Text),
                ],
                primary_key: &["role_id", "target_role_id", "kind"],
                foreign_keys: &[
                    local(&["role_id"], "role", OnDelete::Cascade),
                    local(&["target_role_id"], "role", OnDelete::Cascade),
                ],
                checks: &[Check::OneOf("kind", &["grant", "override", "view_as"])],
                without_natural_key: Some("the primary key is the natural key"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "permission_override",
                columns: &[
                    Column::new("scope_id", Type::Uuid),
                    Column::new("role_id", Type::Uuid),
                    Column::new("permission_code", Type::Text),
                    Column::new("value", Type::Text),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                ],
                primary_key: &["scope_id", "role_id", "permission_code"],
                foreign_keys: &[
                    local(&["scope_id"], "scope", OnDelete::Cascade),
                    local(&["role_id"], "role", OnDelete::Cascade),
                    ForeignKey {
                        columns: &["permission_code"],
                        module: None,
                        table: "permission",
                        references: &["code"],
                        on_delete: OnDelete::Restrict,
                    },
                ],
                checks: &[Check::OneOf("value", VALUES)],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "principal",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("kind", Type::Text).default(ColumnDefault::Text("account")),
                    Column::new("campus_id", Type::Uuid),
                    Column::new("active", Type::Boolean).default(ColumnDefault::Bool(true)),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                foreign_keys: &[local(&["campus_id"], "scope", OnDelete::Restrict)],
                checks: &[Check::OneOf("kind", &["account"])],
                without_natural_key: Some(
                    "a principal is identified by its account, which holds the natural keys",
                ),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "grant",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("role_id", Type::Uuid),
                    Column::new("scope_id", Type::Uuid),
                    Column::new("principal_id", Type::Uuid),
                    Column::new("origin", Type::Text),
                    Column::new("origin_module", Type::Text).nullable(),
                    Column::new("valid_from", Type::Timestamptz),
                    Column::new("valid_until", Type::Timestamptz).nullable(),
                    Column::new("granted_by", Type::Uuid).nullable(),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                foreign_keys: &[
                    local(&["role_id"], "role", OnDelete::Restrict),
                    local(&["scope_id"], "scope", OnDelete::Cascade),
                    local(&["principal_id"], "principal", OnDelete::Cascade),
                    local(&["granted_by"], "principal", OnDelete::SetNull),
                ],
                checks: &[
                    Check::OneOf("origin", &["manual", "module"]),
                    Check::After("valid_until", "valid_from"),
                ],
                without_natural_key: Some(
                    "manual grants are unique by role, scope and principal through a partial index of the next change",
                ),
                ..Table::EMPTY
            }),
            Change::Postgres {
                sql: "alter table {schema}.\"grant\" add constraint \"grant_origin_module_check\" check ((\"origin\" = 'module') = (\"origin_module\" is not null));\ncreate unique index \"grant_manual_key\" on {schema}.\"grant\" (\"role_id\", \"scope_id\", \"principal_id\") where \"origin\" = 'manual';",
                reason: "the neutral schema has no condition between two columns and no partial unique index",
            },
            Change::CreateTable(Table {
                name: "view_as_session",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("principal_id", Type::Uuid),
                    Column::new("scope_id", Type::Uuid),
                    Column::new("role_id", Type::Uuid),
                    Column::new("started_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("ended_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["id"],
                foreign_keys: &[
                    local(&["principal_id"], "principal", OnDelete::Cascade),
                    local(&["scope_id"], "scope", OnDelete::Cascade),
                    local(&["role_id"], "role", OnDelete::Cascade),
                ],
                checks: &[Check::After("ended_at", "started_at")],
                without_natural_key: Some("a session is an event, identified by its id"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "change",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("actor", Type::Uuid).nullable(),
                    Column::new("entity", Type::Text),
                    Column::new("entity_key", Type::Text),
                    Column::new("before", Type::Json("access.change")).nullable(),
                    Column::new("after", Type::Json("access.change")).nullable(),
                    Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                indexes: &[kampus_db::schema::Index {
                    columns: &["entity", "entity_key", "at"],
                    unique: false,
                }],
                without_natural_key: Some("an append-only history of changes"),
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 2,
        name: "roles that require a second factor",
        revert: &[],
        action: None,
        changes: &[Change::AddColumn {
            table: "role",
            column: Column::new("second_factor_required", Type::Boolean).default(ColumnDefault::Bool(false)),
        }],
    },
];

#[cfg(test)]
mod tests {

    #[test]
    fn the_schema_passes_the_linter() {
        assert_eq!(kampus_db::lint::lint(&[crate::MODULE.schema()], "0.1.0"), []);
    }
}
