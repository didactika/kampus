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

//! What operators and long jobs do with jobs: read them with their attempts, retry a failed one, cancel
//! one, report and read progress, and purge finished jobs past their retention.

use kampus_common::{Error, Result};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::instant;
use crate::{Attempt, Job, JobId, JobState, Tasks, sql};

/// How long finished jobs and their attempts are kept, in days.
const RETENTION_DAYS: i32 = 30;

/// The columns of a job.
const COLUMNS: &str = "id, type_code, payload, state, attempts, available_at, progress, cancel_requested, last_error, \
    created_at, finished_at";

/// A job from its row.
fn job(row: &PgRow) -> Job {
    let epoch = || kampus_common::Timestamp::from_offset_date_time(time::OffsetDateTime::UNIX_EPOCH);
    Job {
        id: JobId::from_uuid(row.get("id")),
        type_code: row.get("type_code"),
        payload: row.get("payload"),
        state: JobState::read(row.get::<String, _>("state").as_str()),
        attempts: row.get("attempts"),
        available_at: instant(row, "available_at").unwrap_or_else(epoch),
        progress: row.get("progress"),
        cancel_requested: row.get("cancel_requested"),
        last_error: row.get("last_error"),
        created_at: instant(row, "created_at").unwrap_or_else(epoch),
        finished_at: instant(row, "finished_at"),
    }
}

impl Tasks {
    /// The most recent jobs, of one type or state or all, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn jobs(
        &self,
        type_code: Option<&str>,
        state: Option<JobState>,
        limit: i64,
    ) -> Result<Vec<Job>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where ($1::text is null or type_code = $1) and ($2::text is null or state = $2)
             order by created_at desc, id desc limit $3",
            self.table("job")
        )))
        .bind(type_code)
        .bind(state.map(JobState::as_str))
        .bind(limit.clamp(1, 500))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(job).collect())
    }

    /// A job with its attempts, oldest first.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown job.
    pub async fn job(&self, id: JobId) -> Result<(Job, Vec<Attempt>)> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1",
            self.table("job")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("tasks.job_not_found"))?;
        let attempts = sqlx::query(sql(format!(
            "select worker, started_at, ended_at, outcome, error from {} where job_id = $1 order by started_at, id",
            self.table("attempt")
        )))
        .bind(id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let epoch = || kampus_common::Timestamp::from_offset_date_time(time::OffsetDateTime::UNIX_EPOCH);
        Ok((
            job(&row),
            attempts
                .iter()
                .map(|a| Attempt {
                    worker: a.get("worker"),
                    started_at: instant(a, "started_at").unwrap_or_else(epoch),
                    ended_at: instant(a, "ended_at"),
                    outcome: a.get("outcome"),
                    error: a.get("error"),
                })
                .collect(),
        ))
    }

    /// Queues a failed job again, with its attempts from zero.
    ///
    /// # Errors
    ///
    /// `conflict` (`tasks.job_not_failed`) for a job that did not fail.
    pub async fn retry(&self, id: JobId) -> Result<()> {
        let changed = sqlx::query(sql(format!(
            "update {} set state = 'queued', attempts = 0, available_at = now(), finished_at = null,
             cancel_requested = false where id = $1 and state = 'failed'",
            self.table("job")
        )))
        .bind(id.as_uuid())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            self.job(id).await?;
            return Err(Error::conflict("tasks.job_not_failed"));
        }
        Ok(())
    }

    /// Cancels a job: a queued one at once; a running one is asked to stop at its next step and keeps what
    /// it finished.
    ///
    /// # Errors
    ///
    /// `conflict` (`tasks.job_finished`) for a job that already finished.
    pub async fn cancel(&self, id: JobId) -> Result<JobState> {
        let state: Option<String> = sqlx::query_scalar(sql(format!(
            "update {} set cancel_requested = true,
                 state = case when state = 'queued' then 'cancelled' else state end,
                 finished_at = case when state = 'queued' then now() else finished_at end
             where id = $1 and state in ('queued', 'running') returning state",
            self.table("job")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if let Some(state) = state {
            Ok(JobState::read(&state))
        } else {
            self.job(id).await?;
            Err(Error::conflict("tasks.job_finished"))
        }
    }

    /// Reports the progress of a running job, in percent.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn report_progress(&self, id: JobId, percent: u8) -> Result<()> {
        sqlx::query(sql(format!(
            "update {} set progress = $2 where id = $1 and state = 'running'",
            self.table("job")
        )))
        .bind(id.as_uuid())
        .bind(i32::from(percent.min(100)))
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
    }

    /// Whether someone asked to cancel a job; a long job checks it between steps.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown job.
    pub async fn cancelled(&self, id: JobId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select cancel_requested from {} where id = $1",
            self.table("job")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("tasks.job_not_found"))
    }

    /// Removes finished jobs, with their attempts, older than the retention.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn purge(&self) -> Result<u64> {
        sqlx::query(sql(format!(
            "delete from {} where state in ('succeeded', 'failed', 'cancelled')
             and finished_at < now() - make_interval(days => $1)",
            self.table("job")
        )))
        .bind(RETENTION_DAYS)
        .execute(self.db.pool())
        .await
        .map(|r| r.rows_affected())
        .map_err(kampus_db::classify)
    }
}
