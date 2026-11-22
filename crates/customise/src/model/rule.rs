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

//! Rules: when an event of a declared type happens in a scope and its conditions hold, the rule runs the
//! actions modules declare, as its owner; and the record of each evaluation.

use std::collections::BTreeMap;

use kampus_access::ScopeId;
use kampus_common::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

kampus_common::define_id! {
    /// Identifies a rule.
    pub struct RuleId;
}

/// How a field of the payload is compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    /// Equal to the value.
    Eq,
    /// Not equal to the value.
    Ne,
    /// One of the values of a list.
    In,
    /// Present and not null.
    Exists,
}

/// A comparison of a field of the payload of the event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Comparison {
    /// A field the event type declares.
    pub field: String,
    /// How it is compared.
    pub op: Operator,
    /// What it is compared with; a list for `in`, nothing for `exists`.
    #[serde(default)]
    pub value: Value,
}

/// When a rule runs: a tree of comparisons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// Every condition holds.
    All(Vec<Condition>),
    /// At least one condition holds.
    Any(Vec<Condition>),
    /// The condition does not hold.
    Not(Box<Condition>),
    /// A field of the payload compares as said.
    Compare(Comparison),
}

impl Condition {
    /// Whether it holds for a payload.
    #[must_use]
    pub fn holds(&self, payload: &Value) -> bool {
        match self {
            Self::All(all) => all.iter().all(|c| c.holds(payload)),
            Self::Any(any) => any.iter().any(|c| c.holds(payload)),
            Self::Not(not) => !not.holds(payload),
            Self::Compare(c) => {
                let field = payload.get(&c.field).filter(|v| !v.is_null());
                match c.op {
                    Operator::Eq => field == Some(&c.value),
                    Operator::Ne => field != Some(&c.value),
                    Operator::In => field.is_some_and(|f| c.value.as_array().is_some_and(|l| l.contains(f))),
                    Operator::Exists => field.is_some(),
                }
            }
        }
    }
}

/// An action of a rule, with its parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RuleAction {
    /// The action a module declares, by code.
    pub action: String,
    /// Its parameters, each of its declared kind.
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

/// A rule to write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RuleSpec {
    /// Its name.
    pub name: String,
    /// The event type that triggers it.
    pub trigger: String,
    /// When it runs; always when absent.
    #[serde(default)]
    pub conditions: Option<Condition>,
    /// What it does, in order.
    pub actions: Vec<RuleAction>,
    /// Whether it runs.
    #[serde(default = "enabled")]
    pub enabled: bool,
}

/// Rules run unless said otherwise.
const fn enabled() -> bool {
    true
}

/// A rule of a scope; it reacts to the events of its scope and of the scopes below.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Rule {
    /// The rule.
    pub id: RuleId,
    /// Its scope.
    pub scope: ScopeId,
    /// The principal it acts as: who wrote it last.
    pub owner: uuid::Uuid,
    /// Its name.
    pub name: String,
    /// The event type that triggers it.
    pub trigger: String,
    /// When it runs.
    pub conditions: Option<Condition>,
    /// What it does, in order.
    pub actions: Vec<RuleAction>,
    /// Whether it runs.
    pub enabled: bool,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// One evaluation of a rule for an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct RuleRun {
    /// The event.
    pub event: uuid::Uuid,
    /// `running`, `succeeded`, `failed`, or `skipped` when its conditions did not hold.
    pub outcome: String,
    /// How many of its actions ran.
    pub actions_done: i32,
    /// Why it failed.
    pub error: Option<String>,
    /// When it was evaluated.
    pub at: Timestamp,
}
