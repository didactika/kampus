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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a module offers to access conditions: a kind of condition, such as a date window or a grade, whose
//! leaves it keeps in its own table, pointing to what they depend on by a foreign key. Activities keeps the
//! tree and asks each kind about its leaves, so it never depends on the modules that provide them.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use kampus_common::Result;
use serde_json::Value;

use crate::ServiceContext;

/// What a call to a kind returns.
pub type ConditionFuture<'c, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'c>>;

/// A leaf to keep, in the transaction that writes the tree.
pub struct ConditionWrite<'c> {
    /// The leaf, a row of the tree.
    pub condition: uuid::Uuid,
    /// The course of the course module or section the condition is on; what a leaf depends on must be in it.
    pub course: uuid::Uuid,
    /// The parameters, as the person gave them.
    pub params: &'c Value,
    /// The transaction that writes the tree.
    pub conn: &'c mut sqlx::PgConnection,
    /// The services of the process.
    pub services: &'c ServiceContext,
}

impl fmt::Debug for ConditionWrite<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConditionWrite")
            .field("condition", &self.condition)
            .finish_non_exhaustive()
    }
}

/// A question about a leaf: its parameters, or whether a person meets it.
#[derive(Debug, Clone, Copy)]
pub struct ConditionCheck<'c> {
    /// The leaf.
    pub condition: uuid::Uuid,
    /// The person asked about; ignored when reading the parameters.
    pub account: uuid::Uuid,
    /// The services of the process.
    pub services: &'c ServiceContext,
}

/// A question about what depends on a course module.
#[derive(Debug, Clone, Copy)]
pub struct ConditionTarget<'c> {
    /// The course module.
    pub module: uuid::Uuid,
    /// The services of the process.
    pub services: &'c ServiceContext,
}

/// A kind of access condition a module provides.
#[derive(Clone, Copy)]
pub struct ConditionKindDeclaration {
    /// `<module>.<kind>`, unique in the build.
    pub code: &'static str,
    /// What it checks, for the people who restrict access.
    pub summary: &'static str,
    /// Checks the parameters and keeps the leaf; a `validation` error refuses the whole tree.
    pub store: for<'c> fn(ConditionWrite<'c>) -> ConditionFuture<'c, ()>,
    /// The parameters of a leaf, as they would be given again; `None` when the leaf lost what it depends on.
    pub params: for<'c> fn(ConditionCheck<'c>) -> ConditionFuture<'c, Option<Value>>,
    /// Whether a person meets the leaf.
    pub met: for<'c> fn(ConditionCheck<'c>) -> ConditionFuture<'c, bool>,
    /// The leaves that depend on a course module, so deleting it is refused or removes them.
    pub depending_on: for<'c> fn(ConditionTarget<'c>) -> ConditionFuture<'c, Vec<uuid::Uuid>>,
}

impl fmt::Debug for ConditionKindDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConditionKindDeclaration")
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}

/// A kind whose leaves never depend on a course module.
pub fn nothing_depends(_target: ConditionTarget<'_>) -> ConditionFuture<'_, Vec<uuid::Uuid>> {
    Box::pin(async { Ok(Vec::new()) })
}
