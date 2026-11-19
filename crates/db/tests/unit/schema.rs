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

//! Tests of `kampus-db`: schema.

use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// A table using every feature of the neutral schema.
const GROUP: Table = Table {
    name: "group",
    columns: &[
        Column::new("id", Type::Uuid),
        Column::new("course_id", Type::Uuid),
        Column::new("name", Type::Varchar(120)),
        Column::new("state", Type::Text).default(ColumnDefault::Text("active")),
        Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
    ],
    primary_key: &["id"],
    unique: &[&["course_id", "name"]],
    foreign_keys: &[ForeignKey {
        columns: &["course_id"],
        module: None,
        table: "course",
        references: &["id"],
        on_delete: OnDelete::Cascade,
    }],
    checks: &[Check::OneOf("state", &["active", "archived"])],
    indexes: &[Index {
        columns: &["created_at"],
        unique: false,
    }],
    ..Table::EMPTY
};

#[test]
fn tables_are_written_with_quoted_names_and_placeholders() {
    let sql = Change::CreateTable(GROUP).postgres_template();
    assert!(
        sql.starts_with("create table {schema}.\"group\" (\n    \"id\" uuid not null,"),
        "{sql}"
    );
    assert!(sql.contains("\"state\" text not null default 'active'"));
    assert!(sql.contains("\"created_at\" timestamptz not null default now()"));
    assert!(sql.contains("constraint \"group_course_id_name_key\" unique (\"course_id\", \"name\")"));
    assert!(sql.contains("references {schema}.\"course\" (\"id\") on delete cascade"));
    assert!(sql.contains("check (\"state\" in ('active', 'archived'))"));
    assert!(sql.contains("create index \"group_created_at_idx\" on {schema}.\"group\" (\"created_at\");"));
}

#[test]
fn foreign_keys_are_indexed_unless_a_key_leads_with_them() {
    /// An association whose key leads with one of its references.
    const MEMBER: Table = Table {
        name: "member",
        columns: &[
            Column::new("group_id", Type::Uuid),
            Column::new("account_id", Type::Uuid),
        ],
        primary_key: &["group_id", "account_id"],
        foreign_keys: &[
            ForeignKey {
                columns: &["group_id"],
                module: None,
                table: "group",
                references: &["id"],
                on_delete: OnDelete::Cascade,
            },
            ForeignKey {
                columns: &["account_id"],
                module: Some("identity"),
                table: "account",
                references: &["id"],
                on_delete: OnDelete::Restrict,
            },
        ],
        ..Table::EMPTY
    };
    let sql = Change::CreateTable(GROUP).postgres_template();
    assert!(
        !sql.contains("group_course_id_idx"),
        "the natural key already starts with course_id"
    );
    let sql = Change::CreateTable(MEMBER).postgres_template();
    assert!(sql.contains("references {schema:identity}.\"account\""));
    assert!(!sql.contains("member_group_id_idx"));
    assert!(sql.contains("create index \"member_account_id_idx\""));
}

#[test]
fn checks_have_neutral_forms() {
    let table = "grant";
    let arc = Change::AddCheck {
        table,
        check: Check::ExactlyOne(&["account_id", "scope_id"]),
    }
    .postgres_template();
    assert!(arc.ends_with("check (num_nonnulls(\"account_id\", \"scope_id\") = 1);"));
    let window = Change::AddCheck {
        table,
        check: Check::After("valid_until", "valid_from"),
    }
    .postgres_template();
    assert!(window.ends_with("check (\"valid_until\" > \"valid_from\");"));
}

#[test]
fn other_changes_are_written_too() {
    let add = Change::AddColumn {
        table: "group",
        column: Column::new("visible", Type::Boolean).default(ColumnDefault::Bool(true)),
    };
    assert_eq!(
        add.postgres_template(),
        "alter table {schema}.\"group\" add column \"visible\" boolean not null default true;"
    );
    let drop = Change::DropColumn {
        table: "group",
        column: "legacy",
        unused_since: "0.1.0",
    };
    assert_eq!(
        drop.postgres_template(),
        "alter table {schema}.\"group\" drop column \"legacy\";"
    );
    assert_eq!(Type::Numeric(12, 5).postgres(), "numeric(12,5)");
    assert_eq!(Type::Json("settings").postgres(), "jsonb");
}
