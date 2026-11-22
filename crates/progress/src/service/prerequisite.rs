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

//! Prerequisites: a course can require other courses to be completed before someone enrols in it by
//! themselves. The graph stays acyclic, also together with the course criteria, and enrolment asks progress
//! through its gate.

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result};
use kampus_courses::CourseId;
use serde::Serialize;
use sqlx::{PgConnection, Row};

use super::principal;
use crate::permissions::MANAGE;
use crate::{Progress, events, sql};

/// The most courses a course may require first.
const MAX_PREREQUISITES: usize = 20;

/// The prerequisites around a course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Prerequisites {
    /// The course.
    pub course: CourseId,
    /// The courses it requires first.
    pub requires: Vec<CourseId>,
    /// The courses that require it first.
    pub required_by: Vec<CourseId>,
}

impl Progress {
    /// The graph of what each course requires: its course criteria, but those of `without_criteria_of`, and
    /// its prerequisites, but those of `without_prerequisites_of`.
    pub(crate) async fn requirement_graph_in(
        &self,
        conn: &mut PgConnection,
        without_criteria_of: Option<CourseId>,
        without_prerequisites_of: Option<CourseId>,
    ) -> Result<BTreeMap<uuid::Uuid, BTreeSet<uuid::Uuid>>> {
        let mut graph = self.required_courses_in(&mut *conn).await?;
        if let Some(course) = without_criteria_of {
            graph.remove(&course.as_uuid());
        }
        let rows = sqlx::query(sql(format!(
            "select course_id, requires_course_id from {} where course_id is distinct from $1",
            self.table("prerequisite")
        )))
        .bind(without_prerequisites_of.map(|c| c.as_uuid()))
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        for row in &rows {
            graph
                .entry(row.get("course_id"))
                .or_default()
                .insert(row.get("requires_course_id"));
        }
        Ok(graph)
    }

    /// Replaces the courses a course requires first; needs `progress.manage` in the course.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`progress.prerequisite_invalid`) for the course itself or
    /// an unknown one, `conflict` (`progress.prerequisites_cycle`) for a graph that would require a course of
    /// itself.
    pub async fn set_prerequisites(
        &self,
        actor: &Actor,
        course: CourseId,
        requires: Vec<CourseId>,
    ) -> Result<Prerequisites> {
        let found = self.courses.course(course).await?;
        if !self.courses.sees(actor, found.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        self.access.require(actor, MANAGE, found.scope).await?;
        let wanted: BTreeSet<CourseId> = requires.into_iter().collect();
        if wanted.len() > MAX_PREREQUISITES {
            return Err(Error::validation(
                "progress.prerequisite_invalid",
                vec![FieldError::new("requires", "max")],
            ));
        }
        for other in &wanted {
            if *other == course || self.courses.course(*other).await.is_err() {
                return Err(Error::validation(
                    "progress.prerequisite_invalid",
                    vec![FieldError::new("requires", "course")],
                ));
            }
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where course_id = $1",
            self.table("prerequisite")
        )))
        .bind(course.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut graph = self.requirement_graph_in(&mut tx, None, Some(course)).await?;
        for other in &wanted {
            if Self::requires(&graph, other.as_uuid(), course.as_uuid()) {
                return Err(
                    Error::conflict("progress.prerequisites_cycle").with_param("course", other.to_string())
                );
            }
            graph.entry(course.as_uuid()).or_default().insert(other.as_uuid());
        }
        let ids: Vec<uuid::Uuid> = wanted.iter().map(CourseId::as_uuid).collect();
        sqlx::query(sql(format!(
            "insert into {} (course_id, requires_course_id) select $1, unnest($2::uuid[])",
            self.table("prerequisite")
        )))
        .bind(course.as_uuid())
        .bind(&ids)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let mut event = kampus_events::Publish::new(
            events::PREREQUISITES_CHANGED,
            Some(course.as_uuid()),
            serde_json::json!({ "course": course }),
        );
        event.scope = Some(found.scope.as_uuid());
        event.actor = principal(actor).map(|p| p.as_uuid());
        event.summary = Some(format!("{} {course}", events::PREREQUISITES_CHANGED));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.prerequisites(actor, course).await
    }

    /// The courses a course requires first and those that require it, for whoever sees it.
    ///
    /// # Errors
    ///
    /// `not_found` for a course the actor does not see.
    pub async fn prerequisites(&self, actor: &Actor, course: CourseId) -> Result<Prerequisites> {
        let found = self.courses.course(course).await?;
        if !self.courses.sees(actor, found.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        let rows = sqlx::query(sql(format!(
            "select course_id, requires_course_id from {} where course_id = $1 or requires_course_id = $1
             order by course_id, requires_course_id",
            self.table("prerequisite")
        )))
        .bind(course.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut requires = Vec::new();
        let mut required_by = Vec::new();
        for row in &rows {
            let (from, to): (uuid::Uuid, uuid::Uuid) = (row.get("course_id"), row.get("requires_course_id"));
            if from == course.as_uuid() {
                requires.push(CourseId::from_uuid(to));
            } else {
                required_by.push(CourseId::from_uuid(from));
            }
        }
        Ok(Prerequisites {
            course,
            requires,
            required_by,
        })
    }

    /// The prerequisites of a course a person has not completed.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn missing_prerequisites(
        &self,
        course: CourseId,
        account: PrincipalId,
    ) -> Result<Vec<CourseId>> {
        let missing: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select p.requires_course_id from {} p where p.course_id = $1 and not exists (
                 select 1 from {} s where s.course_id = p.requires_course_id and s.account_id = $2
                     and s.state = 'completed')
             order by p.requires_course_id",
            self.table("prerequisite"),
            self.table("course_state")
        )))
        .bind(course.as_uuid())
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(missing.into_iter().map(CourseId::from_uuid).collect())
    }
}

/// The gate enrolment asks before someone enrols themselves: the prerequisites they have not completed.
#[derive(Debug, Clone)]
pub struct PrerequisiteGate {
    /// Progress, which knows the prerequisites and the completions.
    progress: Progress,
}

impl PrerequisiteGate {
    /// The gate of an installation.
    #[must_use]
    pub const fn new(progress: Progress) -> Self {
        Self { progress }
    }
}

#[async_trait]
impl kampus_enrol::ports::gate::JoiningGate for PrerequisiteGate {
    async fn missing(&self, course: CourseId, account: PrincipalId) -> Result<Vec<CourseId>> {
        self.progress.missing_prerequisites(course, account).await
    }
}
