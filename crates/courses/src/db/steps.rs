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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `courses` schema, in the neutral schema. Each category, course and section
//! owns a scope of access; the forced language of a course gains its foreign key once languages exist.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The visibility modes of a node.
pub const VISIBILITIES: &[&str] = &["shown", "hidden", "scheduled"];

/// The lifecycle states of a course.
pub const STATES: &[&str] = &["draft", "published", "archived", "deletion_scheduled"];

/// Who sees a group.
pub const GROUP_VISIBILITIES: &[&str] = &["everyone", "members", "own_only", "nobody"];

/// A foreign key to a scope of access, which takes the row with it.
const fn scope(columns: &'static [&'static str]) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some("access"),
        table: "scope",
        references: &["id"],
        on_delete: OnDelete::Cascade,
    }
}

/// A foreign key to an account of identity, which takes the row with it.
const ACCOUNT: ForeignKey = ForeignKey {
    columns: &["account_id"],
    module: Some("identity"),
    table: "account",
    references: &["id"],
    on_delete: OnDelete::Cascade,
};

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

/// A foreign key to a table of customisation, which takes the row with it.
const fn customise(columns: &'static [&'static str], table: &'static str) -> ForeignKey {
    ForeignKey {
        columns,
        module: Some("customise"),
        table,
        references: &["id"],
        on_delete: OnDelete::Cascade,
    }
}

/// The columns of an own visibility.
const VISIBILITY: [Column; 3] = [
    Column::new("visibility", Type::Text).default(ColumnDefault::Text("shown")),
    Column::new("visible_from", Type::Timestamptz).nullable(),
    Column::new("visible_until", Type::Timestamptz).nullable(),
];

/// The checks of an own visibility.
const VISIBILITY_CHECKS: [Check; 2] = [
    Check::OneOf("visibility", VISIBILITIES),
    Check::After("visible_until", "visible_from"),
];

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "categories, courses, sections and groups",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "category",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("scope_id", Type::Uuid),
                Column::new("campus_id", Type::Uuid),
                Column::new("parent_id", Type::Uuid).nullable(),
                Column::new("code", Type::Text).nullable(),
                Column::new("name", Type::Text),
                Column::new("description", Type::Text).nullable(),
                Column::new("sort_key", Type::Text),
                VISIBILITY[0],
                VISIBILITY[1],
                VISIBILITY[2],
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["scope_id"]],
            foreign_keys: &[
                scope(&["scope_id"]),
                scope(&["campus_id"]),
                own(&["parent_id"], "category", &["id"], OnDelete::Restrict),
            ],
            checks: &VISIBILITY_CHECKS,
            indexes: &[Index {
                columns: &["parent_id"],
                unique: false,
            }],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "course",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("scope_id", Type::Uuid),
                Column::new("campus_id", Type::Uuid),
                Column::new("category_id", Type::Uuid),
                Column::new("short_name", Type::Text),
                Column::new("full_name", Type::Text),
                Column::new("code", Type::Text).nullable(),
                Column::new("summary", Type::Text).nullable(),
                Column::new("layout_code", Type::Text).default(ColumnDefault::Text("topics")),
                Column::new("language_code", Type::Text).nullable(),
                Column::new("starts_at", Type::Timestamptz).nullable(),
                Column::new("ends_at", Type::Timestamptz).nullable(),
                Column::new("state", Type::Text).default(ColumnDefault::Text("draft")),
                Column::new("state_changed_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("deletion_due_at", Type::Timestamptz).nullable(),
                VISIBILITY[0],
                VISIBILITY[1],
                VISIBILITY[2],
                Column::new("sort_key", Type::Text),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["scope_id"], &["category_id", "sort_key"]],
            foreign_keys: &[
                scope(&["scope_id"]),
                scope(&["campus_id"]),
                own(&["category_id"], "category", &["id"], OnDelete::Restrict),
            ],
            checks: &[
                Check::After("ends_at", "starts_at"),
                Check::OneOf("state", STATES),
                VISIBILITY_CHECKS[0],
                VISIBILITY_CHECKS[1],
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "course_owner",
            columns: &[
                Column::new("course_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("since", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["course_id", "account_id"],
            foreign_keys: &[own(&["course_id"], "course", &["id"], OnDelete::Cascade), ACCOUNT],
            without_natural_key: Some("keyed by the course and the account"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "section",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("scope_id", Type::Uuid),
                Column::new("course_id", Type::Uuid),
                Column::new("parent_id", Type::Uuid).nullable(),
                Column::new("title", Type::Text).nullable(),
                Column::new("summary", Type::Text).nullable(),
                Column::new("sort_key", Type::Text),
                VISIBILITY[0],
                VISIBILITY[1],
                VISIBILITY[2],
                Column::new("deleted_at", Type::Timestamptz).nullable(),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["scope_id"], &["id", "course_id"]],
            foreign_keys: &[
                scope(&["scope_id"]),
                own(&["course_id"], "course", &["id"], OnDelete::Cascade),
                own(
                    &["parent_id", "course_id"],
                    "section",
                    &["id", "course_id"],
                    OnDelete::Cascade,
                ),
            ],
            checks: &VISIBILITY_CHECKS,
            indexes: &[Index {
                columns: &["course_id", "parent_id"],
                unique: false,
            }],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "group",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_id", Type::Uuid),
                Column::new("section_id", Type::Uuid).nullable(),
                Column::new("name", Type::Text),
                Column::new("code", Type::Text).nullable(),
                Column::new("description", Type::Text).nullable(),
                Column::new("visibility", Type::Text).default(ColumnDefault::Text("everyone")),
                Column::new("joining_key_hash", Type::Bytes).nullable(),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["id", "course_id"]],
            foreign_keys: &[
                own(&["course_id"], "course", &["id"], OnDelete::Cascade),
                own(
                    &["section_id", "course_id"],
                    "section",
                    &["id", "course_id"],
                    OnDelete::Cascade,
                ),
            ],
            checks: &[Check::OneOf("visibility", GROUP_VISIBILITIES)],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "group_member",
            columns: &[
                Column::new("group_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("origin", Type::Text),
                Column::new("added_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["group_id", "account_id", "origin"],
            foreign_keys: &[own(&["group_id"], "group", &["id"], OnDelete::Cascade), ACCOUNT],
            indexes: &[Index {
                columns: &["account_id"],
                unique: false,
            }],
            without_natural_key: Some("one row per group, account and origin"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "group_set",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_id", Type::Uuid),
                Column::new("name", Type::Text),
                Column::new("code", Type::Text).nullable(),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[&["id", "course_id"]],
            foreign_keys: &[own(&["course_id"], "course", &["id"], OnDelete::Cascade)],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "group_set_group",
            columns: &[
                Column::new("set_id", Type::Uuid),
                Column::new("group_id", Type::Uuid),
                Column::new("course_id", Type::Uuid),
            ],
            primary_key: &["set_id", "group_id"],
            foreign_keys: &[
                own(
                    &["set_id", "course_id"],
                    "group_set",
                    &["id", "course_id"],
                    OnDelete::Cascade,
                ),
                own(
                    &["group_id", "course_id"],
                    "group",
                    &["id", "course_id"],
                    OnDelete::Cascade,
                ),
            ],
            without_natural_key: Some("keyed by the set and the group, both of the same course"),
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "create unique index \"category_campus_code_key\" on {schema}.\"category\" (\"campus_id\", lower(\"code\")) where \"code\" is not null;
create unique index \"category_parent_sort_key\" on {schema}.\"category\" (\"campus_id\", \"parent_id\", \"sort_key\") nulls not distinct;
create unique index \"course_campus_short_name_key\" on {schema}.\"course\" (\"campus_id\", lower(\"short_name\"));
create unique index \"course_campus_code_key\" on {schema}.\"course\" (\"campus_id\", lower(\"code\")) where \"code\" is not null;
create unique index \"section_parent_sort_key\" on {schema}.\"section\" (\"course_id\", \"parent_id\", \"sort_key\") nulls not distinct;
create unique index \"group_course_name_key\" on {schema}.\"group\" (\"course_id\", lower(\"name\"));
create unique index \"group_course_code_key\" on {schema}.\"group\" (\"course_id\", lower(\"code\")) where \"code\" is not null;
create unique index \"group_set_course_name_key\" on {schema}.\"group_set\" (\"course_id\", lower(\"name\"));",
            reason: "the neutral schema has no unique index on an expression, a partial one, or one where nulls are equal",
        },
        Change::Postgres {
            sql: "alter table {schema}.\"category\" alter column \"sort_key\" type text collate \"C\";
alter table {schema}.\"course\" alter column \"sort_key\" type text collate \"C\";
alter table {schema}.\"section\" alter column \"sort_key\" type text collate \"C\";
alter table {schema}.\"course\" add constraint \"course_deletion_due_check\" check ((\"state\" = 'deletion_scheduled') = (\"deletion_due_at\" is not null));
alter table {schema}.\"group_member\" add constraint \"group_member_origin_check\" check (\"origin\" ~ '^(manual|[a-z][a-z0-9_]*)$');",
            reason: "the neutral schema has no collation, no condition between two columns and no pattern; sort keys compare byte by byte",
        },
    ],
}, Step {
    sequence: 2,
    name: "custom fields of courses",
    revert: &[],
    action: None,
    changes: &[Change::CreateTable(Table {
        name: "course_field",
        columns: &[
            Column::new("course_id", Type::Uuid),
            Column::new("field_id", Type::Uuid),
            Column::new("field_value_id", Type::Uuid),
        ],
        primary_key: &["course_id", "field_id"],
        unique: &[&["field_value_id"]],
        foreign_keys: &[
            own(&["course_id"], "course", &["id"], OnDelete::Cascade),
            customise(&["field_id"], "field"),
            customise(&["field_value_id"], "field_value"),
        ],
        ..Table::EMPTY
    })],
}];
