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

//! The queue: jobs are enqueued in the transaction that needs them, claimed with `FOR UPDATE SKIP
//! LOCKED` under a lease as long as their timeout, and retried with back-off until they run out of
//! attempts and fail with their error. "Now" is always the database clock.

use kampus_common::{Error, FieldError, Result, Timestamp};
use serde_json::Value;
use sqlx::{PgConnection, Row};

use crate::{JobId, Tasks, sql};

/// A job a worker claimed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Claimed {
    /// The job.
    pub(crate) id: JobId,
    /// Its type.
    pub(crate) type_code: String,
    /// Its payload.
    pub(crate) payload: Value,
    /// This attempt's number, from 1.
    pub(crate) attempt: i32,
    /// How many attempts it may make.
    pub(crate) max_attempts: i32,
    /// The wait before the first retry, in seconds.
    pub(crate) backoff_seconds: i32,
    /// How long one run may take, in seconds.
    pub(crate) timeout_seconds: i32,
    /// The attempt row.
    pub(crate) attempt_id: uuid::Uuid,
}

/// How an attempt ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ended {
    /// It did its work.
    Succeeded,
    /// It failed with this error.
    Failed(String),
    /// It ran past its timeout.
    TimedOut,
}

impl Tasks {
    /// Enqueues a job of a declared type inside the caller's transaction, so it exists only if the change
    /// that needs it commits; it may run from `available_at` (now when absent).
    ///
    /// # Errors
    ///
    /// `validation` (`tasks.job_type_unknown`) for a type no module declares.
    pub async fn enqueue_in(
        &self,
        conn: &mut PgConnection,
        type_code: &str,
        payload: &Value,
        available_at: Option<Timestamp>,
    ) -> Result<JobId> {
        let id = JobId::generate();
        let inserted = sqlx::query(sql(format!(
            "insert into {jobs} (id, type_code, payload, available_at, concurrency_key)
             select $1, t.code, $3, coalesce($4, now()),
                    case when t.concurrency is null then null else t.code || ':' || coalesce($3 ->> t.concurrency, '') end
             from {types} t where t.code = $2 and t.retired_at is null",
            jobs = self.table("job"),
            types = self.table("job_type")
        )))
        .bind(id.as_uuid())
        .bind(type_code)
        .bind(payload)
        .bind(available_at.map(|a| a.as_offset_date_time()))
        .execute(conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if inserted == 0 {
            return Err(Error::validation(
                "tasks.job_type_unknown",
                vec![FieldError::new("type", "declared")],
            )
            .with_param("type", type_code.to_owned()));
        }
        Ok(id)
    }

    /// Enqueues a job on its own.
    ///
    /// # Errors
    ///
    /// As [`Tasks::enqueue_in`].
    pub async fn enqueue(
        &self,
        type_code: &str,
        payload: &Value,
        available_at: Option<Timestamp>,
    ) -> Result<JobId> {
        let mut tx = self.db.begin().await?;
        let id = self.enqueue_in(&mut tx, type_code, payload, available_at).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Puts back in the queue the jobs whose lease expired, because their worker stopped.
    pub(crate) async fn recover_lost(&self) -> Result<u64> {
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "update {} set ended_at = now(), outcome = 'lost' where ended_at is null and job_id in
             (select id from {} where state = 'running' and lease_until < now())",
            self.table("attempt"),
            self.table("job")
        )))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let recovered = sqlx::query(sql(format!(
            "update {} set state = 'queued', lease_until = null, worker = null, available_at = now()
             where state = 'running' and lease_until < now()",
            self.table("job")
        )))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(recovered)
    }

    /// Claims the next job of one of these types that may run now and whose concurrency key is free.
    pub(crate) async fn claim(&self, worker: &str, types: &[String]) -> Result<Option<Claimed>> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "update {jobs} j set state = 'running', attempts = j.attempts + 1, worker = $1,
                 lease_until = now() + make_interval(secs => t.timeout_seconds)
             from {types} t
             where t.code = j.type_code and j.id = (
                 select q.id from {jobs} q where q.state = 'queued' and q.available_at <= now()
                 and q.type_code = any($2)
                 and (q.concurrency_key is null or not exists (
                     select 1 from {jobs} r where r.state = 'running' and r.concurrency_key = q.concurrency_key))
                 order by q.available_at, q.id limit 1 for update skip locked)
             returning j.id, j.type_code, j.payload, j.attempts, t.max_attempts, t.backoff_seconds, t.timeout_seconds",
            jobs = self.table("job"),
            types = self.table("job_type")
        )))
        .bind(worker)
        .bind(types)
        .fetch_optional(&mut *tx)
        .await;
        let row = match row {
            Ok(row) => row,
            Err(error) => {
                let error = kampus_db::classify(error);
                if error.code() == "db.unique_violation" {
                    return Ok(None);
                }
                return Err(error);
            }
        };
        let Some(row) = row else {
            return Ok(None);
        };
        let id = JobId::from_uuid(row.get("id"));
        let attempt_id = uuid::Uuid::now_v7();
        sqlx::query(sql(format!(
            "insert into {} (id, job_id, worker) values ($1, $2, $3)",
            self.table("attempt")
        )))
        .bind(attempt_id)
        .bind(id.as_uuid())
        .bind(worker)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(Some(Claimed {
            id,
            type_code: row.get("type_code"),
            payload: row.get("payload"),
            attempt: row.get("attempts"),
            max_attempts: row.get("max_attempts"),
            backoff_seconds: row.get("backoff_seconds"),
            timeout_seconds: row.get("timeout_seconds"),
            attempt_id,
        }))
    }

    /// Records how an attempt ended: a success finishes the job, a cancellation asked for meanwhile
    /// cancels it, and a failure retries it with back-off or fails it for good.
    pub(crate) async fn finish(&self, job: &Claimed, ended: Ended) -> Result<()> {
        let (outcome, error) = match &ended {
            Ended::Succeeded => ("succeeded", None),
            Ended::Failed(error) => ("failed", Some(error.clone())),
            Ended::TimedOut => ("timed_out", Some("the run went past its timeout".to_owned())),
        };
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "update {} set ended_at = now(), outcome = $2, error = $3 where id = $1",
            self.table("attempt")
        )))
        .bind(job.attempt_id)
        .bind(outcome)
        .bind(&error)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let jobs = self.table("job");
        let statement = match ended {
            Ended::Succeeded => format!(
                "update {jobs} set state = case when cancel_requested then 'cancelled' else 'succeeded' end,
                 progress = case when cancel_requested then progress else 100 end,
                 finished_at = now(), lease_until = null where id = $1"
            ),
            Ended::Failed(_) | Ended::TimedOut if job.attempt >= job.max_attempts => format!(
                "update {jobs} set state = 'failed', last_error = $2, finished_at = now(), lease_until = null where id = $1"
            ),
            Ended::Failed(_) | Ended::TimedOut => format!(
                "update {jobs} set state = case when cancel_requested then 'cancelled' else 'queued' end,
                 last_error = $2, lease_until = null, worker = null,
                 finished_at = case when cancel_requested then now() else null end,
                 available_at = now() + make_interval(secs => $3::double precision)
                 where id = $1"
            ),
        };
        let backoff = f64::from(job.backoff_seconds) * 2_f64.powi(job.attempt - 1);
        sqlx::query(sql(statement))
            .bind(job.id.as_uuid())
            .bind(&error)
            .bind(backoff)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        tx.commit().await.map_err(kampus_db::classify)?;
        if job.attempt >= job.max_attempts
            && let Some(error) = error
        {
            tracing::error!(job = %job.id, type_code = job.type_code, error, "job failed after its last attempt");
        }
        Ok(())
    }
}
