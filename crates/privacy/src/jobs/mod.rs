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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The jobs of privacy: answering the parts of requests, and forgetting exported data when it expires.

use kampus_module::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};

use crate::Privacy;

/// The jobs of this module.
pub const JOBS: &[JobDeclaration] = &[
    JobDeclaration {
        code: "privacy.run",
        summary: "Ask each module for its part of the pending privacy requests",
        max_attempts: 5,
        backoff_seconds: 60,
        timeout_seconds: 900,
        concurrency: Some("all"),
        run: run_requests,
    },
    JobDeclaration {
        code: "privacy.expire",
        summary: "Forget the exported data whose time ended",
        max_attempts: 3,
        backoff_seconds: 300,
        timeout_seconds: 600,
        concurrency: Some("all"),
        run: expire,
    },
];

/// The schedules of this module.
pub const SCHEDULES: &[ScheduleDeclaration] = &[
    ScheduleDeclaration {
        job: "privacy.run",
        cron: "*/5 * * * *",
        time_zone: "UTC",
    },
    ScheduleDeclaration {
        job: "privacy.expire",
        cron: "15 4 * * *",
        time_zone: "UTC",
    },
];

/// Answers the pending parts.
fn run_requests(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let privacy = run.services.service::<Privacy>()?;
        while privacy.run(run.services).await? > 0 {}
        Ok(())
    })
}

/// Forgets what expired.
fn expire(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Privacy>()?.expire().await?;
        Ok(())
    })
}
