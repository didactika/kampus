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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade steps of the `customise` schema, in the neutral schema: terms and their overrides, feature
//! flags and their values, custom fields with their choices and typed values, the record of changes, and
//! layouts with their placements, menus with their items, and themes.

use kampus_db::Step;
use kampus_db::schema::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

/// The types of a custom field.
pub const FIELD_TYPES: &[&str] = &[
    "text", "integer", "date", "boolean", "choice", "choices", "account",
];

/// Who sees the value of a custom field.
pub const VISIBILITIES: &[&str] = &["public", "private"];

/// A foreign key to a scope of access, which takes the row with it.
const SCOPE: ForeignKey = ForeignKey {
    columns: &["scope_id"],
    module: Some("access"),
    table: "scope",
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

/// The upgrade steps of this module.
pub const STEPS: &[Step] = &[Step {
    sequence: 1,
    name: "terms, flags, custom fields and changes",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "term",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("forms", Type::Json("customise.term_forms")),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "term_override",
            columns: &[
                Column::new("term_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("language_code", Type::Text),
                Column::new("singular", Type::Text),
                Column::new("plural", Type::Text),
                Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["term_code", "scope_id", "language_code"],
            foreign_keys: &[own(&["term_code"], "term", &["code"], OnDelete::Cascade), SCOPE],
            without_natural_key: Some("one word per term, scope and language"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "flag",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("default_value", Type::Boolean),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "flag_value",
            columns: &[
                Column::new("flag_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("enabled", Type::Boolean),
                Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
            ],
            primary_key: &["flag_code", "scope_id"],
            foreign_keys: &[own(&["flag_code"], "flag", &["code"], OnDelete::Cascade), SCOPE],
            without_natural_key: Some("one value per flag and scope"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "field_kind",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "field",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("kind_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("code", Type::Text),
                Column::new("name", Type::Text),
                Column::new("type", Type::Text),
                Column::new("required", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("visibility", Type::Text).default(ColumnDefault::Text("public")),
                Column::new("max_length", Type::Integer).nullable(),
                Column::new("sort_key", Type::Text),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["id"],
            unique: &[&["kind_code", "scope_id", "code"]],
            foreign_keys: &[own(&["kind_code"], "field_kind", &["code"], OnDelete::Cascade), SCOPE],
            checks: &[
                Check::OneOf("type", FIELD_TYPES),
                Check::OneOf("visibility", VISIBILITIES),
                Check::Positive("max_length"),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "field_choice",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("field_id", Type::Uuid),
                Column::new("code", Type::Text),
                Column::new("label", Type::Text),
                Column::new("sort_key", Type::Text),
            ],
            primary_key: &["id"],
            unique: &[&["field_id", "code"]],
            foreign_keys: &[own(&["field_id"], "field", &["id"], OnDelete::Cascade)],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "field_value",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("field_id", Type::Uuid),
                Column::new("text_value", Type::Text).nullable(),
                Column::new("integer_value", Type::BigInt).nullable(),
                Column::new("date_value", Type::Date).nullable(),
                Column::new("boolean_value", Type::Boolean).nullable(),
                Column::new("choice_id", Type::Uuid).nullable(),
                Column::new("account_id", Type::Uuid).nullable(),
                Column::new("updated_at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                own(&["field_id"], "field", &["id"], OnDelete::Cascade),
                own(&["choice_id"], "field_choice", &["id"], OnDelete::Cascade),
                ForeignKey {
                    columns: &["account_id"],
                    module: Some("identity"),
                    table: "account",
                    references: &["id"],
                    on_delete: OnDelete::Cascade,
                },
            ],
            indexes: &[Index {
                columns: &["field_id"],
                unique: false,
            }],
            without_natural_key: Some("one value of a field, owned through the link table of the module of its entity"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "field_value_choice",
            columns: &[
                Column::new("field_value_id", Type::Uuid),
                Column::new("choice_id", Type::Uuid),
            ],
            primary_key: &["field_value_id", "choice_id"],
            foreign_keys: &[
                own(&["field_value_id"], "field_value", &["id"], OnDelete::Cascade),
                own(&["choice_id"], "field_choice", &["id"], OnDelete::Cascade),
            ],
            without_natural_key: Some("the choices of a multiple-choice value"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "change",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("kind", Type::Text),
                Column::new("object", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("actor", Type::Uuid).nullable(),
                Column::new("before", Type::Json("customise.change")).nullable(),
                Column::new("after", Type::Json("customise.change")).nullable(),
                Column::new("at", Type::Timestamptz).default(ColumnDefault::Now),
            ],
            primary_key: &["id"],
            foreign_keys: &[SCOPE],
            indexes: &[Index {
                columns: &["scope_id", "at"],
                unique: false,
            }],
            without_natural_key: Some("an append-only record of changes"),
            ..Table::EMPTY
        }),
        Change::Postgres {
            sql: "alter table {schema}.\"field\" alter column \"sort_key\" type text collate \"C\";
alter table {schema}.\"field_choice\" alter column \"sort_key\" type text collate \"C\";
alter table {schema}.\"field_value\" add constraint \"field_value_at_most_one_check\" check (num_nonnulls(\"text_value\", \"integer_value\", \"date_value\", \"boolean_value\", \"choice_id\", \"account_id\") <= 1);",
            reason: "the neutral schema has no collation; a multiple-choice value keeps its choices in field_value_choice, so it sets none of the typed columns",
        },
    ],
}, Step {
    sequence: 2,
    name: "layouts, menus and themes",
    revert: &[],
    action: None,
    changes: &[
        Change::CreateTable(Table {
            name: "page_kind",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("regions", Type::Json("customise.page_regions")),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "widget",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("page_kinds", Type::Json("customise.widget_page_kinds")),
                Column::new("settings", Type::Json("customise.widget_settings")),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "menu_kind",
            columns: &[
                Column::new("code", Type::Text),
                Column::new("module", Type::Text),
                Column::new("summary", Type::Text),
                Column::new("retired_at", Type::Timestamptz).nullable(),
            ],
            primary_key: &["code"],
            without_natural_key: Some("keyed by the code its module declares"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "layout",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("page_kind_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["id"],
            unique: &[&["page_kind_code", "scope_id"]],
            foreign_keys: &[own(&["page_kind_code"], "page_kind", &["code"], OnDelete::Cascade), SCOPE],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "placement",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("layout_id", Type::Uuid),
                Column::new("region", Type::Text),
                Column::new("widget_code", Type::Text),
                Column::new("settings", Type::Json("customise.placement_settings")),
                Column::new("permission", Type::Text).nullable(),
                Column::new("position", Type::Integer),
            ],
            primary_key: &["id"],
            unique: &[&["layout_id", "region", "position"]],
            foreign_keys: &[
                own(&["layout_id"], "layout", &["id"], OnDelete::Cascade),
                own(&["widget_code"], "widget", &["code"], OnDelete::Cascade),
            ],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "menu",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("menu_kind_code", Type::Text),
                Column::new("scope_id", Type::Uuid),
                Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["id"],
            unique: &[&["menu_kind_code", "scope_id"]],
            foreign_keys: &[own(&["menu_kind_code"], "menu_kind", &["code"], OnDelete::Cascade), SCOPE],
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "menu_item",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("menu_id", Type::Uuid),
                Column::new("parent_id", Type::Uuid).nullable(),
                Column::new("label", Type::Text),
                Column::new("target", Type::Text),
                Column::new("permission", Type::Text).nullable(),
                Column::new("position", Type::Integer),
            ],
            primary_key: &["id"],
            foreign_keys: &[
                own(&["menu_id"], "menu", &["id"], OnDelete::Cascade),
                own(&["parent_id"], "menu_item", &["id"], OnDelete::Cascade),
            ],
            indexes: &[Index {
                columns: &["menu_id", "parent_id", "position"],
                unique: false,
            }],
            without_natural_key: Some("an item of a menu, replaced with the whole menu"),
            ..Table::EMPTY
        }),
        Change::CreateTable(Table {
            name: "theme",
            columns: &[
                Column::new("id", Type::Uuid),
                Column::new("scope_id", Type::Uuid),
                Column::new("tokens", Type::Json("customise.theme_tokens")),
                Column::new("locked", Type::Boolean).default(ColumnDefault::Bool(false)),
                Column::new("version", Type::Integer).default(ColumnDefault::Int(1)),
            ],
            primary_key: &["id"],
            unique: &[&["scope_id"]],
            foreign_keys: &[SCOPE],
            ..Table::EMPTY
        }),
    ],
}];
