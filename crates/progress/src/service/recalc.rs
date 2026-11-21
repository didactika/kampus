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

//! Re-evaluation after the rules of a course module change: the course module is queued, and a job
//! evaluates everyone who did something on it or has a grade, leaving the decisions of teachers as they are.

use std::collections::HashSet;

use kampus_access::PrincipalId;
use kampus_activities::ModuleId;
use kampus_common::Result;
use serde_json::json;
use sqlx::PgConnection;

use crate::{Progress, sql};

/// How many course modules one run takes.
const BATCH: i64 = 20;

impl Progress {
    /// Queues a course module for re-evaluation, and a job to do it when it was not queued yet.
    pub(crate) async fn queue_in(&self, conn: &mut PgConnection, module: ModuleId) -> Result<()> {
        let queued = sqlx::query(sql(format!(
            "insert into {} (course_module_id) values ($1) on conflict do nothing",
            self.table("pending")
        )))
        .bind(module.as_uuid())
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if queued > 0 {
            kampus_tasks::Tasks::new(self.db.clone())
                .enqueue_in(conn, "progress.recalculate", &json!({}), None)
                .await?;
        }
        Ok(())
    }

    /// Re-evaluates the people of the course modules queued; returns how many course modules it did. A
    /// course module that fails stays queued for the next run.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn recalculate(&self) -> Result<u64> {
        let mut tx = self.db.begin().await?;
        let modules: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "delete from {t} q using (select course_module_id from {t} order by queued_at
                 limit $1 for update skip locked) s
             where q.course_module_id = s.course_module_id returning q.course_module_id",
            t = self.table("pending")
        )))
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for module in &modules {
            let module = ModuleId::from_uuid(*module);
            let mut people: HashSet<PrincipalId> = HashSet::new();
            let known: Vec<uuid::Uuid> = sqlx::query_scalar(sql(format!(
                "select account_id from {} where course_module_id = $1
                 union select account_id from {} where course_module_id = $1
                 union select account_id from {} where course_module_id = $1",
                self.table("signal"),
                self.table("activity_state"),
                self.table("type_signal")
            )))
            .bind(module.as_uuid())
            .fetch_all(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            people.extend(known.into_iter().map(PrincipalId::from_uuid));
            people.extend(self.records.graded_accounts(module).await?);
            for person in people {
                self.evaluate_in(&mut tx, module, person, "rules_changed").await?;
            }
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(u64::try_from(modules.len()).unwrap_or(u64::MAX))
    }
}
