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

//! The entities of progress: what a course module requires, the state of a person on it, and the
//! exceptions teachers set.

use kampus_access::PrincipalId;
use kampus_activities::ModuleId;
use kampus_common::Timestamp;
use serde::{Deserialize, Serialize};

/// What a course module can require of a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// They mark it done themselves.
    Manual,
    /// They view it.
    View,
    /// They submit something to it.
    Submit,
    /// They receive a grade.
    Grade,
    /// They reach the passing grade.
    Pass,
}

impl RuleKind {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::View => "view",
            Self::Submit => "submit",
            Self::Grade => "grade",
            Self::Pass => "pass",
        }
    }

    /// The rule a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        [Self::View, Self::Submit, Self::Grade, Self::Pass]
            .into_iter()
            .find(|k| k.code() == code)
            .unwrap_or(Self::Manual)
    }
}

/// The state of a person on a course module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// They have not done anything.
    NotStarted,
    /// They have done something.
    InProgress,
    /// They met every rule.
    Done,
    /// They met every rule and passed.
    DonePassed,
    /// They did not reach the passing grade.
    DoneFailed,
}

impl State {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::DonePassed => "done_passed",
            Self::DoneFailed => "done_failed",
        }
    }

    /// The state a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        [Self::InProgress, Self::Done, Self::DonePassed, Self::DoneFailed]
            .into_iter()
            .find(|s| s.code() == code)
            .unwrap_or(Self::NotStarted)
    }

    /// The state shown when what decides how it ended is hidden from the viewer: done, without saying
    /// whether they passed.
    #[must_use]
    pub const fn without_outcome(self) -> Self {
        match self {
            Self::DonePassed | Self::DoneFailed => Self::Done,
            other => other,
        }
    }
}

/// Where a state came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The rules and what the person did.
    Automatic,
    /// A teacher decided it.
    Exception,
}

impl Source {
    /// The code stored for it.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Exception => "exception",
        }
    }

    /// The source a stored code names.
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        if code == "exception" {
            Self::Exception
        } else {
            Self::Automatic
        }
    }
}

/// The state of a person on a course module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActivityState {
    /// The course module.
    pub course_module: ModuleId,
    /// The person.
    pub account: PrincipalId,
    /// Their state.
    pub state: State,
    /// Where it came from.
    pub source: Source,
    /// Since when.
    pub at: Timestamp,
    /// The version of the row.
    pub version: i32,
}

/// A teacher's decision about the state of a person on a course module, which wins over the rules until it
/// is removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Exception {
    /// The course module.
    pub course_module: ModuleId,
    /// The person.
    pub account: PrincipalId,
    /// The state it sets.
    pub state: State,
    /// Why.
    pub reason: String,
    /// Who decided it.
    pub author: Option<uuid::Uuid>,
    /// When.
    pub at: Timestamp,
}
