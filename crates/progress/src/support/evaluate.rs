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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The state of a person on a course module, from what the course module requires and what the person has
//! done. Nothing here reads the database, so every case has a worked example as a test.
//!
//! A course module with no rules asks for nothing, so its state is `not_started` until someone sets it with
//! an exception. With rules, the person is `in_progress` once any of them is met or any signal exists, and
//! done when all are met. A pass rule decides how it ends: a grade at or above the passing grade is
//! `done_passed` and a grade below it is `done_failed`, whatever else is left to do. A rule
//! of the type of the course module counts like the others, as the module of the type reports it.

use rust_decimal::Decimal;

use crate::{RuleKind, State};

/// What a person has done on a course module.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent thing a person did, read together by the rules"
)]
pub struct Facts {
    /// They viewed it.
    pub viewed: bool,
    /// They submitted something to it.
    pub submitted: bool,
    /// They marked it done.
    pub marked: bool,
    /// What counts of their grade, when they have one.
    pub grade: Option<Decimal>,
    /// The grade that passes; without one, any grade passes.
    pub pass_grade: Option<Decimal>,
    /// The rules of its type the course module requires: none when it requires none, and otherwise whether
    /// the person meets every one of them.
    pub type_rules: Option<bool>,
    /// The module of the type said something about the person on one of its rules.
    pub type_signal: bool,
}

/// Whether a rule is met by the facts.
fn met(kind: RuleKind, facts: &Facts) -> bool {
    match kind {
        RuleKind::Manual => facts.marked,
        RuleKind::View => facts.viewed,
        RuleKind::Submit => facts.submitted,
        RuleKind::Grade => facts.grade.is_some(),
        RuleKind::Pass => passed(facts).unwrap_or(false),
    }
}

/// Whether the grade passes: none when there is no grade.
fn passed(facts: &Facts) -> Option<bool> {
    let grade = facts.grade?;
    Some(facts.pass_grade.is_none_or(|pass| grade >= pass))
}

/// The state of a person from the rules of a course module and what they have done.
#[must_use]
pub fn evaluate(rules: &[RuleKind], facts: &Facts) -> State {
    let anything =
        facts.viewed || facts.submitted || facts.marked || facts.grade.is_some() || facts.type_signal;
    if rules.is_empty() && facts.type_rules.is_none() {
        return if anything {
            State::InProgress
        } else {
            State::NotStarted
        };
    }
    if rules.contains(&RuleKind::Pass) && passed(facts) == Some(false) {
        return State::DoneFailed;
    }
    if rules.iter().all(|rule| met(*rule, facts)) && facts.type_rules != Some(false) {
        return if rules.contains(&RuleKind::Pass) {
            State::DonePassed
        } else {
            State::Done
        };
    }
    if anything {
        State::InProgress
    } else {
        State::NotStarted
    }
}
