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

//! Schedules: modules declare them with a default rule, operators may change the rule, its time zone or
//! disable it, and each firing is a tick unique per schedule and instant, so several schedulers never
//! enqueue the same run twice.

use kampus_common::{Error, Result, Timestamp};
use kampus_module::Module;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::instant;
use crate::support::cron::Rule;
use crate::{Schedule, ScheduleId, Tasks, sql};

/// A kampus instant as a `jiff` one.
fn to_jiff(at: Timestamp) -> Result<jiff::Timestamp> {
    jiff::Timestamp::from_second(at.as_offset_date_time().unix_timestamp())
        .map_err(|_| Error::internal("tasks.instant_invalid"))
}

/// A `jiff` instant as a kampus one.
fn from_jiff(at: jiff::Timestamp) -> Result<Timestamp> {
    time::OffsetDateTime::from_unix_timestamp(at.as_second())
        .map(Timestamp::from_offset_date_time)
        .map_err(|_| Error::internal("tasks.instant_invalid"))
}

/// The next run of a rule after an instant.
fn next_run(cron: &str, time_zone: &str, after: Timestamp) -> Result<Timestamp> {
    from_jiff(Rule::parse(cron, time_zone)?.next_after(to_jiff(after)?)?)
}

/// A schedule from its row.
fn schedule(row: &PgRow) -> Schedule {
    Schedule {
        id: ScheduleId::from_uuid(row.get("id")),
        type_code: row.get("type_code"),
        cron: row.get("cron"),
        time_zone: row.get("time_zone"),
        enabled: row.get("enabled"),
        next_run_at: instant(row, "next_run_at")
            .unwrap_or_else(|| Timestamp::from_offset_date_time(time::OffsetDateTime::UNIX_EPOCH)),
        last_run_at: instant(row, "last_run_at"),
        version: row.get("version"),
    }
}

/// The columns of a schedule.
const COLUMNS: &str = "id, type_code, cron, time_zone, enabled, next_run_at, last_run_at, version";

impl Tasks {
    /// Brings what the modules of the build declare into the database: their job types, retiring those
    /// no module declares, and their schedules, which keep the rule an operator set.
    ///
    /// # Errors
    ///
    /// `validation` for a declared schedule whose rule does not parse or whose job is not declared.
    pub async fn sync_declarations(&self, modules: &[&Module], now: Timestamp) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let mut codes = Vec::new();
        for module in modules {
            for job in module.jobs {
                codes.push(job.code.to_owned());
                sqlx::query(sql(format!(
                    "insert into {} (code, module, summary, max_attempts, backoff_seconds, timeout_seconds, concurrency)
                     values ($1, $2, $3, $4, $5, $6, $7)
                     on conflict (code) do update set module = excluded.module, summary = excluded.summary,
                     max_attempts = excluded.max_attempts, backoff_seconds = excluded.backoff_seconds,
                     timeout_seconds = excluded.timeout_seconds, concurrency = excluded.concurrency, retired_at = null",
                    self.table("job_type")
                )))
                .bind(job.code)
                .bind(module.name)
                .bind(job.summary)
                .bind(i32::from(job.max_attempts))
                .bind(i64::from(job.backoff_seconds))
                .bind(i64::from(job.timeout_seconds))
                .bind(job.concurrency)
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        sqlx::query(sql(format!(
            "update {} set retired_at = now() where not (code = any($1)) and retired_at is null",
            self.table("job_type")
        )))
        .bind(&codes)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for module in modules {
            for declared in module.schedules {
                if !codes.iter().any(|c| c == declared.job) {
                    return Err(Error::validation("tasks.schedule_job_unknown", vec![])
                        .with_param("job", declared.job));
                }
                let next = next_run(declared.cron, declared.time_zone, now)?;
                sqlx::query(sql(format!(
                    "insert into {} (id, type_code, cron, time_zone, next_run_at) values ($1, $2, $3, $4, $5)
                     on conflict (type_code) do nothing",
                    self.table("schedule")
                )))
                .bind(uuid::Uuid::now_v7())
                .bind(declared.job)
                .bind(declared.cron)
                .bind(declared.time_zone)
                .bind(next.as_offset_date_time())
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
            }
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Every schedule, by job type.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn schedules(&self) -> Result<Vec<Schedule>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} order by type_code",
            self.table("schedule")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(schedule).collect())
    }

    /// Changes the rule, time zone or state of a schedule; the change keeps who made it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown job type, `validation` for a rule that does not parse.
    pub async fn change_schedule(
        &self,
        type_code: &str,
        cron: &str,
        time_zone: &str,
        enabled: bool,
        changed_by: &str,
    ) -> Result<Schedule> {
        let now: time::OffsetDateTime = sqlx::query_scalar(sql("select now()".to_owned()))
            .fetch_one(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        let next = next_run(cron, time_zone, Timestamp::from_offset_date_time(now))?;
        sqlx::query(sql(format!(
            "update {} set cron = $2, time_zone = $3, enabled = $4, next_run_at = $5, changed_by = $6,
             version = version + 1 where type_code = $1 returning {COLUMNS}",
            self.table("schedule")
        )))
        .bind(type_code)
        .bind(cron)
        .bind(time_zone)
        .bind(enabled)
        .bind(next.as_offset_date_time())
        .bind(changed_by)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .map(|row| schedule(&row))
        .ok_or_else(|| Error::not_found("tasks.schedule_not_found"))
    }

    /// Enqueues a run of a scheduled job now, outside its rule.
    ///
    /// # Errors
    ///
    /// `validation` for an unknown job type.
    pub async fn run_now(&self, type_code: &str) -> Result<crate::JobId> {
        self.enqueue(type_code, &json!({}), None).await
    }

    /// Fires the schedules that are due: each one records its tick, enqueues its job in the same
    /// transaction, and moves to its next run. A tick that exists already, fired by another scheduler, is
    /// left alone. Returns how many jobs were enqueued.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn tick(&self) -> Result<usize> {
        let mut enqueued = 0;
        loop {
            let mut tx = self.db.begin().await?;
            let due = sqlx::query(sql(format!(
                "select id, type_code, cron, time_zone, next_run_at, now() as now from {}
                 where enabled and next_run_at <= now() order by next_run_at, id limit 1 for update skip locked",
                self.table("schedule")
            )))
            .fetch_optional(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            let Some(row) = due else {
                return Ok(enqueued);
            };
            let id: uuid::Uuid = row.get("id");
            let at: time::OffsetDateTime = row.get("next_run_at");
            let type_code: String = row.get("type_code");
            let fresh = sqlx::query(sql(format!(
                "insert into {} (schedule_id, at) values ($1, $2) on conflict do nothing",
                self.table("schedule_tick")
            )))
            .bind(id)
            .bind(at)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?
            .rows_affected()
                == 1;
            if fresh {
                let job = self.enqueue_in(&mut tx, &type_code, &json!({}), None).await?;
                sqlx::query(sql(format!(
                    "update {} set job_id = $3 where schedule_id = $1 and at = $2",
                    self.table("schedule_tick")
                )))
                .bind(id)
                .bind(at)
                .bind(job.as_uuid())
                .execute(&mut *tx)
                .await
                .map_err(kampus_db::classify)?;
                enqueued += 1;
            }
            let now = Timestamp::from_offset_date_time(row.get("now"));
            let next = next_run(
                &row.get::<String, _>("cron"),
                &row.get::<String, _>("time_zone"),
                now,
            )?;
            sqlx::query(sql(format!(
                "update {} set next_run_at = $2, last_run_at = $3 where id = $1",
                self.table("schedule")
            )))
            .bind(id)
            .bind(next.as_offset_date_time())
            .bind(at)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            tx.commit().await.map_err(kampus_db::classify)?;
        }
    }
}
