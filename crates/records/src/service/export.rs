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
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Exports of the grades of a course: someone who may see and export them asks, a job writes a CSV file with
//! what that person may see at that moment, the file is theirs alone, and it goes after a week.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::CourseId;
use kampus_module::ServiceContext;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::principal;
use crate::permissions::{EXPORT, VIEW_ALL};
use crate::{Export, ExportId, Records, sql};

/// How long the file of an export stays.
pub const EXPORT_LIFETIME: time::Duration = time::Duration::days(7);

/// The file area of exports.
pub const EXPORT_AREA: &str = "records.export";

/// How many exports one run makes.
const BATCH: i64 = 20;

/// The columns of an export.
const COLUMNS: &str =
    "id, course_id, include_inactive, state, file_id, error, created_at, completed_at, expires_at";

/// An export from its row.
fn export(row: &PgRow) -> Export {
    let stamp = |c: &str| {
        row.get::<Option<time::OffsetDateTime>, _>(c)
            .map(Timestamp::from_offset_date_time)
    };
    Export {
        id: ExportId::from_uuid(row.get("id")),
        course: CourseId::from_uuid(row.get("course_id")),
        include_inactive: row.get("include_inactive"),
        state: row.get("state"),
        file: row.get("file_id"),
        error: row.get("error"),
        created_at: Timestamp::from_offset_date_time(row.get("created_at")),
        completed_at: stamp("completed_at"),
        expires_at: stamp("expires_at"),
    }
}

/// A field of a CSV line: quoted when it has to be, and never read as a formula by a spreadsheet.
#[must_use]
pub fn csv_field(text: &str) -> String {
    let text = if text.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{text}")
    } else {
        text.to_owned()
    };
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text
    }
}

impl Records {
    /// Asks for an export of the grades of a course; needs `records.export` and `records.view_all` there.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn request_grades_export(
        &self,
        actor: &Actor,
        course: CourseId,
        include_inactive: bool,
    ) -> Result<Export> {
        let found = self.courses.course(course).await?;
        self.access.require(actor, EXPORT, found.scope).await?;
        self.access.require(actor, VIEW_ALL, found.scope).await?;
        let by = principal(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        let id = ExportId::generate();
        let mut tx = self.db.begin().await?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, course_id, requested_by, include_inactive) values ($1, $2, $3, $4)
             returning {COLUMNS}",
            self.table("export")
        )))
        .bind(id.as_uuid())
        .bind(course.as_uuid())
        .bind(by)
        .bind(include_inactive)
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        kampus_tasks::Tasks::new(self.db.clone())
            .enqueue_in(&mut tx, "records.export", &json!({}), None)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(export(&row))
    }

    /// An export, for the person who asked for it.
    ///
    /// # Errors
    ///
    /// `not_found` (`records.export_not_found`) for one of someone else.
    pub async fn grades_export(&self, actor: &Actor, id: ExportId) -> Result<Export> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1 and requested_by = $2",
            self.table("export")
        )))
        .bind(id.as_uuid())
        .bind(principal(actor))
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("records.export_not_found"))?;
        Ok(export(&row))
    }

    /// The exports the actor asked for in a course, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn grades_exports(&self, actor: &Actor, course: CourseId) -> Result<Vec<Export>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where course_id = $1 and requested_by = $2
             order by created_at desc, id desc limit 50",
            self.table("export")
        )))
        .bind(course.as_uuid())
        .bind(principal(actor))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(export).collect())
    }

    /// The CSV of the grades of a course as an actor sees them: one line per person, one column per record.
    async fn grades_csv(
        &self,
        actor: &Actor,
        course: CourseId,
        include_inactive: bool,
        services: &ServiceContext,
    ) -> Result<String> {
        let tree = self.tree(actor, course).await?;
        let mut lines = vec![
            ["account", "name"]
                .into_iter()
                .map(str::to_owned)
                .chain(tree.records.iter().map(|r| r.title.clone()))
                .map(|f| csv_field(&f))
                .collect::<Vec<_>>()
                .join(","),
        ];
        let users = services.service::<kampus_users::Users>()?;
        let mut after = None;
        loop {
            let page = self
                .grader_report(actor, course, 200, after, include_inactive)
                .await?;
            let accounts: Vec<kampus_identity::AccountId> = page
                .rows
                .iter()
                .map(|r| kampus_identity::AccountId::from_uuid(r.account.as_uuid()))
                .collect();
            let names = users.names_of(&accounts).await?;
            for row in &page.rows {
                let name = names
                    .get(&kampus_identity::AccountId::from_uuid(row.account.as_uuid()))
                    .cloned()
                    .unwrap_or_default();
                let mut fields = vec![row.account.to_string(), name];
                for record in &tree.records {
                    let value = row
                        .grades
                        .iter()
                        .find(|g| g.record == record.id)
                        .and_then(|g| {
                            g.text
                                .clone()
                                .or_else(|| g.final_value.map(|v| v.normalize().to_string()))
                        })
                        .unwrap_or_default();
                    fields.push(value);
                }
                lines.push(fields.iter().map(|f| csv_field(f)).collect::<Vec<_>>().join(","));
            }
            match page.next {
                Some(next) => after = Some(next),
                None => break,
            }
        }
        Ok(lines.join("\r\n") + "\r\n")
    }

    /// Makes the pending exports, at most a batch per run, and returns how many it made. Each is written with
    /// what the person who asked may see now; one that fails is recorded with the reason.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn run_exports(&self, services: &ServiceContext) -> Result<u64> {
        let rows = sqlx::query(sql(format!(
            "select id, course_id, requested_by, include_inactive from {} where state = 'pending'
             order by created_at limit $1",
            self.table("export")
        )))
        .bind(BATCH)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut made = 0;
        for row in &rows {
            let id: uuid::Uuid = row.get("id");
            let course = CourseId::from_uuid(row.get("course_id"));
            let by = PrincipalId::from_uuid(row.get("requested_by"));
            let outcome = self
                .write_export(
                    &Actor::Principal(by),
                    course,
                    row.get("include_inactive"),
                    id,
                    services,
                )
                .await;
            let (state, file, error) = match outcome {
                Ok(file) => ("completed", Some(file), None),
                Err(error) => ("failed", None, Some(error.code().to_owned())),
            };
            let changed = sqlx::query(sql(format!(
                "update {} set state = $2, file_id = $3, error = $4, completed_at = now(),
                     expires_at = case when $2 = 'completed' then $5 else null end
                 where id = $1 and state = 'pending'",
                self.table("export")
            )))
            .bind(id)
            .bind(state)
            .bind(file)
            .bind(error)
            .bind(time::OffsetDateTime::now_utc() + EXPORT_LIFETIME)
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
            if changed == 0 {
                if let Some(file) = file {
                    services
                        .service::<kampus_files::Files>()?
                        .remove(kampus_files::FileId::from_uuid(file))
                        .await
                        .ok();
                }
                continue;
            }
            made += u64::from(state == "completed");
        }
        Ok(made)
    }

    /// Writes the file of one export and returns it.
    async fn write_export(
        &self,
        actor: &Actor,
        course: CourseId,
        include_inactive: bool,
        id: uuid::Uuid,
        services: &ServiceContext,
    ) -> Result<uuid::Uuid> {
        let csv = self.grades_csv(actor, course, include_inactive, services).await?;
        let scope = self.courses.course(course).await?.scope;
        let owner = match actor {
            Actor::Principal(p) | Actor::ViewAs(p, _) => *p,
            Actor::Visitor(_) => return Err(Error::permission_denied("access.denied")),
        };
        let file = services
            .service::<kampus_files::Files>()?
            .store_for(
                EXPORT_AREA,
                scope,
                owner,
                &format!("grades-{id}.csv"),
                (bytes::Bytes::from(csv), "text/csv"),
            )
            .await?;
        Ok(file.id.as_uuid())
    }

    /// Removes the files of the exports whose week ended; returns how many expired.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn expire_exports(&self, services: &ServiceContext) -> Result<u64> {
        let due = sqlx::query(sql(format!(
            "select id, file_id from {} where state = 'completed' and expires_at < now()",
            self.table("export")
        )))
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let files = services.service::<kampus_files::Files>()?;
        for row in &due {
            if let Some(file) = row.get::<Option<uuid::Uuid>, _>("file_id") {
                files.remove(kampus_files::FileId::from_uuid(file)).await.ok();
            }
        }
        let ids: Vec<uuid::Uuid> = due.iter().map(|r| r.get("id")).collect();
        sqlx::query(sql(format!(
            "update {} set state = 'expired', file_id = null where id = any($1)",
            self.table("export")
        )))
        .bind(&ids)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(u64::try_from(ids.len()).unwrap_or(u64::MAX))
    }

    /// Whether a person may read the file of an export: they asked for it and it has not expired.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn export_readable(&self, file: uuid::Uuid, principal: Option<uuid::Uuid>) -> Result<bool> {
        let Some(principal) = principal else {
            return Ok(false);
        };
        sqlx::query_scalar(sql(format!(
            "select exists (select 1 from {} where file_id = $1 and requested_by = $2 and state = 'completed')",
            self.table("export")
        )))
        .bind(file)
        .bind(principal)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)
    }
}
