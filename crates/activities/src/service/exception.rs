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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Exceptions: a change of the dates, the time limit or the attempts of a course module for one person or
//! one group, through one model for every type that supports them. What applies to a person is resolved in
//! one place: their own exception wins, setting by setting; otherwise, among the groups they belong to, the
//! most generous value of each setting; otherwise the value of the course module.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_courses::GroupId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::principal;
use crate::permissions::EXCEPTION_MANAGE;
use crate::{Activities, Module, ModuleId, sql};

kampus_common::define_id! {
    /// Identifies an exception.
    pub struct ExceptionId;
}

/// Whom an exception is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// One person.
    Person(PrincipalId),
    /// Every member of a group.
    Group(GroupId),
}

/// The settings an exception may change; each left out keeps the value below it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Values {
    /// When it opens.
    #[serde(default)]
    pub opens_at: Option<Timestamp>,
    /// When it closes.
    #[serde(default)]
    pub closes_at: Option<Timestamp>,
    /// When it is due.
    #[serde(default)]
    pub due_at: Option<Timestamp>,
    /// The last moment a late one is accepted.
    #[serde(default)]
    pub cutoff_at: Option<Timestamp>,
    /// The time a person has once started, in seconds.
    #[serde(default)]
    pub time_limit_seconds: Option<i32>,
    /// How many attempts a person has.
    #[serde(default)]
    pub attempts: Option<i32>,
}

impl Values {
    /// Whether it changes nothing.
    const fn is_empty(&self) -> bool {
        self.opens_at.is_none()
            && self.closes_at.is_none()
            && self.due_at.is_none()
            && self.cutoff_at.is_none()
            && self.time_limit_seconds.is_none()
            && self.attempts.is_none()
    }

    /// What this takes from `other` where it sets nothing.
    const fn or(self, other: Self) -> Self {
        Self {
            opens_at: pick(self.opens_at, other.opens_at),
            closes_at: pick(self.closes_at, other.closes_at),
            due_at: pick(self.due_at, other.due_at),
            cutoff_at: pick(self.cutoff_at, other.cutoff_at),
            time_limit_seconds: pick(self.time_limit_seconds, other.time_limit_seconds),
            attempts: pick(self.attempts, other.attempts),
        }
    }
}

/// The first of two optional values that is set.
const fn pick<T: Copy>(first: Option<T>, second: Option<T>) -> Option<T> {
    match first {
        Some(_) => first,
        None => second,
    }
}

/// An exception of a course module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Exception {
    /// The exception.
    pub id: ExceptionId,
    /// The course module.
    pub module: ModuleId,
    /// Whom it is for.
    pub target: Target,
    /// What it changes.
    #[serde(flatten)]
    pub values: Values,
    /// Why it was granted.
    pub reason: Option<String>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// The columns of an exception.
const COLUMNS: &str = "id, course_module_id, account_id, group_id, opens_at, closes_at, due_at, cutoff_at,
    time_limit_seconds, attempts, reason, version";

/// An instant read from a nullable column.
fn instant(row: &PgRow, column: &str) -> Option<Timestamp> {
    row.get::<Option<time::OffsetDateTime>, _>(column)
        .map(Timestamp::from_offset_date_time)
}

/// The values an exception row holds.
fn values_of(row: &PgRow) -> Values {
    Values {
        opens_at: instant(row, "opens_at"),
        closes_at: instant(row, "closes_at"),
        due_at: instant(row, "due_at"),
        cutoff_at: instant(row, "cutoff_at"),
        time_limit_seconds: row.get("time_limit_seconds"),
        attempts: row.get("attempts"),
    }
}

/// An exception from its row.
fn exception(row: &PgRow) -> Exception {
    let target = match row.get::<Option<uuid::Uuid>, _>("account_id") {
        Some(account) => Target::Person(PrincipalId::from_uuid(account)),
        None => Target::Group(GroupId::from_uuid(row.get("group_id"))),
    };
    Exception {
        id: ExceptionId::from_uuid(row.get("id")),
        module: ModuleId::from_uuid(row.get("course_module_id")),
        target,
        values: values_of(row),
        reason: row.get("reason"),
        version: row.get("version"),
    }
}

/// The most generous of several values: the earliest to open, the latest to close, the largest to allow.
fn generous(groups: &[Values]) -> Values {
    let earliest = |f: fn(&Values) -> Option<Timestamp>| groups.iter().filter_map(f).min();
    let latest = |f: fn(&Values) -> Option<Timestamp>| groups.iter().filter_map(f).max();
    Values {
        opens_at: earliest(|v| v.opens_at),
        closes_at: latest(|v| v.closes_at),
        due_at: latest(|v| v.due_at),
        cutoff_at: latest(|v| v.cutoff_at),
        time_limit_seconds: groups.iter().filter_map(|v| v.time_limit_seconds).max(),
        attempts: groups.iter().filter_map(|v| v.attempts).max(),
    }
}

/// The value of a setting of a course module as a timestamp, when it holds one.
fn setting_time(module: &Module, key: &str) -> Option<Timestamp> {
    module
        .settings
        .get(key)
        .and_then(Value::as_str)
        .and_then(|t| Timestamp::parse_rfc3339(t).ok())
}

/// The value of a setting of a course module as a whole number, when it holds one.
fn setting_number(module: &Module, key: &str) -> Option<i32> {
    module
        .settings
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
}

impl Activities {
    /// Fails unless the type of a course module supports exceptions.
    fn require_exceptions(&self, module: &Module) -> Result<()> {
        let supports = self
            .type_declaration(&module.type_code)
            .is_some_and(|t| t.features.contains(&"exceptions"));
        if supports {
            Ok(())
        } else {
            Err(Error::validation(
                "activities.exceptions_unsupported",
                vec![FieldError::new("type", "feature")],
            )
            .with_param("type", module.type_code.clone()))
        }
    }

    /// Grants an exception to a person or a group on a course module, replacing the one they had; needs
    /// `activities.exception.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`activities.exceptions_unsupported`) for a type
    /// without them, (`activities.exception_invalid`) for one that changes nothing or whose dates are out of
    /// order or numbers not positive, and for a group of another course.
    pub async fn set_exception(
        &self,
        actor: &Actor,
        module: ModuleId,
        target: Target,
        values: Values,
        reason: Option<&str>,
    ) -> Result<Exception> {
        let stored = self.stored(module).await?;
        self.access.require(actor, EXCEPTION_MANAGE, stored.scope).await?;
        self.require_exceptions(&stored)?;
        let mut problems = Vec::new();
        if values.is_empty() {
            problems.push(FieldError::new("values", "required"));
        }
        if let (Some(opens), Some(closes)) = (values.opens_at, values.closes_at)
            && closes <= opens
        {
            problems.push(FieldError::new("closes_at", "after_opens_at"));
        }
        if values.time_limit_seconds.is_some_and(|t| t <= 0) {
            problems.push(FieldError::new("time_limit_seconds", "positive"));
        }
        if values.attempts.is_some_and(|a| a < 0) {
            problems.push(FieldError::new("attempts", "min"));
        }
        if let Target::Group(group) = target {
            let course = self.courses.section(stored.section).await?.course;
            if !self
                .courses
                .groups(actor, course)
                .await?
                .iter()
                .any(|g| g.id == group)
            {
                problems.push(FieldError::new("group", "course"));
            }
        }
        if !problems.is_empty() {
            return Err(Error::validation("activities.exception_invalid", problems));
        }
        let (person, group) = match target {
            Target::Person(p) => (Some(p.as_uuid()), None),
            Target::Group(g) => (None, Some(g.as_uuid())),
        };
        let mut tx = self.db.begin().await?;
        let key = if person.is_some() {
            "account_id"
        } else {
            "group_id"
        };
        sqlx::query(sql(format!(
            "delete from {} where course_module_id = $1 and {key} = $2",
            self.table("exception")
        )))
        .bind(module.as_uuid())
        .bind(person.or(group))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let row = sqlx::query(sql(format!(
            "insert into {} (id, course_module_id, account_id, group_id, opens_at, closes_at, due_at, cutoff_at,
             time_limit_seconds, attempts, reason, created_by) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             returning {COLUMNS}",
            self.table("exception")
        )))
        .bind(ExceptionId::generate().as_uuid())
        .bind(module.as_uuid())
        .bind(person)
        .bind(group)
        .bind(values.opens_at.map(|t| t.as_offset_date_time()))
        .bind(values.closes_at.map(|t| t.as_offset_date_time()))
        .bind(values.due_at.map(|t| t.as_offset_date_time()))
        .bind(values.cutoff_at.map(|t| t.as_offset_date_time()))
        .bind(values.time_limit_seconds)
        .bind(values.attempts)
        .bind(reason.map(str::trim).filter(|r| !r.is_empty()))
        .bind(principal(actor))
        .fetch_one(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            crate::events::EXCEPTION_CHANGED,
            &stored,
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(exception(&row))
    }

    /// The exceptions of a course module; needs `activities.exception.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn exceptions(&self, actor: &Actor, module: ModuleId) -> Result<Vec<Exception>> {
        let stored = self.stored(module).await?;
        self.access.require(actor, EXCEPTION_MANAGE, stored.scope).await?;
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where course_module_id = $1 order by created_at, id",
            self.table("exception")
        )))
        .bind(module.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(exception).collect())
    }

    /// Removes an exception; needs `activities.exception.manage` in its course module.
    ///
    /// # Errors
    ///
    /// `not_found` (`activities.exception_not_found`), `permission_denied`.
    pub async fn remove_exception(&self, actor: &Actor, id: ExceptionId) -> Result<()> {
        let module: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select course_module_id from {} where id = $1",
            self.table("exception")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let module = module.ok_or_else(|| Error::not_found("activities.exception_not_found"))?;
        let stored = self.stored(ModuleId::from_uuid(module)).await?;
        self.access.require(actor, EXCEPTION_MANAGE, stored.scope).await?;
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where id = $1",
            self.table("exception")
        )))
        .bind(id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            &mut tx,
            crate::events::EXCEPTION_CHANGED,
            &stored,
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// What applies to a person on a course module: their own exception, setting by setting; otherwise the
    /// most generous of their groups'; otherwise the course module's own value (the settings `opens_at`,
    /// `closes_at`, `due_at`, `cutoff_at`, `time_limit_seconds` and `attempts`, when its type keeps them).
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn effective_values(&self, module: ModuleId, account: PrincipalId) -> Result<Values> {
        let stored = self.stored(module).await?;
        let course = self.courses.section(stored.section).await?.course;
        let groups: Vec<uuid::Uuid> = self
            .courses
            .groups_of(course, account)
            .await?
            .into_iter()
            .map(|g| g.as_uuid())
            .collect();
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where course_module_id = $1 and (account_id = $2 or group_id = any($3))",
            self.table("exception")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .bind(&groups)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let own = rows
            .iter()
            .find(|r| r.get::<Option<uuid::Uuid>, _>("account_id").is_some())
            .map_or_else(Values::default, values_of);
        let of_groups: Vec<Values> = rows
            .iter()
            .filter(|r| r.get::<Option<uuid::Uuid>, _>("group_id").is_some())
            .map(values_of)
            .collect();
        let base = Values {
            opens_at: setting_time(&stored, "opens_at"),
            closes_at: setting_time(&stored, "closes_at"),
            due_at: setting_time(&stored, "due_at"),
            cutoff_at: setting_time(&stored, "cutoff_at"),
            time_limit_seconds: setting_number(&stored, "time_limit_seconds"),
            attempts: setting_number(&stored, "attempts"),
        };
        Ok(own.or(generous(&of_groups)).or(base))
    }
}

impl Activities {
    /// The scope of the course module an exception belongs to, for the permission to manage it.
    ///
    /// # Errors
    ///
    /// `not_found` (`activities.exception_not_found`) for an unknown exception.
    pub async fn exception_scope(&self, id: ExceptionId) -> Result<kampus_access::ScopeId> {
        let module: Option<uuid::Uuid> = sqlx::query_scalar(sql(format!(
            "select course_module_id from {} where id = $1",
            self.table("exception")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let module = module.ok_or_else(|| Error::not_found("activities.exception_not_found"))?;
        self.scope_of(ModuleId::from_uuid(module)).await
    }

    /// What applies to a person on a course module, for that person (who sees the course module) or for
    /// someone with `activities.exception.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found` for a course module the actor does not see, `permission_denied` when asking about
    /// someone else without the permission.
    pub async fn effective_values_for(
        &self,
        actor: &Actor,
        module: ModuleId,
        account: PrincipalId,
    ) -> Result<Values> {
        let stored = self.module(actor, module).await?;
        if principal(actor) != Some(account.as_uuid()) {
            self.access.require(actor, EXCEPTION_MANAGE, stored.scope).await?;
        }
        self.effective_values(module, account).await
    }
}

impl Activities {
    /// The dates, time limit and attempts a course module has of its own, without any exception, for the
    /// modules that show or project them, such as the calendar.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn module_values(&self, module: ModuleId) -> Result<Values> {
        let stored = self.stored(module).await?;
        Ok(Values {
            opens_at: setting_time(&stored, "opens_at"),
            closes_at: setting_time(&stored, "closes_at"),
            due_at: setting_time(&stored, "due_at"),
            cutoff_at: setting_time(&stored, "cutoff_at"),
            time_limit_seconds: setting_number(&stored, "time_limit_seconds"),
            attempts: setting_number(&stored, "attempts"),
        })
    }
}
