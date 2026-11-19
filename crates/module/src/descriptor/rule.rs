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

//! What a module offers to rules: actions a rule runs when its event happens and its conditions hold,
//! each with typed parameters, the permission its owner needs, the payload fields it reads and the events
//! it may publish.

use std::fmt;

use serde_json::Value;

use crate::{Event, JobFuture, ServiceContext, ValueKind};

/// A parameter of a rule action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleParameter {
    /// Its key, unique in the action.
    pub key: &'static str,
    /// The kind of value it takes, with the default of an optional one.
    pub kind: ValueKind,
    /// Whether every rule gives it.
    pub required: bool,
}

/// One run of a rule action.
#[derive(Debug, Clone, Copy)]
pub struct RuleActionRun<'r> {
    /// The event the rule reacts to; what the action publishes names it as its cause.
    pub event: &'r Event,
    /// The parameters, each of its declared kind, with the defaults of those the rule leaves out.
    pub params: &'r Value,
    /// The principal the rule acts as: its owner.
    pub owner: uuid::Uuid,
    /// The services of the process.
    pub services: &'r ServiceContext,
}

/// An action a module offers to rules.
#[derive(Clone, Copy)]
pub struct RuleActionDeclaration {
    /// `<module>.<action>`, unique in the build.
    pub code: &'static str,
    /// What it does, for the people who write rules.
    pub summary: &'static str,
    /// Its parameters.
    pub params: &'static [RuleParameter],
    /// The payload fields the triggering event must have, which the action reads.
    pub needs: &'static [&'static str],
    /// The permission the owner of a rule holds, in the scope of the event, for the action to run.
    pub permission: &'static str,
    /// The event types it may publish; a rule that runs it cannot be triggered by them.
    pub publishes: &'static [&'static str],
    /// Runs it; it may run again for the same event, so it does nothing twice.
    pub run: for<'r> fn(RuleActionRun<'r>) -> JobFuture<'r>,
}

impl fmt::Debug for RuleActionDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuleActionDeclaration")
            .field("code", &self.code)
            .field("permission", &self.permission)
            .finish_non_exhaustive()
    }
}
