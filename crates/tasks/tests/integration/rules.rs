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

//! One test per behaviour of the task queue, against PostgreSQL: enqueueing in a transaction, retries
//! and failure, timeouts, concurrency keys, lost leases, cancelling, and schedules that fire once.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use kampus_common::{Clock, Error, SystemClock};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_module::{JobDeclaration, JobFuture, JobRun, Module, ScheduleDeclaration, ServiceContext};
use kampus_tasks::{JobState, Tasks, Worker};
use serde_json::json;

/// How many runs of the keyed job are going on now, and the most seen at once.
static RUNNING: AtomicU32 = AtomicU32::new(0);
/// The most runs of the keyed job seen at once.
static MOST: AtomicU32 = AtomicU32::new(0);

/// A job that succeeds.
fn succeed(_run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async { Ok(()) })
}

/// A job that always fails.
fn fail(_run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async { Err(Error::unavailable("testing.broken")) })
}

/// A job that runs past its timeout.
fn slow(_run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(())
    })
}

/// A job that counts how many of its kind run at once.
fn keyed(_run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async {
        let now = RUNNING.fetch_add(1, Ordering::SeqCst) + 1;
        MOST.fetch_max(now, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(300)).await;
        RUNNING.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    })
}

/// A job declaration of these tests.
const fn job(
    code: &'static str,
    max_attempts: u16,
    timeout_seconds: u32,
    run: for<'j> fn(JobRun<'j>) -> JobFuture<'j>,
) -> JobDeclaration {
    JobDeclaration {
        code,
        summary: "a job of the tests",
        max_attempts,
        backoff_seconds: 0,
        timeout_seconds,
        concurrency: None,
        run,
    }
}

/// The jobs of these tests.
const JOBS: &[JobDeclaration] = &[
    job("testing.succeed", 3, 30, succeed),
    job("testing.fail", 2, 30, fail),
    job("testing.slow", 2, 1, slow),
    JobDeclaration {
        concurrency: Some("course"),
        ..job("testing.keyed", 3, 30, keyed)
    },
];

/// The schedules of these tests.
const SCHEDULES: &[ScheduleDeclaration] = &[ScheduleDeclaration {
    job: "testing.succeed",
    cron: "* * * * *",
    time_zone: "UTC",
}];

/// A module that declares the jobs of these tests.
const TESTING: Module = Module {
    name: "testing",
    jobs: JOBS,
    schedules: SCHEDULES,
    ..kampus_tasks::MODULE
};

/// An installation with the queue and the jobs of these tests.
struct World {
    /// The queue.
    tasks: Tasks,
    /// A worker of every job of these tests.
    worker: Worker,
    /// The database.
    db: Database,
}

/// Builds the world in an installation of its own prefix.
async fn world() -> World {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("t{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 6).await.unwrap();
    Upgrader::new(&db, &[kampus_tasks::MODULE.schema()], "0.1.0")
        .apply(true)
        .await
        .unwrap();
    let tasks = Tasks::new(db.clone());
    tasks
        .sync_declarations(&[&kampus_tasks::MODULE, &TESTING], SystemClock.now())
        .await
        .unwrap();
    let worker = Worker::new(
        tasks.clone(),
        JOBS.iter().copied(),
        ServiceContext::default(),
        "test".to_owned(),
    );
    World { tasks, worker, db }
}

impl World {
    /// The state and attempts of a job.
    async fn state(&self, id: kampus_tasks::JobId) -> (JobState, i32) {
        let (job, _) = self.tasks.job(id).await.unwrap();
        (job.state, job.attempts)
    }

    /// Runs jobs until none is left that may run now.
    async fn drain(&self) {
        while self.worker.run_once().await.unwrap() {}
    }

    /// Drops the installation.
    async fn drop(self) {
        for module in ["tasks", "db"] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(module)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
        self.db.close().await;
    }
}

#[tokio::test]
async fn a_job_exists_only_if_the_transaction_that_enqueued_it_commits() {
    let w = world().await;
    let mut tx = w.db.begin().await.unwrap();
    w.tasks
        .enqueue_in(&mut tx, "testing.succeed", &json!({}), None)
        .await
        .unwrap();
    drop(tx);
    assert!(
        w.tasks.jobs(None, None, 10).await.unwrap().is_empty(),
        "rolled back, nothing queued"
    );
    assert_eq!(
        w.tasks
            .enqueue("testing.nothing", &json!({}), None)
            .await
            .unwrap_err()
            .code(),
        "tasks.job_type_unknown"
    );
    let id = w
        .tasks
        .enqueue("testing.succeed", &json!({}), None)
        .await
        .unwrap();
    w.drain().await;
    assert_eq!(w.state(id).await, (JobState::Succeeded, 1));
    let (job, attempts) = w.tasks.job(id).await.unwrap();
    assert_eq!(job.progress, 100);
    assert_eq!(attempts[0].outcome.as_deref(), Some("succeeded"));
    w.drop().await;
}

#[tokio::test]
async fn a_failing_job_is_retried_then_fails_with_its_error_and_can_be_retried() {
    let w = world().await;
    let id = w.tasks.enqueue("testing.fail", &json!({}), None).await.unwrap();
    w.drain().await;
    assert_eq!(
        w.state(id).await,
        (JobState::Failed, 2),
        "two attempts, then failed"
    );
    let (job, attempts) = w.tasks.job(id).await.unwrap();
    assert!(job.last_error.unwrap().contains("testing.broken"));
    assert_eq!(attempts.len(), 2);
    w.tasks.retry(id).await.unwrap();
    assert_eq!(w.state(id).await, (JobState::Queued, 0));
    assert_eq!(
        w.tasks.retry(id).await.unwrap_err().code(),
        "tasks.job_not_failed"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_job_past_its_timeout_is_stopped_and_retried() {
    let w = world().await;
    let id = w.tasks.enqueue("testing.slow", &json!({}), None).await.unwrap();
    w.drain().await;
    let (job, attempts) = w.tasks.job(id).await.unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert!(attempts.iter().all(|a| a.outcome.as_deref() == Some("timed_out")));
    w.drop().await;
}

#[tokio::test]
async fn jobs_sharing_a_concurrency_key_run_one_at_a_time() {
    let w = world().await;
    for _ in 0..3 {
        w.tasks
            .enqueue("testing.keyed", &json!({ "course": "physics" }), None)
            .await
            .unwrap();
    }
    let mut runs = Vec::new();
    for _ in 0..3 {
        let worker = w.worker.clone();
        runs.push(tokio::spawn(async move { worker.run_once().await.unwrap() }));
    }
    for run in runs {
        run.await.unwrap();
    }
    w.drain().await;
    assert_eq!(MOST.load(Ordering::SeqCst), 1, "never two at once");
    let done = w
        .tasks
        .jobs(Some("testing.keyed"), Some(JobState::Succeeded), 10)
        .await
        .unwrap();
    assert_eq!(done.len(), 3);
    w.drop().await;
}

#[tokio::test]
async fn the_job_of_a_stopped_worker_is_taken_again_when_its_lease_expires() {
    let w = world().await;
    let id = w
        .tasks
        .enqueue("testing.succeed", &json!({}), None)
        .await
        .unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {}.\"job\" set state = 'running', attempts = 1, lease_until = now() - interval '1 minute' where id = '{id}'",
        w.db.prefix().schema("tasks")
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    w.drain().await;
    assert_eq!(w.state(id).await, (JobState::Succeeded, 2));
    w.drop().await;
}

#[tokio::test]
async fn a_cancelled_job_does_not_run() {
    let w = world().await;
    let id = w
        .tasks
        .enqueue("testing.succeed", &json!({}), None)
        .await
        .unwrap();
    assert_eq!(w.tasks.cancel(id).await.unwrap(), JobState::Cancelled);
    w.drain().await;
    assert_eq!(w.state(id).await, (JobState::Cancelled, 0));
    assert_eq!(w.tasks.cancel(id).await.unwrap_err().code(), "tasks.job_finished");
    w.drop().await;
}

#[tokio::test]
async fn a_schedule_fires_once_per_instant_even_with_several_schedulers() {
    let w = world().await;
    let schedules = w.tasks.schedules().await.unwrap();
    assert_eq!(schedules.len(), 2, "the declared schedules, the purge among them");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "update {}.\"schedule\" set next_run_at = now() - interval '1 minute' where type_code = 'testing.succeed'",
        w.db.prefix().schema("tasks")
    )))
    .execute(w.db.pool())
    .await
    .unwrap();
    let (a, b) = tokio::join!(w.tasks.tick(), w.tasks.tick());
    assert_eq!(
        a.unwrap() + b.unwrap(),
        1,
        "one run, whatever the number of schedulers"
    );
    assert_eq!(
        w.tasks.tick().await.unwrap(),
        0,
        "and not again until its next instant"
    );
    let after = w.tasks.schedules().await.unwrap();
    let fired = after.iter().find(|s| s.type_code == "testing.succeed").unwrap();
    assert!(fired.last_run_at.is_some());

    let changed = w
        .tasks
        .change_schedule("testing.succeed", "0 4 * * *", "Europe/Madrid", false, "operator")
        .await
        .unwrap();
    assert!(!changed.enabled);
    assert_eq!(
        w.tasks
            .change_schedule("testing.succeed", "0 25 * * *", "UTC", true, "operator")
            .await
            .unwrap_err()
            .code(),
        "tasks.cron_invalid"
    );
    w.drop().await;
}
