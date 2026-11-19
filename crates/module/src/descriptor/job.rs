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

//! The background work a module declares: job types, which the task queue runs with retries and leases,
//! and schedules, which enqueue a job of a type on a recurring rule.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use kampus_common::Result;
use serde_json::Value;

use crate::ServiceContext;

/// What a job handler returns.
pub type JobFuture<'j> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'j>>;

/// One run of a job: its id, its payload, and the services the server composed.
#[derive(Debug, Clone, Copy)]
pub struct JobRun<'j> {
    /// The job, for its progress and cancellation.
    pub job: uuid::Uuid,
    /// Its payload, which holds identifiers, never copies of personal data.
    pub payload: &'j Value,
    /// The services of the process.
    pub services: &'j ServiceContext,
}

/// A kind of job a module runs in the background.
#[derive(Clone, Copy)]
pub struct JobDeclaration {
    /// Its code, `<module>.<name>`.
    pub code: &'static str,
    /// What it does, for operators.
    pub summary: &'static str,
    /// How many times it runs before it is failed.
    pub max_attempts: u16,
    /// The wait before the first retry, in seconds; it doubles at each retry.
    pub backoff_seconds: u32,
    /// How long one run may take, in seconds, before its lease lets another worker take it.
    pub timeout_seconds: u32,
    /// The payload field whose value keeps jobs of this type one at a time, such as a course.
    pub concurrency: Option<&'static str>,
    /// Runs the job; it is called again on retries, so it does what is left and nothing twice.
    pub run: for<'j> fn(JobRun<'j>) -> JobFuture<'j>,
}

impl fmt::Debug for JobDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobDeclaration")
            .field("code", &self.code)
            .field("max_attempts", &self.max_attempts)
            .finish_non_exhaustive()
    }
}

/// A recurring rule that enqueues a job of a type, with an empty payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleDeclaration {
    /// The job type it enqueues.
    pub job: &'static str,
    /// Its default rule: minute, hour, day of month, month and day of week, as cron writes them.
    pub cron: &'static str,
    /// The time zone its rule is read in.
    pub time_zone: &'static str,
}
