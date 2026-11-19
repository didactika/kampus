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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The jobs of identity, run by the task runner: deletions whose grace period ended, and the purge of
//! expired tokens, sessions and old sign-in attempts.

pub mod delete_account;
pub mod purge;

use kampus_module::{JobDeclaration, JobFuture, JobRun, ScheduleDeclaration};

use crate::Identity;

/// The jobs of identity.
pub const JOBS: &[JobDeclaration] = &[
    JobDeclaration {
        code: "identity.purge",
        summary: "Remove expired one-time tokens and sessions, and sign-in attempts past their retention",
        max_attempts: 3,
        backoff_seconds: 300,
        timeout_seconds: 600,
        concurrency: Some("all"),
        run: run_purge,
    },
    JobDeclaration {
        code: "identity.run_deletions",
        summary: "Delete the accounts whose grace period ended, leaving their tombstones",
        max_attempts: 5,
        backoff_seconds: 300,
        timeout_seconds: 1800,
        concurrency: Some("all"),
        run: run_deletions,
    },
];

/// The schedules of identity.
pub const SCHEDULES: &[ScheduleDeclaration] = &[
    ScheduleDeclaration {
        job: "identity.purge",
        cron: "0 3 * * *",
        time_zone: "UTC",
    },
    ScheduleDeclaration {
        job: "identity.run_deletions",
        cron: "15 * * * *",
        time_zone: "UTC",
    },
];

/// Runs the purge.
fn run_purge(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Identity>()?.purge().await?;
        Ok(())
    })
}

/// Runs the deletions that are due.
fn run_deletions(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Identity>()?.run_due_deletions().await?;
        Ok(())
    })
}
