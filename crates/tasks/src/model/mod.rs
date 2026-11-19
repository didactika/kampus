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
// @package   kampus-tasks
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The entities of tasks: jobs with their state and attempts, and schedules.

use kampus_common::Timestamp;
use serde::Serialize;
use serde_json::Value;

kampus_common::define_id! {
    /// Identifies a job.
    pub struct JobId;
}

kampus_common::define_id! {
    /// Identifies a schedule.
    pub struct ScheduleId;
}

/// The state of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Waiting for a worker.
    Queued,
    /// Run by a worker that holds its lease.
    Running,
    /// Done.
    Succeeded,
    /// Ran out of attempts; its last error is kept.
    Failed,
    /// Cancelled before it finished; what it finished stays.
    Cancelled,
}

impl JobState {
    /// The state as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Reads a stored state.
    #[must_use]
    pub fn read(text: &str) -> Self {
        match text {
            "running" => Self::Running,
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Queued,
        }
    }
}

/// A job.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Job {
    /// The job.
    pub id: JobId,
    /// Its type.
    pub type_code: String,
    /// Its payload.
    pub payload: Value,
    /// Its state.
    pub state: JobState,
    /// How many times it started.
    pub attempts: i32,
    /// When it may run.
    pub available_at: Timestamp,
    /// Its progress, in percent.
    pub progress: i32,
    /// Whether someone asked to cancel it.
    pub cancel_requested: bool,
    /// The error of its last failed attempt.
    pub last_error: Option<String>,
    /// When it was enqueued.
    pub created_at: Timestamp,
    /// When it finished.
    pub finished_at: Option<Timestamp>,
}

/// One execution of a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Attempt {
    /// The worker that ran it.
    pub worker: String,
    /// When it started.
    pub started_at: Timestamp,
    /// When it ended.
    pub ended_at: Option<Timestamp>,
    /// How it ended.
    pub outcome: Option<String>,
    /// Its error.
    pub error: Option<String>,
}

/// A schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Schedule {
    /// The schedule.
    pub id: ScheduleId,
    /// The job type it enqueues.
    pub type_code: String,
    /// Its rule.
    pub cron: String,
    /// The time zone of its rule.
    pub time_zone: String,
    /// Whether it fires.
    pub enabled: bool,
    /// When it fires next.
    pub next_run_at: Timestamp,
    /// When it fired last.
    pub last_run_at: Option<Timestamp>,
    /// The version, for `If-Match`.
    pub version: i32,
}
