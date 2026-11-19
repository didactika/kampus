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

//! The processes that run background work: workers claim and run jobs, and the scheduler fires the
//! schedules that are due. Both stop when told to, after the job or tick in hand.

use std::collections::HashMap;
use std::time::Duration;

use kampus_common::Result;
use kampus_module::{JobDeclaration, JobRun, ServiceContext};
use tokio::sync::watch;

use crate::Tasks;
use crate::service::queue::Ended;

/// How long an idle worker waits before it looks for work again.
const IDLE: Duration = Duration::from_secs(2);

/// How often the scheduler looks for schedules that are due.
const TICK: Duration = Duration::from_secs(20);

/// Claims and runs jobs of the types this build declares.
#[derive(Clone)]
pub struct Worker {
    /// The queue.
    tasks: Tasks,
    /// The handlers, by job type.
    handlers: HashMap<&'static str, JobDeclaration>,
    /// The services the handlers use.
    services: ServiceContext,
    /// The name of this worker, kept with each attempt.
    name: String,
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl Worker {
    /// A worker for the job types these declarations give.
    #[must_use]
    pub fn new(
        tasks: Tasks,
        jobs: impl IntoIterator<Item = JobDeclaration>,
        services: ServiceContext,
        name: String,
    ) -> Self {
        Self {
            tasks,
            handlers: jobs.into_iter().map(|j| (j.code, j)).collect(),
            services,
            name,
        }
    }

    /// Recovers the jobs of stopped workers, then claims and runs one job; whether it found one.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; the job's own failure is recorded, not returned.
    pub async fn run_once(&self) -> Result<bool> {
        self.tasks.recover_lost().await?;
        let types: Vec<String> = self.handlers.keys().map(|k| (*k).to_owned()).collect();
        let Some(job) = self.tasks.claim(&self.name, &types).await? else {
            return Ok(false);
        };
        let Some(handler) = self.handlers.get(job.type_code.as_str()) else {
            return Ok(false);
        };
        let run = JobRun {
            job: job.id.as_uuid(),
            payload: &job.payload,
            services: &self.services,
        };
        let timeout = Duration::from_secs(u64::try_from(job.timeout_seconds).unwrap_or(60));
        let ended = match tokio::time::timeout(timeout, (handler.run)(run)).await {
            Ok(Ok(())) => Ended::Succeeded,
            Ok(Err(error)) => Ended::Failed(error.to_string()),
            Err(_) => Ended::TimedOut,
        };
        self.tasks.finish(&job, ended).await?;
        Ok(true)
    }

    /// Runs jobs until `stop` turns true.
    pub async fn run(self, mut stop: watch::Receiver<bool>) {
        tracing::info!(worker = self.name, types = self.handlers.len(), "worker started");
        while !*stop.borrow() {
            let found = match self.run_once().await {
                Ok(found) => found,
                Err(error) => {
                    tracing::error!(%error, "worker could not reach the queue");
                    false
                }
            };
            if !found {
                tokio::select! {
                    () = tokio::time::sleep(IDLE) => {}
                    _ = stop.changed() => {}
                }
            }
        }
        tracing::info!(worker = self.name, "worker stopped");
    }
}

/// Fires the schedules that are due.
#[derive(Debug, Clone)]
pub struct Scheduler {
    /// The queue.
    tasks: Tasks,
}

impl Scheduler {
    /// A scheduler of the installation.
    #[must_use]
    pub const fn new(tasks: Tasks) -> Self {
        Self { tasks }
    }

    /// Fires due schedules until `stop` turns true.
    pub async fn run(self, mut stop: watch::Receiver<bool>) {
        tracing::info!("scheduler started");
        while !*stop.borrow() {
            match self.tasks.tick().await {
                Ok(0) => {}
                Ok(enqueued) => tracing::info!(enqueued, "schedules fired"),
                Err(error) => tracing::error!(%error, "scheduler could not fire its schedules"),
            }
            tokio::select! {
                () = tokio::time::sleep(TICK) => {}
                _ = stop.changed() => {}
            }
        }
        tracing::info!("scheduler stopped");
    }
}
