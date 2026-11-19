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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The custom fields of courses: the fields customisation defines for courses in the scopes above a course
//! and in it, with the values this module links to each course.

use kampus_access::Actor;
use kampus_common::{Error, Result};
use kampus_customise::permissions::FIELD_EDIT;
use kampus_customise::{Customise, FieldEntry, FieldValueId};
use serde_json::Value;
use sqlx::Row;

use crate::settings::customise::COURSE_KIND;
use crate::{CourseId, Courses, sql};

impl Courses {
    /// Customisation, which custom fields need.
    fn customise(&self) -> Result<&Customise> {
        self.customise
            .as_ref()
            .ok_or_else(|| Error::internal("courses.customise_missing"))
    }

    /// The custom fields of a course with their values, those whose values the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course.
    pub async fn course_fields(&self, actor: &Actor, course: CourseId) -> Result<Vec<FieldEntry>> {
        let customise = self.customise()?;
        let course = self.course(course).await?;
        let fields = customise.fields_for(COURSE_KIND, course.scope).await?;
        let rows = sqlx::query(sql(format!(
            "select field_id, field_value_id from {} where course_id = $1",
            self.table("course_field")
        )))
        .bind(course.id.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let linked: Vec<(uuid::Uuid, FieldValueId)> = rows
            .iter()
            .map(|r| {
                (
                    r.get("field_id"),
                    FieldValueId::from_uuid(r.get("field_value_id")),
                )
            })
            .collect();
        let ids: Vec<FieldValueId> = linked.iter().map(|(_, v)| *v).collect();
        let values = customise.values(&ids).await?;
        let mut entries = Vec::new();
        for field in fields {
            if !customise.sees_values(actor, &field, course.scope).await? {
                continue;
            }
            let value = linked
                .iter()
                .find(|(f, _)| *f == field.id.as_uuid())
                .and_then(|(_, v)| values.get(v).cloned())
                .unwrap_or(Value::Null);
            entries.push(FieldEntry { field, value });
        }
        Ok(entries)
    }

    /// Sets the value of a custom field of a course, or clears it with null; needs
    /// `customise.field.edit` in the course.
    ///
    /// # Errors
    ///
    /// `not_found` (`courses.field_not_found`) for a field that does not apply to the course,
    /// `permission_denied` without the permission, `validation` for a value that does not fit the field.
    pub async fn set_course_field(
        &self,
        actor: &Actor,
        course: CourseId,
        code: &str,
        value: &Value,
    ) -> Result<FieldEntry> {
        let customise = self.customise()?;
        let course = self.course(course).await?;
        self.access.require(actor, FIELD_EDIT, course.scope).await?;
        let field = customise
            .fields_for(COURSE_KIND, course.scope)
            .await?
            .into_iter()
            .find(|f| f.code == code)
            .ok_or_else(|| Error::not_found("courses.field_not_found").with_param("code", code.to_owned()))?;
        let mut tx = self.db.begin().await?;
        let existing: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select field_value_id from {} where course_id = $1 and field_id = $2 for update",
            self.table("course_field")
        )))
        .bind(course.id.as_uuid())
        .bind(field.id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let written = customise
            .write_value_in(&mut tx, &field, existing.map(FieldValueId::from_uuid), value)
            .await?;
        if let Some(id) = written {
            sqlx::query(sql(format!(
                "insert into {} (course_id, field_id, field_value_id) values ($1, $2, $3)",
                self.table("course_field")
            )))
            .bind(course.id.as_uuid())
            .bind(field.id.as_uuid())
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        tx.commit().await.map_err(kampus_db::classify)?;
        let value = match written {
            Some(id) => customise.values(&[id]).await?.remove(&id).unwrap_or(Value::Null),
            None => Value::Null,
        };
        Ok(FieldEntry { field, value })
    }
}
