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

//! Course progress: a course requires typed criteria, combined with `all` or `any`; the people it follows are
//! its active participants who hold `progress.tracked`; a queue and a job evaluate them when something they
//! depend on changes, record the completion, and revoke it when a criterion stops being met unless the course
//! says completion is final.

use std::collections::{BTreeMap, BTreeSet};

use kampus_access::{Actor, PrincipalId, RoleId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_courses::{Course, CourseId};
use kampus_records::RecordId;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgConnection, Row};

use super::principal;
use crate::permissions::{MANAGE, TRACKED, VIEW_ALL, VIEW_OWN};
use crate::{CourseCriteria, CourseProgress, Criterion, CriterionState, Progress, events, sql};

/// How many people one run evaluates.
const BATCH: i64 = 200;

/// The most criteria a course may have.
const MAX_CRITERIA: usize = 50;

/// A wrong criterion.
fn wrong(field: &str, rule: &str) -> Error {
    Error::validation("progress.criterion_invalid", vec![FieldError::new(field, rule)])
}

/// The kind of a criterion as stored.
const fn kind(criterion: &Criterion) -> &'static str {
    match criterion {
        Criterion::Activity { .. } => "activity",
        Criterion::Grade { .. } => "grade",
        Criterion::Date { .. } => "date",
        Criterion::Duration { .. } => "duration",
        Criterion::RoleApproval { .. } => "role_approval",
        Criterion::Course { .. } => "course",
    }
}

impl Progress {
    /// The course and its scope, for someone who sees it.
    async fn seen_course(&self, actor: &Actor, course: CourseId) -> Result<Course> {
        let found = self.courses.course(course).await?;
        if !self.courses.sees(actor, found.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        Ok(found)
    }

    /// Checks a criterion against the course it is for.
    async fn check_criterion(&self, course: &Course, criterion: &Criterion) -> Result<()> {
        match criterion {
            Criterion::Activity { module, required } => {
                if !crate::db::steps::ACTIVITY_REQUIRED.contains(&required.as_str()) {
                    return Err(wrong("required", "one_of"));
                }
                let of = self
                    .activities
                    .course_of_module(*module)
                    .await
                    .map_err(|_| wrong("module", "exists"))?;
                if of != course.id {
                    return Err(wrong("module", "same_course"));
                }
            }
            Criterion::Grade { record, min_percent } => {
                if *min_percent < Decimal::ZERO || *min_percent > Decimal::ONE_HUNDRED {
                    return Err(wrong("min_percent", "range"));
                }
                let of = self
                    .records
                    .course_of_record(*record)
                    .await
                    .map_err(|_| wrong("record", "exists"))?;
                if of != course.id {
                    return Err(wrong("record", "same_course"));
                }
            }
            Criterion::Date { .. } | Criterion::RoleApproval { .. } => {}
            Criterion::Duration { enrolled_days } => {
                if !(1..=3650).contains(enrolled_days) {
                    return Err(wrong("enrolled_days", "range"));
                }
            }
            Criterion::Course { course: other } => {
                if *other == course.id {
                    return Err(wrong("course", "itself"));
                }
                self.courses
                    .course(*other)
                    .await
                    .map_err(|_| wrong("course", "exists"))?;
            }
        }
        Ok(())
    }

    /// The courses a course requires, through its course criteria and its prerequisites.
    pub(crate) async fn required_courses_in(
        &self,
        conn: &mut PgConnection,
    ) -> Result<BTreeMap<uuid::Uuid, BTreeSet<uuid::Uuid>>> {
        let rows = sqlx::query(sql(format!(
            "select c.course_id, k.required_course_id from {} c join {} k on k.criterion_id = c.id",
            self.table("criterion"),
            self.table("criterion_course")
        )))
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut graph: BTreeMap<uuid::Uuid, BTreeSet<uuid::Uuid>> = BTreeMap::new();
        for row in &rows {
            graph
                .entry(row.get("course_id"))
                .or_default()
                .insert(row.get("required_course_id"));
        }
        Ok(graph)
    }

    /// Whether `target` is required, directly or through others, by `from`.
    pub(crate) fn requires(
        graph: &BTreeMap<uuid::Uuid, BTreeSet<uuid::Uuid>>,
        from: uuid::Uuid,
        target: uuid::Uuid,
    ) -> bool {
        let mut seen = BTreeSet::new();
        let mut pending = vec![from];
        while let Some(id) = pending.pop() {
            if id == target {
                return true;
            }
            if seen.insert(id) {
                pending.extend(graph.get(&id).into_iter().flatten().copied());
            }
        }
        false
    }

    /// Replaces what a course requires; needs `progress.manage` in the course. A criterion that stays the same
    /// keeps its approvals; everyone followed is evaluated again.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`progress.criterion_invalid`) for a criterion that does
    /// not fit the course, (`progress.criteria_cycle`) for a course that would require itself.
    pub async fn define_criteria(
        &self,
        actor: &Actor,
        course: CourseId,
        criteria: CourseCriteria,
    ) -> Result<CourseCriteria> {
        let found = self.seen_course(actor, course).await?;
        self.access.require(actor, MANAGE, found.scope).await?;
        if !crate::db::steps::AGGREGATIONS.contains(&criteria.aggregation.as_str()) {
            return Err(wrong("aggregation", "one_of"));
        }
        if criteria.criteria.len() > MAX_CRITERIA {
            return Err(wrong("criteria", "max"));
        }
        for criterion in &criteria.criteria {
            self.check_criterion(&found, criterion).await?;
        }
        let mut tx = self.db.begin().await?;
        let mut graph = self.required_courses_in(&mut tx).await?;
        graph.remove(&course.as_uuid());
        for criterion in &criteria.criteria {
            if let Criterion::Course { course: other } = criterion {
                if Self::requires(&graph, other.as_uuid(), course.as_uuid()) {
                    return Err(Error::validation(
                        "progress.criteria_cycle",
                        vec![FieldError::new("course", "cycle")],
                    ));
                }
                graph.entry(course.as_uuid()).or_default().insert(other.as_uuid());
            }
        }
        sqlx::query(sql(format!(
            "insert into {} (course_id, aggregation, \"final\") values ($1, $2, $3)
             on conflict (course_id) do update set aggregation = excluded.aggregation, \"final\" = excluded.\"final\",
                 updated_at = now()",
            self.table("course_setting")
        )))
        .bind(course.as_uuid())
        .bind(&criteria.aggregation)
        .bind(criteria.is_final)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let existing = self.criteria_in(&mut tx, course).await?;
        let mut kept = BTreeSet::new();
        for (position, criterion) in criteria.criteria.iter().enumerate() {
            let position = i32::try_from(position).unwrap_or(i32::MAX);
            let reused = existing
                .iter()
                .find(|(id, c)| c == criterion && !kept.contains(id))
                .map(|(id, _)| *id);
            let id = match reused {
                Some(id) => {
                    sqlx::query(sql(format!(
                        "update {} set position = $2 where id = $1",
                        self.table("criterion")
                    )))
                    .bind(id)
                    .bind(position)
                    .execute(&mut *tx)
                    .await
                    .map_err(kampus_db::classify)?;
                    id
                }
                None => {
                    self.insert_criterion_in(&mut tx, course, position, criterion)
                        .await?
                }
            };
            kept.insert(id);
        }
        let gone: Vec<uuid::Uuid> = existing
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| !kept.contains(id))
            .collect();
        sqlx::query(sql(format!(
            "delete from {} where id = any($1)",
            self.table("criterion")
        )))
        .bind(&gone)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_course_in(&mut tx, events::CRITERIA_CHANGED, &found, None, principal(actor))
            .await?;
        let people = self.tracked(&found).await?;
        self.queue_course_in(&mut tx, course, &people).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.course_criteria(actor, course).await
    }

    /// Writes a new criterion and its leaf.
    async fn insert_criterion_in(
        &self,
        conn: &mut PgConnection,
        course: CourseId,
        position: i32,
        criterion: &Criterion,
    ) -> Result<uuid::Uuid> {
        let id = uuid::Uuid::now_v7();
        sqlx::query(sql(format!(
            "insert into {} (id, course_id, kind, position) values ($1, $2, $3, $4)",
            self.table("criterion")
        )))
        .bind(id)
        .bind(course.as_uuid())
        .bind(kind(criterion))
        .bind(position)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let (table, columns, values): (&str, &str, Vec<serde_json::Value>) = match criterion {
            Criterion::Activity { module, required } => (
                "criterion_activity",
                "course_module_id, required",
                vec![json!(module), json!(required)],
            ),
            Criterion::Grade { record, min_percent } => (
                "criterion_grade",
                "record_id, min_percent",
                vec![json!(record), json!(min_percent)],
            ),
            Criterion::Date { on_or_after } => ("criterion_date", "on_or_after", vec![json!(on_or_after)]),
            Criterion::Duration { enrolled_days } => {
                ("criterion_duration", "enrolled_days", vec![json!(enrolled_days)])
            }
            Criterion::RoleApproval { role } => ("criterion_role_approval", "role_id", vec![json!(role)]),
            Criterion::Course { course } => ("criterion_course", "required_course_id", vec![json!(course)]),
        };
        let casts: Vec<String> = values
            .iter()
            .enumerate()
            .map(|(i, _)| format!("(($1::jsonb) ->> {i})::{}", cast(table, i)))
            .collect();
        sqlx::query(sql(format!(
            "insert into {} (criterion_id, {columns}) select $2, {}",
            self.table(table),
            casts.join(", ")
        )))
        .bind(serde_json::Value::Array(values))
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.foreign_key_violation" {
                wrong("target", "exists")
            } else {
                e
            }
        })?;
        Ok(id)
    }

    /// The criteria of a course with their ids, in order.
    async fn criteria_in(
        &self,
        conn: &mut PgConnection,
        course: CourseId,
    ) -> Result<Vec<(uuid::Uuid, Criterion)>> {
        let rows = sqlx::query(sql(format!(
            "select c.id, c.kind, a.course_module_id, a.required, g.record_id, g.min_percent, d.on_or_after,
                 u.enrolled_days, r.role_id, k.required_course_id
             from {c} c
             left join {a} a on a.criterion_id = c.id left join {g} g on g.criterion_id = c.id
             left join {d} d on d.criterion_id = c.id left join {u} u on u.criterion_id = c.id
             left join {r} r on r.criterion_id = c.id left join {k} k on k.criterion_id = c.id
             where c.course_id = $1 order by c.position, c.id",
            c = self.table("criterion"),
            a = self.table("criterion_activity"),
            g = self.table("criterion_grade"),
            d = self.table("criterion_date"),
            u = self.table("criterion_duration"),
            r = self.table("criterion_role_approval"),
            k = self.table("criterion_course")
        )))
        .bind(course.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let id: uuid::Uuid = row.get("id");
                let found = match row.get::<String, _>("kind").as_str() {
                    "activity" => Criterion::Activity {
                        module: kampus_activities::ModuleId::from_uuid(
                            row.get::<Option<uuid::Uuid>, _>("course_module_id")?,
                        ),
                        required: row.get::<Option<String>, _>("required")?,
                    },
                    "grade" => Criterion::Grade {
                        record: RecordId::from_uuid(row.get::<Option<uuid::Uuid>, _>("record_id")?),
                        min_percent: row.get::<Option<Decimal>, _>("min_percent")?.normalize(),
                    },
                    "date" => Criterion::Date {
                        on_or_after: Timestamp::from_offset_date_time(
                            row.get::<Option<time::OffsetDateTime>, _>("on_or_after")?,
                        ),
                    },
                    "duration" => Criterion::Duration {
                        enrolled_days: row.get::<Option<i32>, _>("enrolled_days")?,
                    },
                    "role_approval" => Criterion::RoleApproval {
                        role: RoleId::from_uuid(row.get::<Option<uuid::Uuid>, _>("role_id")?),
                    },
                    "course" => Criterion::Course {
                        course: CourseId::from_uuid(row.get::<Option<uuid::Uuid>, _>("required_course_id")?),
                    },
                    _ => return None,
                };
                Some((id, found))
            })
            .collect())
    }

    /// How the criteria of a course combine and whether completion is final.
    async fn setting(&self, course: CourseId) -> Result<(String, bool)> {
        let row = sqlx::query(sql(format!(
            "select aggregation, \"final\" from {} where course_id = $1",
            self.table("course_setting")
        )))
        .bind(course.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(row.map_or_else(
            || ("all".to_owned(), false),
            |r| (r.get("aggregation"), r.get("final")),
        ))
    }

    /// What a course requires, for whoever sees it.
    ///
    /// # Errors
    ///
    /// `not_found` for a course the actor does not see.
    pub async fn course_criteria(&self, actor: &Actor, course: CourseId) -> Result<CourseCriteria> {
        self.seen_course(actor, course).await?;
        let (aggregation, is_final) = self.setting(course).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let criteria = self.criteria_in(&mut conn, course).await?;
        Ok(CourseCriteria {
            aggregation,
            is_final,
            criteria: criteria.into_iter().map(|(_, c)| c).collect(),
        })
    }

    /// The people progress follows in a course: its active participants who hold `progress.tracked` there.
    ///
    /// # Errors
    ///
    /// `unavailable` when the participants cannot be read.
    pub async fn tracked(&self, course: &Course) -> Result<Vec<PrincipalId>> {
        let mut followed = Vec::new();
        for account in self.courses.participants_of(course.id).await? {
            if self
                .access
                .check(&Actor::Principal(account), TRACKED, course.scope)
                .await?
                .allowed
            {
                followed.push(account);
            }
        }
        Ok(followed)
    }

    /// Whether a person meets one criterion now.
    async fn meets(
        &self,
        course: &Course,
        id: uuid::Uuid,
        criterion: &Criterion,
        account: PrincipalId,
    ) -> Result<bool> {
        let now = Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc());
        Ok(match criterion {
            Criterion::Activity { module, required } => {
                let state: Option<String> = sqlx::query_scalar(sql(format!(
                    "select state from {} where course_module_id = $1 and account_id = $2",
                    self.table("activity_state")
                )))
                .bind(module.as_uuid())
                .bind(account.as_uuid())
                .fetch_optional(self.db.pool())
                .await
                .map_err(kampus_db::classify)?;
                let state = state.unwrap_or_default();
                match required.as_str() {
                    "passed" => state == "done_passed",
                    _ => matches!(state.as_str(), "done" | "done_passed" | "done_failed"),
                }
            }
            Criterion::Grade { record, min_percent } => self
                .records
                .grade_share(*record, account)
                .await?
                .is_some_and(|share| share >= *min_percent),
            Criterion::Date { on_or_after } => now >= *on_or_after,
            Criterion::Duration { enrolled_days } => self
                .courses
                .taking_part_since(course.id, account)
                .await?
                .is_some_and(|since| {
                    since.as_offset_date_time() + time::Duration::days(i64::from(*enrolled_days))
                        <= now.as_offset_date_time()
                }),
            Criterion::RoleApproval { .. } => sqlx::query_scalar(sql(format!(
                "select exists (select 1 from {} where criterion_id = $1 and account_id = $2)",
                self.table("approval")
            )))
            .bind(id)
            .bind(account.as_uuid())
            .fetch_one(self.db.pool())
            .await
            .map_err(kampus_db::classify)?,
            Criterion::Course { course: other } => sqlx::query_scalar(sql(format!(
                "select exists (select 1 from {} where course_id = $1 and account_id = $2 and state = 'completed')",
                self.table("course_state")
            )))
            .bind(other.as_uuid())
            .bind(account.as_uuid())
            .fetch_one(self.db.pool())
            .await
            .map_err(kampus_db::classify)?,
        })
    }

    /// Each criterion of a course with whether a person meets it, and whether the course is met.
    async fn assess(&self, course: &Course, account: PrincipalId) -> Result<(Vec<CriterionState>, bool)> {
        let (aggregation, _) = self.setting(course.id).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let criteria = self.criteria_in(&mut conn, course.id).await?;
        drop(conn);
        let mut states = Vec::new();
        for (id, criterion) in criteria {
            let met = self.meets(course, id, &criterion, account).await?;
            states.push(CriterionState { id, criterion, met });
        }
        let met = if aggregation == "any" {
            states.iter().any(|s| s.met)
        } else {
            !states.is_empty() && states.iter().all(|s| s.met)
        };
        Ok((states, met))
    }

    /// Queues people of a course to be evaluated, and the job that does it when something new was queued.
    pub(crate) async fn queue_course_in(
        &self,
        conn: &mut PgConnection,
        course: CourseId,
        accounts: &[PrincipalId],
    ) -> Result<()> {
        let ids: Vec<uuid::Uuid> = accounts.iter().map(PrincipalId::as_uuid).collect();
        let queued = sqlx::query(sql(format!(
            "insert into {} (course_id, account_id) select $1, unnest($2::uuid[]) on conflict do nothing",
            self.table("course_pending")
        )))
        .bind(course.as_uuid())
        .bind(&ids)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if queued > 0 {
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(conn, "progress.course_recalculate", &json!({}), None)
                .await?;
        }
        Ok(())
    }

    /// Queues one person of a course, for the listeners.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn queue_course(&self, course: CourseId, account: PrincipalId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        self.queue_course_in(&mut tx, course, &[account]).await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Queues the people of the courses that require a course, after their progress in it changed.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn queue_requiring(&self, course: CourseId, account: PrincipalId) -> Result<()> {
        let requiring: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct c.course_id from {} c join {} k on k.criterion_id = c.id where k.required_course_id = $1",
            self.table("criterion"),
            self.table("criterion_course")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut tx = self.db.begin().await?;
        for other in requiring {
            self.queue_course_in(&mut tx, CourseId::from_uuid(other), &[account])
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Queues everyone followed in the courses whose criteria depend on time, for the daily job.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn queue_timed(&self) -> Result<u64> {
        let courses: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct course_id from {} where kind in ('date', 'duration')",
            self.table("criterion")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut queued = 0;
        for course in courses {
            let Ok(found) = self.courses.course(CourseId::from_uuid(course)).await else {
                continue;
            };
            let people = self.tracked(&found).await?;
            queued += u64::try_from(people.len()).unwrap_or(0);
            let mut tx = self.db.begin().await?;
            self.queue_course_in(&mut tx, found.id, &people).await?;
            tx.commit().await.map_err(kampus_db::classify)?;
        }
        Ok(queued)
    }

    /// Evaluates the people queued, at most a batch per run, and returns how many it evaluated. Someone the
    /// course does not follow keeps what they had.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; the queue keeps what was not done.
    pub async fn recalculate_courses(&self) -> Result<u64> {
        let mut tx = self.db.begin().await?;
        let rows = sqlx::query(sql(format!(
            "delete from {t} q using (select course_id, account_id from {t} order by queued_at
                 limit $1 for update skip locked) s
             where q.course_id = s.course_id and q.account_id = s.account_id returning q.course_id, q.account_id",
            t = self.table("course_pending")
        )))
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for row in &rows {
            let Ok(course) = self
                .courses
                .course(CourseId::from_uuid(row.get("course_id")))
                .await
            else {
                continue;
            };
            let account = PrincipalId::from_uuid(row.get("account_id"));
            if !self.tracked_one(&course, account).await? {
                continue;
            }
            let (_, met) = self.assess(&course, account).await?;
            self.apply_in(&mut tx, &course, account, met).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(u64::try_from(rows.len()).unwrap_or(u64::MAX))
    }

    /// Whether a course follows one person.
    async fn tracked_one(&self, course: &Course, account: PrincipalId) -> Result<bool> {
        Ok(self.courses.takes_part(course.id, account).await?
            && self
                .access
                .check(&Actor::Principal(account), TRACKED, course.scope)
                .await?
                .allowed)
    }

    /// Records whether a person completed a course: a completion when they meet it, a revocation when they no
    /// longer do and completion is not final.
    async fn apply_in(
        &self,
        conn: &mut PgConnection,
        course: &Course,
        account: PrincipalId,
        met: bool,
    ) -> Result<()> {
        let (_, is_final) = self.setting(course.id).await?;
        let current: Option<String> = sqlx::query_scalar(sql(format!(
            "select state from {} where course_id = $1 and account_id = $2 for update",
            self.table("course_state")
        )))
        .bind(course.id.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let completed = current.as_deref() == Some("completed");
        let change = match (met, completed) {
            (true, false) => Some(("completed", events::COURSE_COMPLETED)),
            (false, true) if !is_final => Some(("in_progress", events::COURSE_REVOKED)),
            _ => None,
        };
        let Some((state, code)) = change else {
            if current.is_none() {
                sqlx::query(sql(format!(
                    "insert into {} (course_id, account_id, state) values ($1, $2, 'in_progress') on conflict do nothing",
                    self.table("course_state")
                )))
                .bind(course.id.as_uuid())
                .bind(account.as_uuid())
                .execute(&mut *conn)
                .await
                .map_err(kampus_db::classify)?;
            }
            return Ok(());
        };
        sqlx::query(sql(format!(
            "insert into {t} (course_id, account_id, state, completed_at, revoked_at)
             values ($1, $2, $3, case when $3 = 'completed' then now() end, case when $3 = 'in_progress' then now() end)
             on conflict (course_id, account_id) do update set state = excluded.state,
                 completed_at = case when excluded.state = 'completed' then now() else {t}.completed_at end,
                 revoked_at = case when excluded.state = 'in_progress' then now() else {t}.revoked_at end,
                 version = {t}.version + 1",
            t = self.table("course_state")
        )))
        .bind(course.id.as_uuid())
        .bind(account.as_uuid())
        .bind(state)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {} (id, course_id, account_id, event, cause) values ($1, $2, $3, $4, 'criteria')",
            self.table("course_history")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(course.id.as_uuid())
        .bind(account.as_uuid())
        .bind(if state == "completed" {
            "completed"
        } else {
            "revoked"
        })
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_course_in(conn, code, course, Some(account), None)
            .await
    }

    /// Publishes an event about a course, and a person when there is one.
    async fn publish_course_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        course: &Course,
        account: Option<PrincipalId>,
        actor: Option<PrincipalId>,
    ) -> Result<()> {
        let mut payload = json!({ "course": course.id });
        if let Some(account) = account {
            payload["account"] = json!(account);
        }
        let mut event = kampus_events::Publish::new(code, Some(course.id.as_uuid()), payload);
        event.scope = Some(course.scope.as_uuid());
        event.actor = actor.map(|a| a.as_uuid());
        event.summary = Some(format!("{code} {}", course.id));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }

    /// The progress of a person in a course: their own with `progress.view_own`, anyone's with
    /// `progress.view_all`.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn course_progress(
        &self,
        actor: &Actor,
        course: CourseId,
        account: PrincipalId,
    ) -> Result<CourseProgress> {
        let found = self.seen_course(actor, course).await?;
        let own = principal(actor) == Some(account);
        let permission = if own { VIEW_OWN } else { VIEW_ALL };
        if !self.access.check(actor, permission, found.scope).await?.allowed
            && !self.access.check(actor, VIEW_ALL, found.scope).await?.allowed
        {
            return Err(Error::permission_denied("access.denied").with_param("permission", permission));
        }
        let row = sqlx::query(sql(format!(
            "select state, completed_at from {} where course_id = $1 and account_id = $2",
            self.table("course_state")
        )))
        .bind(course.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let (criteria, _) = self.assess(&found, account).await?;
        let tracked = self.tracked_one(&found, account).await?;
        Ok(CourseProgress {
            course,
            account,
            state: match &row {
                Some(r) => r.get("state"),
                None if tracked => "in_progress".to_owned(),
                None => "not_tracked".to_owned(),
            },
            completed_at: row.as_ref().and_then(|r| {
                r.get::<Option<time::OffsetDateTime>, _>("completed_at")
                    .map(Timestamp::from_offset_date_time)
            }),
            criteria,
        })
    }

    /// The progress of everyone a course follows; needs `progress.view_all`.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn course_report(&self, actor: &Actor, course: CourseId) -> Result<Vec<CourseProgress>> {
        let found = self.seen_course(actor, course).await?;
        self.access.require(actor, VIEW_ALL, found.scope).await?;
        let mut report = Vec::new();
        for account in self.tracked(&found).await? {
            report.push(self.course_progress(actor, course, account).await?);
        }
        Ok(report)
    }

    /// Approves a person for a criterion of their course: whoever approves must hold the role the criterion
    /// names in the course. Approving twice changes nothing.
    ///
    /// # Errors
    ///
    /// `not_found` (`progress.criterion_not_found`) for an unknown criterion or one of another kind,
    /// `permission_denied` (`progress.role_required`) for someone without the role.
    pub async fn approve(&self, actor: &Actor, criterion: uuid::Uuid, account: PrincipalId) -> Result<()> {
        let by = principal(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        let row = sqlx::query(sql(format!(
            "select c.course_id, r.role_id from {} c join {} r on r.criterion_id = c.id where c.id = $1",
            self.table("criterion"),
            self.table("criterion_role_approval")
        )))
        .bind(criterion)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("progress.criterion_not_found"))?;
        let course = self
            .seen_course(actor, CourseId::from_uuid(row.get("course_id")))
            .await?;
        let role = RoleId::from_uuid(row.get("role_id"));
        if !self.access.holds_role(by, role, course.scope).await? {
            return Err(
                Error::permission_denied("progress.role_required").with_param("role", role.to_string())
            );
        }
        let mut tx = self.db.begin().await?;
        let inserted = sqlx::query(sql(format!(
            "insert into {} (criterion_id, account_id, approved_by) values ($1, $2, $3) on conflict do nothing",
            self.table("approval")
        )))
        .bind(criterion)
        .bind(account.as_uuid())
        .bind(by.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if inserted > 0 {
            let mut event = kampus_events::Publish::new(
                events::APPROVED,
                Some(criterion),
                json!({ "course": course.id, "criterion": criterion, "account": account }),
            );
            event.scope = Some(course.scope.as_uuid());
            event.actor = Some(by.as_uuid());
            event.summary = Some(format!("{} {criterion}", events::APPROVED));
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, event)
                .await?;
            self.queue_course_in(&mut tx, course.id, &[account]).await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }
}

/// The type a leaf column is cast to from the parameters.
fn cast(table: &str, index: usize) -> &'static str {
    match (table, index) {
        ("criterion_activity", 1) => "text",
        ("criterion_grade", 1) => "numeric",
        ("criterion_date", _) => "timestamptz",
        ("criterion_duration", _) => "integer",
        _ => "uuid",
    }
}
