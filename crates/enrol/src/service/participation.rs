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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Active participation: a person takes part in a course when one of their enrolments in it is active,
//! its instance is enabled and the clock of the service is within its period: the same clock that gave
//! the enrolment its start, so one that starts now takes part at once. Every module asks the same
//! question here, and courses through its participants port.

use std::sync::Arc;

use async_trait::async_trait;
use kampus_access::PrincipalId;
use kampus_common::{Clock, Result, Timestamp};
use kampus_courses::{CourseId, Participants};
use kampus_db::Database;

use crate::{Enrol, sql, table};

/// The condition of an active enrolment `e` of an instance `i` at the instant bound to `now` (a
/// parameter such as `$3`), in SQL.
pub(crate) fn active(now: &str) -> String {
    format!(
        "e.status = 'active' and i.enabled and e.starts_at <= {now}::timestamptz \
         and (e.ends_at is null or {now}::timestamptz < e.ends_at)"
    )
}

/// Answers who takes part in a course for the modules that cannot depend on enrolment.
#[derive(Clone)]
pub struct Participation {
    /// The database of the installation.
    db: Database,
    /// Where "now" comes from, the clock enrolments take their start from.
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for Participation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Participation")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Participation {
    /// Reads participation from an installation, at the instants of a clock.
    #[must_use]
    pub fn new(db: Database, clock: Arc<dyn Clock>) -> Self {
        Self { db, clock }
    }

    /// The current instant, for binding.
    fn now(&self) -> time::OffsetDateTime {
        self.clock.now().as_offset_date_time()
    }

    /// Whether the account takes part in the course now.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn is_active(&self, course: CourseId, account: PrincipalId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} e join {} i on i.id = e.instance_id
             where i.course_id = $1 and e.account_id = $2 and {})",
            table(&self.db, "enrolment"),
            table(&self.db, "instance"),
            active("$3")
        )))
        .bind(course.as_uuid())
        .bind(account.as_uuid())
        .bind(self.now())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    /// Whether anyone is enrolled in the course, whatever the status.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn anyone(&self, course: CourseId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} e join {} i on i.id = e.instance_id where i.course_id = $1)",
            table(&self.db, "enrolment"),
            table(&self.db, "instance")
        )))
        .bind(course.as_uuid())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }
}

#[async_trait]
impl Participants for Participation {
    async fn together(&self, one: PrincipalId, other: PrincipalId) -> Result<bool> {
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {e} e join {i} i on i.id = e.instance_id where e.account_id = $1 and {active}
                 and i.course_id in (select i.course_id from {e} e join {i} i on i.id = e.instance_id
                                     where e.account_id = $2 and {active}))",
            e = table(&self.db, "enrolment"),
            i = table(&self.db, "instance"),
            active = active("$3")
        )))
        .bind(one.as_uuid())
        .bind(other.as_uuid())
        .bind(self.now())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }

    async fn takes_part(&self, course: CourseId, account: PrincipalId) -> Result<bool> {
        self.is_active(course, account).await
    }

    async fn any(&self, course: CourseId) -> Result<bool> {
        self.anyone(course).await
    }

    async fn accounts(&self, course: CourseId) -> Result<Vec<PrincipalId>> {
        let found: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select distinct e.account_id from {} e join {} i on i.id = e.instance_id
             where i.course_id = $1 and {} order by e.account_id",
            table(&self.db, "enrolment"),
            table(&self.db, "instance"),
            active("$2")
        )))
        .bind(course.as_uuid())
        .bind(self.now())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(found.into_iter().map(PrincipalId::from_uuid).collect())
    }

    async fn since(&self, course: CourseId, account: PrincipalId) -> Result<Option<Timestamp>> {
        let since: Option<time::OffsetDateTime> = sqlx::query_scalar(sql(format!(
            "select min(e.starts_at) from {} e join {} i on i.id = e.instance_id
             where i.course_id = $1 and e.account_id = $2 and {}",
            table(&self.db, "enrolment"),
            table(&self.db, "instance"),
            active("$3")
        )))
        .bind(course.as_uuid())
        .bind(account.as_uuid())
        .bind(self.now())
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(since.map(Timestamp::from_offset_date_time))
    }
}

impl Enrol {
    /// Whether the account takes part in the course now.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn is_active_participant(&self, course: CourseId, account: PrincipalId) -> Result<bool> {
        Participation::new(self.db.clone(), self.clock.clone())
            .is_active(course, account)
            .await
    }
}
