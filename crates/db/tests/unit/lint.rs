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

//! Tests of `kampus-db`: lint.

use kampus_db::lint::lint;
use kampus_db::schema::{Change, Column, ColumnDefault, ForeignKey, OnDelete, Table, Type};
use kampus_db::{ModuleSteps, Step};

/// A module that follows every convention.
const GOOD: ModuleSteps = ModuleSteps {
    module: "people",
    steps: &[Step {
        sequence: 1,
        name: "people",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "person",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("email", Type::Text),
                    Column::new("created_at", Type::Timestamptz).default(ColumnDefault::Now),
                ],
                primary_key: &["id"],
                unique: &[&["email"]],
                ..Table::EMPTY
            }),
            Change::CreateTable(Table {
                name: "friend",
                columns: &[
                    Column::new("person_id", Type::Uuid),
                    Column::new("friend_id", Type::Uuid),
                ],
                primary_key: &["person_id", "friend_id"],
                foreign_keys: &[
                    ForeignKey {
                        columns: &["person_id"],
                        module: None,
                        table: "person",
                        references: &["id"],
                        on_delete: OnDelete::Cascade,
                    },
                    ForeignKey {
                        columns: &["friend_id"],
                        module: None,
                        table: "person",
                        references: &["id"],
                        on_delete: OnDelete::Cascade,
                    },
                ],
                ..Table::EMPTY
            }),
        ],
    }],
};

/// A module that breaks most conventions.
const BAD: ModuleSteps = ModuleSteps {
    module: "notes",
    steps: &[Step {
        sequence: 1,
        name: "notes",
        revert: &[],
        action: None,
        changes: &[
            Change::CreateTable(Table {
                name: "Note",
                columns: &[
                    Column::new("id", Type::Uuid),
                    Column::new("author_id", Type::Uuid),
                    Column::new("created_at", Type::BigInt),
                    Column::new("is_draft", Type::Integer),
                    Column::new("data", Type::Json("")),
                    Column::new("course_id", Type::Uuid),
                ],
                primary_key: &["id"],
                foreign_keys: &[ForeignKey {
                    columns: &["course_id"],
                    module: Some("courses"),
                    table: "course",
                    references: &["id"],
                    on_delete: OnDelete::Restrict,
                }],
                ..Table::EMPTY
            }),
            Change::AddColumn {
                table: "Note",
                column: Column::new("body", Type::Text),
            },
            Change::DropColumn {
                table: "Note",
                column: "data",
                unused_since: "0.2.0",
            },
            Change::Postgres {
                sql: "select 1;",
                reason: " ",
            },
        ],
    }],
};

#[test]
fn a_module_that_follows_the_conventions_passes() {
    assert_eq!(lint(&[GOOD], "0.1.0"), []);
}

#[test]
fn every_broken_convention_is_reported() {
    let findings = lint(&[GOOD, BAD], "0.1.0");
    let rules: Vec<&str> = findings.iter().map(|f| f.rule).collect();
    for rule in ["L-01", "L-03", "L-04", "L-05", "L-06", "L-07", "L-08", "L-09"] {
        assert!(rules.contains(&rule), "{rule} missing in {findings:?}");
    }
    assert!(
        findings
            .iter()
            .any(|f| f.rule == "L-04" && f.detail.contains("is_draft"))
    );
    assert!(
        findings
            .iter()
            .any(|f| f.rule == "L-01" && f.detail.contains("author_id"))
    );
}
