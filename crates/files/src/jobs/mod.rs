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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The jobs of files, run by the task runner: conversions, and the hourly purge of expired uploads, files
//! nobody attached and content past its time in the trash.

use kampus_module::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};

use crate::Files;

/// The jobs of this module.
pub const JOBS: &[JobDeclaration] = &[
    JobDeclaration {
        code: "files.convert",
        summary: "Make a conversion of some content, such as a thumbnail",
        max_attempts: 5,
        backoff_seconds: 30,
        timeout_seconds: 600,
        concurrency: None,
        run: convert,
    },
    JobDeclaration {
        code: "files.purge",
        summary: "Remove expired uploads, files nobody attached, and content past its time in the trash",
        max_attempts: 3,
        backoff_seconds: 300,
        timeout_seconds: 1800,
        concurrency: Some("all"),
        run: purge,
    },
];

/// The schedules of this module.
pub const SCHEDULES: &[ScheduleDeclaration] = &[ScheduleDeclaration {
    job: "files.purge",
    cron: "20 * * * *",
    time_zone: "UTC",
}];

/// Makes the conversion of the payload; a payload without one has nothing to do.
fn convert(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let Some(conversion) = run
            .payload
            .get("conversion")
            .and_then(serde_json::Value::as_str)
            .and_then(|c| uuid::Uuid::parse_str(c).ok())
        else {
            return Ok(());
        };
        run.services.service::<Files>()?.convert(conversion).await
    })
}

/// Runs the purge.
fn purge(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Files>()?.purge().await?;
        Ok(())
    })
}
