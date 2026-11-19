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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The jobs of enrolment, run by the task runner: system group instances kept in step with their groups,
//! and the expiry rule applied to ended enrolments.

use kampus_module::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};

use crate::Enrol;

/// The jobs of enrolment.
pub const JOBS: &[JobDeclaration] = &[
    JobDeclaration {
        code: "enrol.sync",
        summary: "Bring the enrolments of every system group instance in step with its group",
        max_attempts: 5,
        backoff_seconds: 60,
        timeout_seconds: 1800,
        concurrency: Some("all"),
        run: run_sync,
    },
    JobDeclaration {
        code: "enrol.run_expiry",
        summary: "Apply the expiry rule to the enrolments whose end has passed",
        max_attempts: 5,
        backoff_seconds: 60,
        timeout_seconds: 1800,
        concurrency: Some("all"),
        run: run_expiry,
    },
];

/// The schedules of enrolment.
pub const SCHEDULES: &[ScheduleDeclaration] = &[
    ScheduleDeclaration {
        job: "enrol.sync",
        cron: "*/15 * * * *",
        time_zone: "UTC",
    },
    ScheduleDeclaration {
        job: "enrol.run_expiry",
        cron: "5 * * * *",
        time_zone: "UTC",
    },
];

/// Runs the sync.
fn run_sync(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Enrol>()?.sync_all().await?;
        Ok(())
    })
}

/// Runs the expiry.
fn run_expiry(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Enrol>()?.run_expiry().await?;
        Ok(())
    })
}
