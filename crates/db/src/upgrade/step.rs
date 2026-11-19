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

//! Upgrade steps: each module declares its ordered steps as neutral schema changes, optionally with a data
//! action run in the same transaction and with the changes that undo the step; a released step is never
//! edited, which its checksum proves.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use kampus_common::Result;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;

use crate::Prefix;
use crate::schema::Change;

/// One upgrade step of a module, applied in one transaction: its changes, then its action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// The position of the step in its module, from 1, without gaps.
    pub sequence: u32,
    /// A short name of what the step does, for the ledger and the logs.
    pub name: &'static str,
    /// The changes that undo the step, in order, kept in the installation when the step is applied so an
    /// older release can go back; empty when the step cannot be undone.
    pub revert: &'static [Change],
    /// Data work that runs after the changes, in the same transaction: for the first step, what the
    /// component sets up when it is installed; for later ones, data migrations.
    pub action: Option<Action>,
    /// The changes of the step, in order.
    pub changes: &'static [Change],
}

/// What an action runs, while the step's transaction is open.
pub type ActionFuture<'c> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'c>>;

/// Data work of a step, written in Rust.
#[derive(Clone, Copy)]
pub struct Action {
    /// A short name, part of the step's checksum: renaming it counts as changing the step.
    pub name: &'static str,
    /// The work.
    pub run: for<'c> fn(&'c mut PgConnection, &'c ActionContext) -> ActionFuture<'c>,
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Action {}

/// Where an action runs.
#[derive(Debug, Clone)]
pub struct ActionContext {
    /// The module of the step.
    pub module: &'static str,
    /// The prefix of the installation.
    pub prefix: Prefix,
    /// The release being installed.
    pub release: String,
}

impl ActionContext {
    /// The quoted name of a table of the step's module.
    #[must_use]
    pub fn table(&self, name: &str) -> String {
        format!("{}.\"{name}\"", self.prefix.schema(self.module))
    }

    /// The quoted schema of another module the step's module may reference.
    #[must_use]
    pub fn schema(&self, module: &str) -> String {
        self.prefix.schema(module)
    }
}

impl Step {
    /// The step in PostgreSQL with schema placeholders: what the checksum covers, so the same step has
    /// the same checksum in every installation.
    #[must_use]
    pub fn postgres_template(&self) -> String {
        self.changes
            .iter()
            .map(Change::postgres_template)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The checksum of the step, recorded when it is applied: its changes and the name of its action.
    #[must_use]
    pub fn checksum(&self) -> Vec<u8> {
        let mut covered = self.postgres_template();
        if let Some(action) = self.action {
            covered.push_str(
                "
-- action: ",
            );
            covered.push_str(action.name);
        }
        Sha256::digest(covered.as_bytes()).to_vec()
    }

    /// The SQL that undoes the step for a module and an installation prefix, if the step can be undone.
    #[must_use]
    pub fn render_revert(&self, module: &str, prefix: &Prefix) -> Option<String> {
        if self.revert.is_empty() {
            return None;
        }
        let template = self
            .revert
            .iter()
            .map(Change::postgres_template)
            .collect::<Vec<_>>()
            .join(
                "
",
            );
        Some(fill(&template, module, prefix))
    }

    /// The SQL of the step for a module and an installation prefix: `{schema}` becomes the module's
    /// schema and `{schema:<module>}` another module's.
    #[must_use]
    pub fn render(&self, module: &str, prefix: &Prefix) -> String {
        fill(&self.postgres_template(), module, prefix)
    }
}

/// Fills the schema placeholders of a template: `{schema}` is the module's schema and `{schema:<module>}`
/// another module's.
fn fill(template: &str, module: &str, prefix: &Prefix) -> String {
    let mut sql = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{schema") {
        sql.push_str(&rest[..start]);
        let tail = &rest[start..];
        let Some(end) = tail.find('}') else {
            sql.push_str(tail);
            rest = "";
            break;
        };
        let target = tail["{schema".len()..end]
            .strip_prefix(':')
            .filter(|t| !t.is_empty())
            .unwrap_or(module);
        sql.push_str(&prefix.schema(target));
        rest = &tail[end + 1..];
    }
    sql.push_str(rest);
    sql
}

/// The upgrade steps of one module, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleSteps {
    /// The logical name of the module's schema (`courses`).
    pub module: &'static str,
    /// The steps, ordered by sequence.
    pub steps: &'static [Step],
}

#[cfg(test)]
mod tests {
    use super::Step;
    use crate::Prefix;
    use crate::schema::Change;

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
            <super::Sha256 as super::Digest>::digest(STEP.changes[0].postgres_template().as_bytes()).to_vec()
        );
    }
}
