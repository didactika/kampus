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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Custom fields: typed fields a scope adds to a kind of entity, applying there and in every scope below;
//! a field of a nearer scope replaces one of the same code above. Values are typed and validated when
//! written, and each module owns the values of its entities through a link table of its own.

use std::collections::BTreeMap;

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use serde_json::{Value, json};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use super::principal;
use crate::permissions::{FIELD_MANAGE, FIELD_VIEW};
use crate::{Choice, Customise, Field, FieldId, FieldSpec, FieldType, FieldValueId, FieldVisibility, sql};

/// The longest text value when a field says nothing.
const MAX_TEXT: i32 = 1000;

/// The columns of a field.
const COLUMNS: &str = "f.id, f.kind_code, f.scope_id, f.code, f.name, f.type, f.required, f.visibility, f.max_length, \
    f.version";

/// Checks that a value fits its field; returns the choice codes of a field of several choices.
fn checked(field: &Field, value: &Value) -> Result<Vec<String>> {
    let choice = |code: &str| field.choices.iter().any(|c| c.code == code);
    let fits = match field.field_type {
        FieldType::Text => value.as_str().is_some_and(|t| {
            t.chars().count() <= usize::try_from(field.max_length.unwrap_or(MAX_TEXT)).unwrap_or(0)
        }),
        FieldType::Integer => value.as_i64().is_some(),
        FieldType::Date => value.as_str().is_some_and(|t| {
            time::Date::parse(t, &time::format_description::well_known::Iso8601::DATE).is_ok()
        }),
        FieldType::Boolean => value.is_boolean(),
        FieldType::Choice => value.as_str().is_some_and(choice),
        FieldType::Choices => value
            .as_array()
            .is_some_and(|codes| codes.iter().all(|c| c.as_str().is_some_and(choice))),
        FieldType::Account => value.as_str().is_some_and(|t| uuid::Uuid::parse_str(t).is_ok()),
    };
    if !fits {
        return Err(invalid(field));
    }
    Ok(value.as_array().map_or_else(Vec::new, |a| {
        a.iter().filter_map(Value::as_str).map(str::to_owned).collect()
    }))
}

/// The error of a value that does not fit its field.
fn invalid(field: &Field) -> Error {
    Error::validation(
        "customise.value_invalid",
        vec![FieldError::new(field.code.clone(), field.field_type.as_str())],
    )
}

impl Customise {
    /// Adds a custom field to a kind of entity in a scope; needs `customise.field.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an unknown kind, a code that is not
    /// one, or a choice field without choices, `conflict` (`customise.field_taken`) for a code the scope
    /// has for that kind.
    pub async fn create_field(
        &self,
        actor: &Actor,
        kind: &str,
        scope: ScopeId,
        spec: FieldSpec,
    ) -> Result<Field> {
        self.access.require(actor, FIELD_MANAGE, scope).await?;
        let mut problems = Vec::new();
        let code = spec.code.trim().to_owned();
        if code.is_empty()
            || !code
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            problems.push(FieldError::new("code", "code"));
        }
        if spec.name.trim().is_empty() {
            problems.push(FieldError::new("name", "required"));
        }
        let with_choices = matches!(spec.field_type, FieldType::Choice | FieldType::Choices);
        if with_choices == spec.choices.is_empty() {
            problems.push(FieldError::new("choices", "type"));
        }
        if !problems.is_empty() {
            return Err(Error::validation("customise.field_invalid", problems));
        }
        let mut tx = self.db.begin().await?;
        let id = FieldId::generate();
        sqlx::query(sql(format!(
            "insert into {fields} (id, kind_code, scope_id, code, name, type, required, visibility, max_length, sort_key)
             select $1, k.code, $3, $4, $5, $6, $7, $8, $9, $1::text
             from {kinds} k where k.code = $2 and k.retired_at is null",
            fields = self.table("field"),
            kinds = self.table("field_kind")
        )))
        .bind(id.as_uuid())
        .bind(kind)
        .bind(scope.as_uuid())
        .bind(&code)
        .bind(spec.name.trim())
        .bind(spec.field_type.as_str())
        .bind(spec.required)
        .bind(match spec.visibility {
            FieldVisibility::Public => "public",
            FieldVisibility::Private => "private",
        })
        .bind(spec.max_length)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.unique_violation" {
                Error::conflict("customise.field_taken")
            } else {
                e
            }
        })
        .and_then(|r| {
            if r.rows_affected() == 1 {
                Ok(())
            } else {
                Err(Error::validation("customise.field_kind_unknown", vec![FieldError::new("kind", "declared")]))
            }
        })?;
        for (position, choice) in spec.choices.iter().enumerate() {
            sqlx::query(sql(format!(
                "insert into {} (id, field_id, code, label, sort_key) values ($1, $2, $3, $4, $5)",
                self.table("field_choice")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(id.as_uuid())
            .bind(choice.code.trim())
            .bind(choice.label.trim())
            .bind(format!("{position:06}"))
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)
            .map_err(|e| {
                if e.code() == "db.unique_violation" {
                    Error::validation(
                        "customise.field_invalid",
                        vec![FieldError::new("choices", "unique")],
                    )
                } else {
                    e
                }
            })?;
        }
        self.record_in(
            &mut tx,
            "field",
            &format!("{kind}/{code}"),
            scope,
            principal(actor),
            (None, Some(json!(spec.name))),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.field(id).await
    }

    /// A custom field with its choices.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown field.
    pub async fn field(&self, id: FieldId) -> Result<Field> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} f where f.id = $1",
            self.table("field")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("customise.field_not_found"))?;
        let mut fields = self.with_choices(vec![row]).await?;
        fields
            .pop()
            .ok_or_else(|| Error::not_found("customise.field_not_found"))
    }

    /// The custom fields that apply to a kind of entity in a scope: those of the scope and of every scope
    /// above, the nearest one winning for a code.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn fields_for(&self, kind: &str, scope: ScopeId) -> Result<Vec<Field>> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let depths = self.depths(&mut conn, scope).await?;
        let ids: Vec<uuid::Uuid> = depths.keys().copied().collect();
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} f where f.kind_code = $1 and f.scope_id = any($2) order by f.sort_key, f.id",
            self.table("field")
        )))
        .bind(kind)
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let mut nearest: BTreeMap<String, (i32, PgRow)> = BTreeMap::new();
        for row in rows {
            let code: String = row.get("code");
            let depth = depths
                .get(&row.get::<uuid::Uuid, _>("scope_id"))
                .copied()
                .unwrap_or(i32::MAX);
            if nearest.get(&code).is_none_or(|(d, _)| depth < *d) {
                nearest.insert(code, (depth, row));
            }
        }
        self.with_choices(nearest.into_values().map(|(_, r)| r).collect())
            .await
    }

    /// Removes a custom field with every value of it; needs `customise.field.manage` in its scope.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission.
    pub async fn delete_field(&self, actor: &Actor, id: FieldId) -> Result<()> {
        let field = self.field(id).await?;
        self.access.require(actor, FIELD_MANAGE, field.scope).await?;
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("field"))))
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        self.record_in(
            &mut tx,
            "field",
            &format!("{}/{}", field.kind, field.code),
            field.scope,
            principal(actor),
            (Some(json!(field.name)), None),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// Whether the actor sees the values of a field in a scope: public ones always, private ones with
    /// `customise.field.view` there.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn sees_values(&self, actor: &Actor, field: &Field, scope: ScopeId) -> Result<bool> {
        if field.visibility == FieldVisibility::Public {
            return Ok(true);
        }
        Ok(self.access.check(actor, FIELD_VIEW, scope).await?.allowed)
    }

    /// Writes the value of a field for one entity inside the owning module's transaction, replacing the
    /// one it had; returns the new value's id, or none when the value is null.
    ///
    /// # Errors
    ///
    /// `validation` (`customise.value_invalid`) for a value that does not fit the field, or
    /// (`customise.value_required`) for null in a required field.
    pub async fn write_value_in(
        &self,
        conn: &mut PgConnection,
        field: &Field,
        existing: Option<FieldValueId>,
        value: &Value,
    ) -> Result<Option<FieldValueId>> {
        if let Some(old) = existing {
            sqlx::query(sql(format!(
                "delete from {} where id = $1",
                self.table("field_value")
            )))
            .bind(old.as_uuid())
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        if value.is_null() {
            if field.required {
                return Err(Error::validation(
                    "customise.value_required",
                    vec![FieldError::new(field.code.clone(), "required")],
                ));
            }
            return Ok(None);
        }
        let codes = checked(field, value)?;
        let id = FieldValueId::generate();
        let (column_list, value_list) = match field.field_type {
            FieldType::Choices => (String::new(), String::new()),
            FieldType::Choice => (
                ", choice_id".to_owned(),
                format!(
                    ", (select id from {} where field_id = $2 and code = $3 ->> 0)",
                    self.table("field_choice")
                ),
            ),
            other => {
                let (column, cast) = match other {
                    FieldType::Integer => ("integer_value", "::bigint"),
                    FieldType::Date => ("date_value", "::date"),
                    FieldType::Boolean => ("boolean_value", "::boolean"),
                    FieldType::Account => ("account_id", "::uuid"),
                    _ => ("text_value", ""),
                };
                (format!(", {column}"), format!(", ($3 ->> 0){cast}"))
            }
        };
        let scalar = match value {
            Value::String(text) => json!(text),
            other => json!(other.to_string()),
        };
        sqlx::query(sql(format!(
            "insert into {} (id, field_id{column_list}) values ($1, $2{value_list})",
            self.table("field_value")
        )))
        .bind(id.as_uuid())
        .bind(field.id.as_uuid())
        .bind(json!([scalar]))
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)
        .map_err(|e| {
            if e.code() == "db.foreign_key_violation" {
                invalid(field)
            } else {
                e
            }
        })?;
        if field.field_type == FieldType::Choices {
            sqlx::query(sql(format!(
                "insert into {} (field_value_id, choice_id) select $1, c.id from {} c
                 where c.field_id = $2 and c.code = any($3) on conflict do nothing",
                self.table("field_value_choice"),
                self.table("field_choice")
            )))
            .bind(id.as_uuid())
            .bind(field.id.as_uuid())
            .bind(codes)
            .execute(&mut *conn)
            .await
            .map_err(kampus_db::classify)?;
        }
        Ok(Some(id))
    }

    /// The values these ids hold, as the field types write them.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn values(&self, ids: &[FieldValueId]) -> Result<BTreeMap<FieldValueId, Value>> {
        let ids: Vec<uuid::Uuid> = ids.iter().map(FieldValueId::as_uuid).collect();
        let rows = sqlx::query(sql(format!(
            "select v.id, f.type, v.text_value, v.integer_value, v.date_value::text as date_value, v.boolean_value,
             v.account_id, c.code as choice,
             (select coalesce(array_agg(cc.code order by cc.sort_key), '{{}}') from {mc} m
              join {choices} cc on cc.id = m.choice_id where m.field_value_id = v.id) as choices
             from {values} v join {fields} f on f.id = v.field_id left join {choices} c on c.id = v.choice_id
             where v.id = any($1)",
            mc = self.table("field_value_choice"),
            choices = self.table("field_choice"),
            values = self.table("field_value"),
            fields = self.table("field")
        )))
        .bind(&ids)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| {
                let value = match r.get::<String, _>("type").as_str() {
                    "text" => json!(r.get::<Option<String>, _>("text_value")),
                    "integer" => json!(r.get::<Option<i64>, _>("integer_value")),
                    "date" => json!(r.get::<Option<String>, _>("date_value")),
                    "boolean" => json!(r.get::<Option<bool>, _>("boolean_value")),
                    "choice" => json!(r.get::<Option<String>, _>("choice")),
                    "choices" => json!(r.get::<Vec<String>, _>("choices")),
                    _ => json!(r.get::<Option<uuid::Uuid>, _>("account_id")),
                };
                (FieldValueId::from_uuid(r.get("id")), value)
            })
            .collect())
    }

    /// Fields from their rows, with their choices in order.
    async fn with_choices(&self, rows: Vec<PgRow>) -> Result<Vec<Field>> {
        let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.get("id")).collect();
        let choices = sqlx::query(sql(format!(
            "select field_id, code, label from {} where field_id = any($1) order by sort_key",
            self.table("field_choice")
        )))
        .bind(&ids)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.iter()
            .map(|r| {
                let id: uuid::Uuid = r.get("id");
                Ok(Field {
                    id: FieldId::from_uuid(id),
                    kind: r.get("kind_code"),
                    scope: ScopeId::from_uuid(r.get("scope_id")),
                    code: r.get("code"),
                    name: r.get("name"),
                    field_type: r.get::<String, _>("type").parse()?,
                    required: r.get("required"),
                    visibility: if r.get::<String, _>("visibility") == "private" {
                        FieldVisibility::Private
                    } else {
                        FieldVisibility::Public
                    },
                    max_length: r.get("max_length"),
                    choices: choices
                        .iter()
                        .filter(|c| c.get::<uuid::Uuid, _>("field_id") == id)
                        .map(|c| Choice {
                            code: c.get("code"),
                            label: c.get("label"),
                        })
                        .collect(),
                    version: r.get("version"),
                })
            })
            .collect()
    }
}
