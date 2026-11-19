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

//! The schema linter (rules L-01 to L-09 of the data model conventions): it replays the declared steps of
//! every module, in dependency order, and reports what breaks the conventions, before any database sees it.

use std::collections::HashMap;
use std::fmt;

use crate::ModuleSteps;
use crate::schema::{Change, Column, Table, Type};

/// Something in a module's schema that breaks a convention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The rule (`L-01`…).
    pub rule: &'static str,
    /// The module.
    pub module: &'static str,
    /// The table, or the step for changes without a table.
    pub table: String,
    /// What is wrong.
    pub detail: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}.{}: {}", self.rule, self.module, self.table, self.detail)
    }
}

/// What the linter knows about a table after replaying the steps.
#[derive(Debug, Default)]
struct Model {
    /// Columns and their types.
    columns: Vec<(&'static str, Type)>,
    /// The primary key.
    primary_key: Vec<&'static str>,
    /// Whether the table has a natural key or a unique index.
    has_unique: bool,
    /// The columns covered by foreign keys.
    referencing: Vec<&'static str>,
    /// Why the table has no natural key, if it says so.
    without_natural_key: bool,
}

/// Checks the steps of every module, given in dependency order. `oldest_supported` is the oldest release
/// still supported: a column may be dropped only when it stopped being read before it.
#[must_use]
pub fn lint(modules: &[ModuleSteps], oldest_supported: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut tables: HashMap<(&'static str, &'static str), Model> = HashMap::new();
    for (position, module) in modules.iter().enumerate() {
        let earlier: Vec<&str> = modules[..position].iter().map(|m| m.module).collect();
        let mut report = |rule: &'static str, table: &str, detail: String| {
            findings.push(Finding {
                rule,
                module: module.module,
                table: table.to_owned(),
                detail,
            });
        };
        for step in module.steps {
            for change in step.changes {
                match change {
                    Change::CreateTable(table) => {
                        let model = model_of(module.module, &earlier, table, &tables, &mut report);
                        tables.insert((module.module, table.name), model);
                    }
                    Change::AddColumn { table, column } => {
                        check_column(table, column, &mut report);
                        if !column.nullable && column.default.is_none() {
                            report(
                                "L-08",
                                table,
                                format!("required column {} added without a default", column.name),
                            );
                        }
                        if let Some(model) = tables.get_mut(&(module.module, *table)) {
                            model.columns.push((column.name, column.ty));
                        }
                    }
                    Change::AddForeignKey { table, foreign_key } => {
                        check_reference(
                            module.module,
                            &earlier,
                            table,
                            foreign_key.module,
                            foreign_key.table,
                            &tables,
                            &mut report,
                        );
                        if let Some(model) = tables.get_mut(&(module.module, *table)) {
                            model.referencing.extend(foreign_key.columns);
                        }
                    }
                    Change::AddIndex { table, index } if index.unique => {
                        if let Some(model) = tables.get_mut(&(module.module, *table)) {
                            model.has_unique = true;
                        }
                    }
                    Change::DropColumn {
                        table,
                        column,
                        unused_since,
                    } => {
                        if !older(unused_since, oldest_supported) {
                            report(
                                "L-08",
                                table,
                                format!(
                                    "{column} is dropped while release {unused_since}, still supported, may read it"
                                ),
                            );
                        }
                        if let Some(model) = tables.get_mut(&(module.module, *table)) {
                            model.columns.retain(|(name, _)| name != column);
                        }
                    }
                    Change::Postgres { reason, .. } if reason.trim().is_empty() => {
                        report(
                            "L-09",
                            step.name,
                            "SQL outside the neutral schema without a reason".to_owned(),
                        );
                    }
                    Change::AddIndex { .. } | Change::AddCheck { .. } | Change::Postgres { .. } => {}
                }
            }
        }
    }
    check_tables(&tables, &mut findings);
    findings.sort_by(|a, b| (a.module, &a.table, a.rule).cmp(&(b.module, &b.table, b.rule)));
    findings
}

/// Checks a new table and returns what the linter keeps about it.
fn model_of(
    module: &'static str,
    earlier: &[&str],
    table: &Table,
    tables: &HashMap<(&'static str, &'static str), Model>,
    report: &mut impl FnMut(&'static str, &str, String),
) -> Model {
    check_names(table, report);
    for column in table.columns {
        check_column(table.name, column, report);
    }
    let mut model = Model {
        columns: table.columns.iter().map(|c| (c.name, c.ty)).collect(),
        primary_key: table.primary_key.to_vec(),
        has_unique: !table.unique.is_empty() || table.indexes.iter().any(|i| i.unique),
        without_natural_key: table.without_natural_key.is_some_and(|reason| !reason.is_empty()),
        ..Model::default()
    };
    for fk in table.foreign_keys {
        check_reference(module, earlier, table.name, fk.module, fk.table, tables, report);
        model.referencing.extend(fk.columns);
    }
    model
}

/// L-01 and L-03, once every step is replayed: references have foreign keys, and tables have a natural key.
fn check_tables(tables: &HashMap<(&'static str, &'static str), Model>, findings: &mut Vec<Finding>) {
    for ((module, table), model) in tables {
        for (column, _) in &model.columns {
            if column.ends_with("_id") && !model.referencing.contains(column) {
                findings.push(Finding {
                    rule: "L-01",
                    module,
                    table: (*table).to_owned(),
                    detail: format!("{column} looks like a reference but has no foreign key"),
                });
            }
        }
        let association =
            model.primary_key.len() >= 2 && model.primary_key.iter().all(|c| model.referencing.contains(c));
        if !model.has_unique && !association && !model.without_natural_key {
            findings.push(Finding {
                rule: "L-03",
                module,
                table: (*table).to_owned(),
                detail: "no natural key, and no reason given for it".to_owned(),
            });
        }
    }
}

/// L-07: names are lower-case snake case within the 63 bytes of PostgreSQL.
fn check_names(table: &Table, report: &mut impl FnMut(&'static str, &str, String)) {
    for name in std::iter::once(table.name).chain(table.columns.iter().map(|c| c.name)) {
        let valid = (1..=63).contains(&name.len())
            && name.as_bytes()[0].is_ascii_lowercase()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !valid {
            report(
                "L-07",
                table.name,
                format!("{name} is not a lower-case snake case name"),
            );
        }
    }
}

/// L-04 and L-05: instants are `timestamptz`, flags are booleans, JSON names its schema.
fn check_column(table: &str, column: &Column, report: &mut impl FnMut(&'static str, &str, String)) {
    if column.name.ends_with("_at") && column.ty != Type::Timestamptz {
        report(
            "L-04",
            table,
            format!("{} is an instant and must be timestamptz", column.name),
        );
    }
    if (column.name.starts_with("is_") || column.name.starts_with("has_")) && column.ty != Type::Boolean {
        report(
            "L-04",
            table,
            format!("{} is a flag and must be boolean", column.name),
        );
    }
    if matches!(column.ty, Type::Json(schema) if schema.trim().is_empty()) {
        report(
            "L-05",
            table,
            format!("{} is JSON without a declared schema", column.name),
        );
    }
}

/// L-06: a foreign key points to the same module or an earlier one, and to a table declared before.
#[allow(clippy::too_many_arguments, reason = "the context of one reference check")]
fn check_reference(
    module: &'static str,
    earlier: &[&str],
    table: &str,
    target_module: Option<&'static str>,
    target_table: &'static str,
    tables: &HashMap<(&'static str, &'static str), Model>,
    report: &mut impl FnMut(&'static str, &str, String),
) {
    let target = target_module.unwrap_or(module);
    if target != module && !earlier.contains(&target) {
        report(
            "L-06",
            table,
            format!("references {target}.{target_table}, a module that is not earlier"),
        );
    } else if !(tables.contains_key(&(target, target_table)) || (target == module && target_table == table)) {
        report(
            "L-06",
            table,
            format!("references {target}.{target_table}, which is not declared before"),
        );
    }
}

/// Whether release `a` is older than `b`, comparing `MAJOR.MINOR.PATCH`.
fn older(a: &str, b: &str) -> bool {
    /// The numbers of a version.
    fn parse(version: &str) -> Option<(u64, u64, u64)> {
        let mut parts = version
            .split(['-', '+'])
            .next()?
            .split('.')
            .map(str::parse::<u64>);
        Some((parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?))
    }
    matches!((parse(a), parse(b)), (Some(a), Some(b)) if a < b)
}
