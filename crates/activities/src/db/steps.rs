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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `activities` schema: the types modules provide, course modules placed in
//! sections with their settings and files, and the recycle bin; the external references of course
//! modules; and the exceptions of dates, time limits and attempts.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// Whether a type is an activity or a material.
pub const KINDS: &[&str] = &["activity", "material"];

/// The visibility modes of a course module.
pub const VISIBILITIES: &[&str] = &["shown", "hidden", "scheduled"];

/// Where a course module stands: in its section, or in the recycle bin.
pub const STATES: &[&str] = &["active", "in_bin"];

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
        name: "types, course modules, settings, files and the recycle bin",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "activity_type",
                columns: &[
                    Column::new("code", Type::Text),
                    Column::new("module", Type::Text),
                    Column::new("kind", Type::Text),
                    Column::new("summary", Type::Text),
                    Column::new("features", Type::Json("activities.features")),
                    Column::new("settings", Type::Json("activities.type_settings")),
                    Column::new("enabled", Type::Boolean).default(ColumnDefault::Bool(true)),
                    Column::new("retired_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["code"],
                checks: &[Check::OneOf("kind", KINDS)],
                without_natural_key: Some("keyed by the code its module declares"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "course_module",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("scope_id", Type::Uuid),
                    Column::new("section_id", Type::Uuid),
                    Column::new("type_code", Type::Text),
                    Column::new("title", Type::Text),
                    Column::new("sort_key", Type::Text),
                    Column::new("visibility", Type::Text).default(ColumnDefault::Text("shown")),
                    Column::new("visible_from", Type::Timestamptz).nullable(),
                    Column::new("visible_until", Type::Timestamptz).nullable(),
                    Column::new("on_course_page", Type::Boolean).default(ColumnDefault::Bool(true)),
                    Column::new("state", Type::Text).default(ColumnDefault::Text("active")),
                    Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["scope_id"], &["section_id", "sort_key"]],
                foreign_keys: &[
                    other(&["scope_id"], "access", "scope", OnDelete::Cascade),
                    other(&["section_id"], "courses", "section", OnDelete::Cascade),
                    ForeignKey {
                        columns: &["type_code"],
                        module: None,
                        table: "activity_type",
                        references: &["code"],
                        on_delete: OnDelete::Restrict,
                    },
                ],
                checks: &[
                    Check::OneOf("visibility", VISIBILITIES),
                    Check::OneOf("state", STATES),
                    Check::After("visible_until", "visible_from"),
                ],
                indexes: &[Index {
                    columns: &["section_id", "state"],
                    unique: false,
                }],
                without_natural_key: Some("a course module is known by its id; titles repeat"),
            }),
            Change::CreateTable(Table {
                name: "module_setting",
                columns: &[
                    Column::new("course_module_id", Type::Uuid),
                    Column::new("values", Type::Json("activities.module_settings")),
                ],
                primary_key: &["course_module_id"],
                foreign_keys: &[own(&["course_module_id"], "course_module", OnDelete::Cascade)],
                without_natural_key: Some("one set of settings per course module"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "module_file",
                columns: &[
                    Column::new("course_module_id", Type::Uuid),
                    Column::new("file_id", Type::Uuid),
                ],
                primary_key: &["course_module_id"],
                unique: &[&["file_id"]],
                foreign_keys: &[
                    own(&["course_module_id"], "course_module", OnDelete::Cascade),
                    other(&["file_id"], "files", "file", OnDelete::Cascade),
                ],
                without_natural_key: Some("the file of a material, owned through this table"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "bin_item",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("course_id", Type::Uuid),
                    Column::new("course_module_id", Type::Uuid),
                    Column::new("deleted_by", Type::Uuid).nullable(),
                    Column::new("deleted_at", Type::Timestamptz).default(ColumnDefault::Now),
                    Column::new("expires_at", Type::Timestamptz),
                    Column::new("error", Type::Text).nullable(),
                ],
                primary_key: &["id"],
                unique: &[&["course_module_id"]],
                foreign_keys: &[
                    own(&["course_module_id"], "course_module", OnDelete::Cascade),
                    other(&["course_id"], "courses", "course", OnDelete::Cascade),
                ],
                indexes: &[Index {
                    columns: &["expires_at"],
                    unique: false,
                }],
                without_natural_key: Some("one bin item per course module in the bin"),
                ..Table::EMPTY
            }),
            Change::Postgres {
                sql: "alter table {schema}.\"course_module\" alter column \"sort_key\" type text collate \"C\";",
                reason: "the neutral schema has no collation; sort keys compare byte by byte",
            },
        ],
    },
    Step {
        sequence: 2,
        name: "external references of course modules",
        revert: &[],
        action: None,
        changes: &[kampus_db::reference_table!(
            "course_module_reference",
            "course_module"
        )],
    },
    Step {
        sequence: 3,
        name: "exceptions",
        revert: &[],
        action: None,
        changes: &[Change::CreateTable(Table {
            name: "exception",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("course_module_id", Type::Uuid),
                Column::new("account_id", Type::Uuid).nullable(),
                Column::new("group_id", Type::Uuid).nullable(),
                Column::new("opens_at", Type::Timestamptz).nullable(),
                Column::new("closes_at", Type::Timestamptz).nullable(),
                Column::new("due_at", Type::Timestamptz).nullable(),
                Column::new("cutoff_at", Type::Timestamptz).nullable(),
                Column::new("time_limit_seconds", Type::Integer).nullable(),
                Column::new("attempts", Type::Integer).nullable(),
                Column::new("reason", Type::Text).nullable(),
                Column::new("created_by", Type::Uuid).nullable(),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
                Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            unique: &[
                &["course_module_id", "account_id"],
                &["course_module_id", "group_id"],
            ],
            foreign_keys: &[
                own(&["course_module_id"], "course_module", OnDelete::Cascade),
                other(&["account_id"], "identity", "account", OnDelete::Cascade),
                other(&["group_id"], "courses", "group", OnDelete::Cascade),
            ],
            checks: &[
                Check::ExactlyOne(&["account_id", "group_id"]),
                Check::Positive("time_limit_seconds"),
                Check::NonNegative("attempts"),
            ],
            indexes: &[Index {
                columns: &["group_id"],
                unique: false,
            }],
            without_natural_key: Some(
                "one exception per course module and person, and one per course module and group",
            ),
        })],
    },
    Step {
        sequence: 4,
        name: "access conditions",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "condition",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("course_module_id", Type::Uuid).nullable(),
                    Column::new("section_id", Type::Uuid).nullable(),
                    Column::new("parent_id", Type::Uuid).nullable(),
                    Column::new("position", Type::Integer).default(ColumnDefault::Int(0)),
                    Column::new("operator", Type::Text),
                    Column::new("kind", Type::Text).nullable(),
                    Column::new("show_when_unmet", Type::Boolean).default(ColumnDefault::Bool(true)),
                ],
                primary_key: &["id"],
                unique: &[&["course_module_id"], &["section_id"]],
                foreign_keys: &[
                    own(&["course_module_id"], "course_module", OnDelete::Cascade),
                    other(&["section_id"], "courses", "section", OnDelete::Cascade),
                    own(&["parent_id"], "condition", OnDelete::Cascade),
                ],
                checks: &[
                    Check::ExactlyOne(&["course_module_id", "section_id", "parent_id"]),
                    Check::OneOf("operator", OPERATORS),
                ],
                indexes: &[Index {
                    columns: &["parent_id", "position"],
                    unique: false,
                }],
                without_natural_key: Some("a node of the tree of a course module or a section"),
            }),
            Change::CreateTable(Table {
                name: "condition_date",
                columns: &[
                    Column::new("condition_id", Type::Uuid),
                    Column::new("opens_at", Type::Timestamptz).nullable(),
                    Column::new("closes_at", Type::Timestamptz).nullable(),
                ],
                primary_key: &["condition_id"],
                foreign_keys: &[own(&["condition_id"], "condition", OnDelete::Cascade)],
                checks: &[Check::After("closes_at", "opens_at")],
                without_natural_key: Some("the parameters of one leaf"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "condition_group",
                columns: &[
                    Column::new("condition_id", Type::Uuid),
                    Column::new("group_id", Type::Uuid).nullable(),
                    Column::new("group_set_id", Type::Uuid).nullable(),
                ],
                primary_key: &["condition_id"],
                foreign_keys: &[
                    own(&["condition_id"], "condition", OnDelete::Cascade),
                    other(&["group_id"], "courses", "group", OnDelete::Cascade),
                    other(&["group_set_id"], "courses", "group_set", OnDelete::Cascade),
                ],
                without_natural_key: Some("the parameters of one leaf"),
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "condition_profile",
                columns: &[
                    Column::new("condition_id", Type::Uuid),
                    Column::new("field", Type::Text),
                    Column::new("operator", Type::Text),
                    Column::new("value", Type::Text).nullable(),
                ],
                primary_key: &["condition_id"],
                foreign_keys: &[own(&["condition_id"], "condition", OnDelete::Cascade)],
                checks: &[
                    Check::OneOf("field", crate::settings::conditions::PROFILE_FIELDS),
                    Check::OneOf("operator", crate::settings::conditions::PROFILE_OPERATORS),
                ],
                without_natural_key: Some("the parameters of one leaf"),
                ..Table::EMPTY
            }),
        ],
    },
];

/// How the nodes of a tree of conditions combine: every child, any child, the opposite of its only child, or
/// a leaf of a kind.
pub const OPERATORS: &[&str] = &["all", "any", "not", "leaf"];
