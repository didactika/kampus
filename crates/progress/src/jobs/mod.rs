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

//! The jobs of progress, run by the task runner: the re-evaluation of the course modules whose rules changed,
//! and of the people of a course whose progress may have changed, both run when something is queued and every
//! five minutes as a safety net; and, daily, the people of the courses whose criteria depend on time.

use kampus_module::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};

use crate::Progress;

/// The jobs of this module.
pub const JOBS: &[JobDeclaration] = &[
    JobDeclaration {
        code: "progress.recalculate",
        summary: "Re-evaluate the people of the course modules whose rules changed",
        max_attempts: 5,
        backoff_seconds: 60,
        timeout_seconds: 900,
        concurrency: Some("all"),
        run: recalculate,
    },
    JobDeclaration {
        code: "progress.course_recalculate",
        summary: "Evaluate the people of a course whose progress may have changed",
        max_attempts: 5,
        backoff_seconds: 60,
        timeout_seconds: 900,
        concurrency: Some("all"),
        run: course_recalculate,
    },
    JobDeclaration {
        code: "progress.course_timed",
        summary: "Queue the people of the courses whose criteria depend on a date or a duration",
        max_attempts: 3,
        backoff_seconds: 300,
        timeout_seconds: 900,
        concurrency: Some("all"),
        run: course_timed,
    },
];

/// The schedules of this module.
pub const SCHEDULES: &[ScheduleDeclaration] = &[
    ScheduleDeclaration {
        job: "progress.recalculate",
        cron: "*/5 * * * *",
        time_zone: "UTC",
    },
    ScheduleDeclaration {
        job: "progress.course_recalculate",
        cron: "*/5 * * * *",
        time_zone: "UTC",
    },
    ScheduleDeclaration {
        job: "progress.course_timed",
        cron: "30 2 * * *",
        time_zone: "UTC",
    },
];

/// Runs the re-evaluation.
fn recalculate(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let progress = run.services.service::<Progress>()?;
        while progress.recalculate().await? > 0 {}
        Ok(())
    })
}

/// Evaluates the people queued in their courses.
fn course_recalculate(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let progress = run.services.service::<Progress>()?;
        while progress.recalculate_courses().await? > 0 {}
        Ok(())
    })
}

/// Queues the people whose progress depends on time.
fn course_timed(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Progress>()?.queue_timed().await?;
        Ok(())
    })
}
