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

//! The neutral schema: modules declare their tables once, with the portable types of the port,
//! and the port writes the SQL of each dialect. Identifiers are always quoted, so tables may keep natural
//! names that are reserved words (`group`, `grant`), and schemas are written as placeholders that the
//! upgrade runner replaces with the installation's prefixed names.

use std::fmt::Write as _;

/// The portable column types; no other type is allowed in a module's schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// A UUID, for identifiers.
    Uuid,
    /// Text without a length limit.
    Text,
    /// Text of at most this many characters.
    Varchar(u16),
    /// A 32-bit integer.
    Integer,
    /// A 64-bit integer.
    BigInt,
    /// An exact number with precision and scale.
    Numeric(u8, u8),
    /// A boolean.
    Boolean,
    /// A calendar date.
    Date,
    /// An instant with time zone.
    Timestamptz,
    /// A duration.
    Interval,
    /// Bytes.
    Bytes,
    /// JSON validated by a declared schema, named here (lint L-05).
    Json(&'static str),
    /// A list of texts.
    TextArray,
}

impl Type {
    /// The type in PostgreSQL.
    fn postgres(self) -> String {
        match self {
            Self::Uuid => "uuid".into(),
            Self::Text => "text".into(),
            Self::Varchar(length) => format!("varchar({length})"),
            Self::Integer => "integer".into(),
            Self::BigInt => "bigint".into(),
            Self::Numeric(precision, scale) => format!("numeric({precision},{scale})"),
            Self::Boolean => "boolean".into(),
            Self::Date => "date".into(),
            Self::Timestamptz => "timestamptz".into(),
            Self::Interval => "interval".into(),
            Self::Bytes => "bytea".into(),
            Self::Json(_) => "jsonb".into(),
            Self::TextArray => "text[]".into(),
        }
    }
}

/// The default of a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnDefault {
    /// The current instant of the database.
    Now,
    /// A boolean.
    Bool(bool),
    /// An integer.
    Int(i64),
    /// A text, quoted by the port.
    Text(&'static str),
}

impl ColumnDefault {
    /// The default in PostgreSQL.
    fn postgres(self) -> String {
        match self {
            Self::Now => "now()".into(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Text(value) => format!("'{}'", value.replace('\'', "''")),
        }
    }
}

/// A column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    /// The name.
    pub name: &'static str,
    /// The type.
    pub ty: Type,
    /// Whether the column may be null; columns are `NOT NULL` unless absence has a meaning.
    pub nullable: bool,
    /// The default, if any.
    pub default: Option<ColumnDefault>,
}

impl Column {
    /// A required column without default.
    #[must_use]
    pub const fn new(name: &'static str, ty: Type) -> Self {
        Self {
            name,
            ty,
            nullable: false,
            default: None,
        }
    }

    /// The same column, nullable.
    #[must_use]
    pub const fn nullable(self) -> Self {
        Self {
            nullable: true,
            ..self
        }
    }

    /// The same column, with a default.
    #[must_use]
    pub const fn default(self, default: ColumnDefault) -> Self {
        Self {
            default: Some(default),
            ..self
        }
    }

    /// The column definition in PostgreSQL.
    fn postgres(&self) -> String {
        let mut sql = format!("{} {}", quote(self.name), self.ty.postgres());
        if !self.nullable {
            sql.push_str(" not null");
        }
        if let Some(default) = self.default {
            let _ = write!(sql, " default {}", default.postgres());
        }
        sql
    }
}

/// What happens to referencing rows when the referenced row is deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDelete {
    /// The deletion is refused while references exist (the default).
    Restrict,
    /// Owned children go with their parent.
    Cascade,
    /// Optional links are cleared.
    SetNull,
}

/// A foreign key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignKey {
    /// The referencing columns.
    pub columns: &'static [&'static str],
    /// The module of the referenced table; `None` for the same module.
    pub module: Option<&'static str>,
    /// The referenced table.
    pub table: &'static str,
    /// The referenced columns.
    pub references: &'static [&'static str],
    /// What happens on deletion of the referenced row.
    pub on_delete: OnDelete,
}

/// A rule every row must follow, written without dialect SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// The column holds one of these values (enumerations as checked values).
    OneOf(&'static str, &'static [&'static str]),
    /// The first column is later than, or greater than, the second, when both are set.
    After(&'static str, &'static str),
    /// The column is greater than zero.
    Positive(&'static str),
    /// The column is zero or more.
    NonNegative(&'static str),
    /// Exactly one of these columns is set: the exclusive arc.
    ExactlyOne(&'static [&'static str]),
}

impl Check {
    /// A short name of the check, for its constraint name.
    fn name(&self) -> String {
        match self {
            Self::OneOf(column, _) | Self::Positive(column) | Self::NonNegative(column) => {
                (*column).to_owned()
            }
            Self::After(later, earlier) => format!("{later}_{earlier}"),
            Self::ExactlyOne(columns) => columns.join("_"),
        }
    }

    /// The condition in PostgreSQL.
    fn postgres(&self) -> String {
        match self {
            Self::OneOf(column, values) => {
                let values: Vec<String> = values.iter().map(|v| ColumnDefault::Text(v).postgres()).collect();
                format!("{} in ({})", quote(column), values.join(", "))
            }
            Self::After(later, earlier) => format!("{} > {}", quote(later), quote(earlier)),
            Self::Positive(column) => format!("{} > 0", quote(column)),
            Self::NonNegative(column) => format!("{} >= 0", quote(column)),
            Self::ExactlyOne(columns) => {
                let columns: Vec<String> = columns.iter().map(|c| quote(c)).collect();
                format!("num_nonnulls({}) = 1", columns.join(", "))
            }
        }
    }
}

/// A secondary index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index {
    /// The indexed columns, in order.
    pub columns: &'static [&'static str],
    /// Whether the index enforces uniqueness.
    pub unique: bool,
}

/// A table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Table {
    /// The logical name, without module or prefix.
    pub name: &'static str,
    /// The columns.
    pub columns: &'static [Column],
    /// The primary key.
    pub primary_key: &'static [&'static str],
    /// The natural keys.
    pub unique: &'static [&'static [&'static str]],
    /// The foreign keys; each gets an index unless a key already starts with its columns.
    pub foreign_keys: &'static [ForeignKey],
    /// The rules of every row.
    pub checks: &'static [Check],
    /// Secondary indexes from the queries the module runs.
    pub indexes: &'static [Index],
    /// Why the table has no natural key, when it has none and is not a pure association (lint L-03).
    pub without_natural_key: Option<&'static str>,
}

impl Table {
    /// A table without columns, to fill with struct update syntax.
    pub const EMPTY: Self = Self {
        name: "",
        columns: &[],
        primary_key: &[],
        unique: &[],
        foreign_keys: &[],
        checks: &[],
        indexes: &[],
        without_natural_key: None,
    };
}

/// One change of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Creates a table with its keys, checks and indexes.
    CreateTable(Table),
    /// Adds a column to a table (expand).
    AddColumn {
        /// The table.
        table: &'static str,
        /// The new column; a required column needs a default so existing rows get a value.
        column: Column,
    },
    /// Adds a foreign key, and its index.
    AddForeignKey {
        /// The table.
        table: &'static str,
        /// The foreign key.
        foreign_key: ForeignKey,
    },
    /// Adds a check.
    AddCheck {
        /// The table.
        table: &'static str,
        /// The check.
        check: Check,
    },
    /// Adds an index.
    AddIndex {
        /// The table.
        table: &'static str,
        /// The index.
        index: Index,
    },
    /// Drops a column no supported release reads any more (the contract half of an expand and contract change).
    DropColumn {
        /// The table.
        table: &'static str,
        /// The column.
        column: &'static str,
        /// The release that stopped reading it; it must be out of support (lint L-08).
        unused_since: &'static str,
    },
    /// SQL for PostgreSQL only, allowed only with the reason the neutral schema cannot express it.
    Postgres {
        /// The SQL, with schema placeholders.
        sql: &'static str,
        /// Why it is not neutral.
        reason: &'static str,
    },
}

impl Change {
    /// The change in PostgreSQL, with schema placeholders: `{schema}` for the module's own schema and
    /// `{schema:<module>}` for another module's.
    #[must_use]
    pub fn postgres_template(&self) -> String {
        match self {
            Self::CreateTable(table) => create_table(table),
            Self::AddColumn { table, column } => format!(
                "alter table {{schema}}.{} add column {};",
                quote(table),
                column.postgres()
            ),
            Self::AddForeignKey { table, foreign_key } => {
                let mut sql = format!(
                    "alter table {{schema}}.{} add {};",
                    quote(table),
                    foreign_key_clause(table, foreign_key)
                );
                sql.push_str(&foreign_key_index(table, foreign_key, &[]));
                sql
            }
            Self::AddCheck { table, check } => format!(
                "alter table {{schema}}.{} add constraint {} check ({});",
                quote(table),
                quote(&constraint(table, &check.name(), "check")),
                check.postgres()
            ),
            Self::AddIndex { table, index } => index_statement(table, index),
            Self::DropColumn { table, column, .. } => format!(
                "alter table {{schema}}.{} drop column {};",
                quote(table),
                quote(column)
            ),
            Self::Postgres { sql, .. } => (*sql).to_owned(),
        }
    }
}

/// Quotes an identifier.
#[must_use]
pub fn quote(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// A constraint or index name: table, columns and kind, within the 63 bytes of PostgreSQL.
fn constraint(table: &str, columns: &str, kind: &str) -> String {
    let mut name = format!("{table}_{columns}_{kind}");
    name.truncate(63);
    name
}

/// The reference of a foreign key, with its schema placeholder.
fn referenced_table(foreign_key: &ForeignKey) -> String {
    match foreign_key.module {
        Some(module) => format!("{{schema:{module}}}.{}", quote(foreign_key.table)),
        None => format!("{{schema}}.{}", quote(foreign_key.table)),
    }
}

/// Quotes and joins a list of columns.
fn column_list(columns: &[&str]) -> String {
    columns.iter().map(|c| quote(c)).collect::<Vec<_>>().join(", ")
}

/// The constraint clause of a foreign key.
fn foreign_key_clause(table: &str, foreign_key: &ForeignKey) -> String {
    let on_delete = match foreign_key.on_delete {
        OnDelete::Restrict => "restrict",
        OnDelete::Cascade => "cascade",
        OnDelete::SetNull => "set null",
    };
    format!(
        "constraint {} foreign key ({}) references {} ({}) on delete {on_delete}",
        quote(&constraint(table, &foreign_key.columns.join("_"), "fkey")),
        column_list(foreign_key.columns),
        referenced_table(foreign_key),
        column_list(foreign_key.references),
    )
}

/// The index of a foreign key, unless one of `covered` (keys of the table) already starts with its columns.
fn foreign_key_index(table: &str, foreign_key: &ForeignKey, covered: &[&[&str]]) -> String {
    let leads = |key: &&[&str]| {
        key.len() >= foreign_key.columns.len() && key[..foreign_key.columns.len()] == *foreign_key.columns
    };
    if covered.iter().any(leads) {
        String::new()
    } else {
        index_statement(
            table,
            &Index {
                columns: foreign_key.columns,
                unique: false,
            },
        )
    }
}

/// The statement of an index.
fn index_statement(table: &str, index: &Index) -> String {
    let kind = if index.unique { "key" } else { "idx" };
    format!(
        "create {}index {} on {{schema}}.{} ({});",
        if index.unique { "unique " } else { "" },
        quote(&constraint(table, &index.columns.join("_"), kind)),
        quote(table),
        column_list(index.columns)
    )
}

/// The statements that create a table with its keys, checks and indexes.
fn create_table(table: &Table) -> String {
    let mut parts: Vec<String> = table.columns.iter().map(Column::postgres).collect();
    if !table.primary_key.is_empty() {
        parts.push(format!("primary key ({})", column_list(table.primary_key)));
    }
    for key in table.unique {
        parts.push(format!(
            "constraint {} unique ({})",
            quote(&constraint(table.name, &key.join("_"), "key")),
            column_list(key)
        ));
    }
    for foreign_key in table.foreign_keys {
        parts.push(foreign_key_clause(table.name, foreign_key));
    }
    for check in table.checks {
        parts.push(format!(
            "constraint {} check ({})",
            quote(&constraint(table.name, &check.name(), "check")),
            check.postgres()
        ));
    }
    let mut sql = format!(
        "create table {{schema}}.{} (\n    {}\n);",
        quote(table.name),
        parts.join(",\n    ")
    );
    let mut covered: Vec<&[&str]> = vec![table.primary_key];
    covered.extend(table.unique.iter().copied());
    covered.extend(table.indexes.iter().map(|i| i.columns));
    for foreign_key in table.foreign_keys {
        sql.push_str(&foreign_key_index(table.name, foreign_key, &covered));
    }
    for index in table.indexes {
        sql.push_str(&index_statement(table.name, index));
    }
    sql
}

#[cfg(test)]
mod tests {
    use super::{Change, Check, Column, ColumnDefault, ForeignKey, Index, OnDelete, Table, Type};

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
        assert!(
            sql.contains("create index \"group_created_at_idx\" on {schema}.\"group\" (\"created_at\");")
        );
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
}
