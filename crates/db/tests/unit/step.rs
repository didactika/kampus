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

//! Tests of `kampus-db`: upgrade step.

use kampus_db::Prefix;
use kampus_db::Step;
use kampus_db::schema::Change;
use sha2::{Digest, Sha256};

/// A step referencing its own schema and an earlier module's.
const STEP: Step = Step {
    sequence: 1,
    name: "create group",
    revert: &[],
    action: None,
    changes: &[Change::Postgres {
        sql: "create table {schema}.\"group\" (scope_id uuid references {schema:access}.scope (id));",
        reason: "a test of placeholders",
    }],
};

#[test]
fn placeholders_become_prefixed_quoted_schemas() {
    let prefix: Prefix = "t1_".parse().unwrap();
    assert_eq!(
        STEP.render("courses", &prefix),
        "create table \"t1_courses\".\"group\" (scope_id uuid references \"t1_access\".scope (id));"
    );
}

#[test]
fn the_checksum_changes_with_the_sql_and_not_with_the_prefix() {
    let changed = Step {
        changes: &[Change::Postgres {
            sql: "create table {schema}.other ();",
            reason: "test",
        }],
        ..STEP
    };
    assert_eq!(STEP.checksum().len(), 32);
    assert_ne!(STEP.checksum(), changed.checksum());
    assert_eq!(
        STEP.checksum(),
        Sha256::digest(STEP.changes[0].postgres_template().as_bytes()).to_vec()
    );
}
