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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `records` schema: the categories and grade records of a course, the entries
//! that change the grade of a person, and the current grade of each person on each record.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// How a category aggregates its children.
pub const METHODS: &[&str] = &[
    "mean",
    "weighted_mean",
    "simple_weighted_mean",
    "natural",
    "median",
    "lowest",
    "highest",
    "mode",
];

/// What a grade record stands for.
pub const KINDS: &[&str] = &["activity", "manual", "category_total", "course_total"];

/// What a grade is made of.
pub const GRADE_TYPES: &[&str] = &["value", "scale", "text"];

/// Where the change of a grade came from.
pub const SOURCES: &[&str] = &["activity", "teacher", "import", "formula"];

/// A foreign key to a table of this module.
const fn own(columns: &'static [&'static str], table: &'static str, on_delete: OnDelete) -> ForeignKey {
    ForeignKey {
        columns,
        module: None,
        table,
        references: &["id"],
        on_delete,
    }
}

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
        name: "categories, grade records, entries and grades",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "category",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("course_id", Type::Uuid),
                    Column::new("parent_id", Type::Uuid).nullable(),
                    Column::new("root_course_id", Type::Uuid).nullable(),
                    Column::new("name", Type::Text),
                    Column::new("method", Type::Text).default(ColumnDefault::Text("natural")),
                    Column::new("drop_lowest", Type::Integer).default(ColumnDefault::Int(0)),
                    Column::new("keep_highest", Type::Integer).default(ColumnDefault::Int(0)),
                    Column::new("exclude_empty", Type::Boolean).default(ColumnDefault::Bool(true)),
                    Column::new("sort_key", Type::Text),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["root_course_id"], &["parent_id", "sort_key"]],
                foreign_keys: &[
                    other(&["course_id"], "courses", "course", OnDelete::Cascade),
                    own(&["parent_id"], "category", OnDelete::Cascade),
                    other(&["root_course_id"], "courses", "course", OnDelete::Cascade),
                ],
                checks: &[
                    Check::OneOf("method", METHODS),
                    Check::NonNegative("drop_lowest"),
                    Check::NonNegative("keep_highest"),
                ],
                indexes: &[Index {
                    columns: &["course_id"],
                    unique: false,
                }],
                without_natural_key: Some("a category is named by its place in the tree of a course"),
            }),
            Change::CreateTable(Table {
                name: "record",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("course_id", Type::Uuid),
                    Column::new("kind", Type::Text),
                    Column::new("title", Type::Text),
                    Column::new("course_module_id", Type::Uuid).nullable(),
                    Column::new("item_number", Type::Integer).nullable(),
                    Column::new("category_id", Type::Uuid).nullable(),
                    Column::new("total_of_id", Type::Uuid).nullable(),
                    Column::new("grade_type", Type::Text).default(ColumnDefault::Text("value")),
                    Column::new("min", Type::Numeric(12, 5)),
                    Column::new("max", Type::Numeric(12, 5)),
                    Column::new("pass_grade", Type::Numeric(12, 5)).nullable(),
                    Column::new("weight", Type::Numeric(9, 5)),
                    Column::new("extra_credit", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("multiplier", Type::Numeric(9, 5)),
                    Column::new("offset_value", Type::Numeric(12, 5)),
                    Column::new("hidden_until", Type::Timestamptz).nullable(),
                    Column::new("locked_at", Type::Timestamptz).nullable(),
                    Column::new("sort_key", Type::Text),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[
                    &["course_module_id", "item_number"],
                    &["total_of_id"],
                    &["category_id", "sort_key"],
                ],
                foreign_keys: &[
                    other(&["course_id"], "courses", "course", OnDelete::Cascade),
                    other(
                        &["course_module_id"],
                        "activities",
                        "course_module",
                        OnDelete::Cascade,
                    ),
                    own(&["category_id"], "category", OnDelete::Cascade),
                    own(&["total_of_id"], "category", OnDelete::Cascade),
                ],
                checks: &[
                    Check::OneOf("kind", KINDS),
                    Check::OneOf("grade_type", GRADE_TYPES),
                    Check::After("max", "min"),
                ],
                indexes: &[Index {
                    columns: &["course_id"],
                    unique: false,
                }],
                without_natural_key: Some(
                    "an activity record is unique by course module and item, a total by its category",
                ),
            }),
            Change::CreateTable(Table {
                name: "entry",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("record_id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("raw", Type::Numeric(12, 5)).nullable(),
                    Column::new("text", Type::Text).nullable(),
                    Column::new("feedback", Type::Text).nullable(),
                    Column::new("source", Type::Text),
                    Column::new("actor", Type::Uuid).nullable(),
                    Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                foreign_keys: &[
                    own(&["record_id"], "record", OnDelete::Cascade),
                    other(&["account_id"], "identity", "account", OnDelete::Cascade),
                ],
                checks: &[Check::OneOf("source", SOURCES)],
                indexes: &[Index {
                    columns: &["record_id", "account_id", "at"],
                    unique: false,
                }],
                without_natural_key: Some("append-only history: one row per change"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "grade",
                columns: &[
                    Column::new("record_id", Type::Uuid),
                    Column::new("account_id", Type::Uuid),
                    Column::new("raw", Type::Numeric(12, 5)).nullable(),
                    Column::new("final", Type::Numeric(12, 5)).nullable(),
                    Column::new("text", Type::Text).nullable(),
                    Column::new("overridden", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("excluded", Type::Boolean).default(ColumnDefault::Bool(false)),
                    Column::new("hidden_until", Type::Timestamptz).nullable(),
                    Column::new("locked_at", Type::Timestamptz).nullable(),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["record_id", "account_id"],
                foreign_keys: &[
                    own(&["record_id"], "record", OnDelete::Cascade),
                    other(&["account_id"], "identity", "account", OnDelete::Cascade),
                ],
                indexes: &[Index {
                    columns: &["account_id"],
                    unique: false,
                }],
                without_natural_key: Some("keyed by record and person, the target of the upsert"),
                ..Table::EMPTY
            }),
            Change::Postgres {
                sql: "alter table {schema}.\"category\" alter column \"sort_key\" type text collate \"C\";
alter table {schema}.\"record\" alter column \"sort_key\" type text collate \"C\";",
                reason: "the neutral schema has no collation; sort keys compare byte by byte",
            },
        ],
    },
    Step {
        sequence: 2,
        name: "the queue of people whose totals are recalculated",
        revert: &[],
        action: None,
        changes: &[Change::CreateTable(Table {
            name: "recalculation",
            columns: &[
                Column::new("course_id", Type::Uuid),
                Column::new("account_id", Type::Uuid),
                Column::new("queued_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["course_id", "account_id"],
            foreign_keys: &[
                other(&["course_id"], "courses", "course", OnDelete::Cascade),
                other(&["account_id"], "identity", "account", OnDelete::Cascade),
            ],
            indexes: &[Index {
                columns: &["queued_at"],
                unique: false,
            }],
            without_natural_key: Some("one row per person and course waiting to be recalculated"),
            ..Table::EMPTY
        })],
    },
    Step {
        sequence: 3,
        name: "scales and letter sets",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "scale",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("scope_id", Type::Uuid),
                    Column::new("code", Type::Text),
                    Column::new("version", Type::Integer),
                    Column::new("items", Type::TextArray),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["scope_id", "code", "version"]],
                foreign_keys: &[other(&["scope_id"], "access", "scope", OnDelete::Cascade)],
                without_natural_key: Some("a scale is named by its scope, code and version"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "letter_set",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("scope_id", Type::Uuid),
                    Column::new("version", Type::Integer),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["scope_id", "version"]],
                foreign_keys: &[other(&["scope_id"], "access", "scope", OnDelete::Cascade)],
                without_natural_key: Some("a letter set is named by its scope and version"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "letter_band",
                columns: &[
                    Column::new("letter_set_id", Type::Uuid),
                    Column::new("min_percent", Type::Numeric(9, 5)),
                    Column::new("letter", Type::Text),
                ],
                primary_key: &["letter_set_id", "min_percent"],
                foreign_keys: &[own(&["letter_set_id"], "letter_set", OnDelete::Cascade)],
                without_natural_key: Some("a band is named by its letter set and its lowest percentage"),
                ..Table::EMPTY
            }),
            Change::AddColumn {
                table: "record",
                column: Column::new("scale_id", Type::Uuid).nullable(),
            },
            Change::AddForeignKey {
                table: "record",
                foreign_key: own(&["scale_id"], "scale", OnDelete::Restrict),
            },
        ],
    },
    Step {
        sequence: 4,
        name: "access conditions on a grade",
        revert: &[],
        action: None,
        changes: &[Change::CreateTable(Table {
            name: "condition_grade",
            columns: &[
                Column::new("condition_id", Type::Uuid),
                Column::new("record_id", Type::Uuid),
                Column::new("min_percent", Type::Numeric(7, 4)).nullable(),
                Column::new("max_percent", Type::Numeric(7, 4)).nullable(),
            ],
            primary_key: &["condition_id"],
            foreign_keys: &[
                other(&["condition_id"], "activities", "condition", OnDelete::Cascade),
                own(&["record_id"], "record", OnDelete::Cascade),
            ],
            checks: &[
                Check::NonNegative("min_percent"),
                Check::After("max_percent", "min_percent"),
            ],
            indexes: &[Index {
                columns: &["record_id"],
                unique: false,
            }],
            without_natural_key: Some("the parameters of one leaf of a tree of activities"),
            ..Table::EMPTY
        })],
    },
    Step {
        sequence: 5,
        name: "formulas and their dependencies",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "formula",
                columns: &[
                    Column::new("record_id", Type::Uuid),
                    Column::new("expression", Type::Text),
                    Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["record_id"],
                foreign_keys: &[own(&["record_id"], "record", OnDelete::Cascade)],
                without_natural_key: Some("the formula of one record"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "formula_dependency",
                columns: &[
                    Column::new("record_id", Type::Uuid),
                    Column::new("depends_on_id", Type::Uuid),
                ],
                primary_key: &["record_id", "depends_on_id"],
                foreign_keys: &[
                    ForeignKey {
                        columns: &["record_id"],
                        module: None,
                        table: "formula",
                        references: &["record_id"],
                        on_delete: OnDelete::Cascade,
                    },
                    own(&["depends_on_id"], "record", OnDelete::Cascade),
                ],
                indexes: &[Index {
                    columns: &["depends_on_id"],
                    unique: false,
                }],
                without_natural_key: Some("one row per record a formula reads"),
                ..Table::EMPTY
            }),
        ],
    },
    Step {
        sequence: 6,
        name: "exports of the grades of a course",
        revert: &[],
        action: None,
        changes: &[Change::CreateTable(Table {
            name: "export",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_id", Type::Uuid),
                Column::new("requested_by", Type::Uuid),
                Column::new("include_inactive", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("state", Type::Text).default(ColumnDefault::Text("pending")),
                Column::new("file_id", Type::Uuid).nullable(),
                Column::new("error", Type::Text).nullable(),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                Column::new("completed_at", Type::Timestamptz).nullable(),
                Column::new("expires_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                other(&["course_id"], "courses", "course", OnDelete::Cascade),
                other(&["requested_by"], "identity", "account", OnDelete::Cascade),
                other(&["file_id"], "files", "file", OnDelete::SetNull),
            ],
            checks: &[Check::OneOf("state", EXPORT_STATES)],
            indexes: &[Index {
                columns: &["course_id", "requested_by", "created_at"],
                unique: false,
            }],
            without_natural_key: Some("a person may export the grades of a course as often as they need"),
            ..Table::EMPTY
        })],
    },
];

/// Where an export stands.
pub const EXPORT_STATES: &[&str] = &["pending", "completed", "failed", "expired"];
