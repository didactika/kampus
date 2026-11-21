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

//! The kinds of access condition activities provides itself: a window of dates, and belonging to a group, to
//! one of the groups of a set, or to any group of the course.

use kampus_access::PrincipalId;
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_courses::{CourseId, GroupId, GroupSetId};
use kampus_module::{
    ConditionCheck, ConditionFuture, ConditionKindDeclaration, ConditionWrite, nothing_depends,
};
use serde_json::{Value, json};
use sqlx::Row;

use crate::{Activities, sql};

/// The kind of a window of dates.
pub const DATE: &str = "activities.date";

/// The kind of belonging to a group.
pub const GROUP: &str = "activities.group";

/// The kinds of this module.
pub const KINDS: &[ConditionKindDeclaration] = &[
    ConditionKindDeclaration {
        code: DATE,
        summary: "From a moment, until a moment, or between both",
        store: store_date,
        params: date_params,
        met: date_met,
        depending_on: nothing_depends,
    },
    ConditionKindDeclaration {
        code: GROUP,
        summary: "Belonging to a group, to one of the groups of a set, or to any group of the course",
        store: store_group,
        params: group_params,
        met: group_met,
        depending_on: nothing_depends,
    },
    ConditionKindDeclaration {
        code: PROFILE,
        summary: "A field of the profile of the person is, is not, contains, starts or ends with a value, or is empty",
        store: store_profile,
        params: profile_params,
        met: profile_met,
        depending_on: nothing_depends,
    },
];

/// The kind of a field of the profile.
pub const PROFILE: &str = "activities.profile";

/// A wrong parameter.
fn wrong(field: &str, rule: &str) -> Error {
    Error::validation("activities.condition_invalid", vec![FieldError::new(field, rule)])
}

/// A moment among the parameters, when given.
fn moment(params: &Value, field: &str) -> Result<Option<Timestamp>> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Timestamp::parse_rfc3339(text)
            .map(Some)
            .map_err(|_| wrong(field, "format")),
        Some(_) => Err(wrong(field, "format")),
    }
}

/// An id among the parameters, when given.
fn id(params: &Value, field: &str) -> Result<Option<uuid::Uuid>> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => uuid::Uuid::parse_str(text)
            .map(Some)
            .map_err(|_| wrong(field, "format")),
        Some(_) => Err(wrong(field, "format")),
    }
}

/// Keeps a window of dates: at least one end, the end after the start.
fn store_date(write: ConditionWrite<'_>) -> ConditionFuture<'_, ()> {
    Box::pin(async move {
        let activities = write.services.service::<Activities>()?;
        let opens = moment(write.params, "opens_at")?;
        let closes = moment(write.params, "closes_at")?;
        if opens.is_none() && closes.is_none() {
            return Err(wrong("opens_at", "required"));
        }
        if let (Some(opens), Some(closes)) = (opens, closes)
            && closes <= opens
        {
            return Err(wrong("closes_at", "after"));
        }
        sqlx::query(sql(format!(
            "insert into {} (condition_id, opens_at, closes_at) values ($1, $2, $3)",
            activities.table("condition_date")
        )))
        .bind(write.condition)
        .bind(opens.map(|t| t.as_offset_date_time()))
        .bind(closes.map(|t| t.as_offset_date_time()))
        .execute(&mut *write.conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    })
}

/// The window of a leaf.
async fn window(
    activities: &Activities,
    condition: uuid::Uuid,
) -> Result<Option<(Option<Timestamp>, Option<Timestamp>)>> {
    let row = sqlx::query(sql(format!(
        "select opens_at, closes_at from {} where condition_id = $1",
        activities.table("condition_date")
    )))
    .bind(condition)
    .fetch_optional(activities.db.pool())
    .await
    .map_err(kampus_db::classify)?;
    Ok(row.map(|r| {
        let read = |c: &str| {
            r.get::<Option<time::OffsetDateTime>, _>(c)
                .map(Timestamp::from_offset_date_time)
        };
        (read("opens_at"), read("closes_at"))
    }))
}

/// The parameters of a window.
fn date_params(check: ConditionCheck<'_>) -> ConditionFuture<'_, Option<Value>> {
    Box::pin(async move {
        let activities = check.services.service::<Activities>()?;
        Ok(window(&activities, check.condition)
            .await?
            .map(|(opens, closes)| json!({ "opens_at": opens, "closes_at": closes })))
    })
}

/// Whether now is inside the window: from its start included, until its end excluded.
fn date_met(check: ConditionCheck<'_>) -> ConditionFuture<'_, bool> {
    Box::pin(async move {
        let activities = check.services.service::<Activities>()?;
        let Some((opens, closes)) = window(&activities, check.condition).await? else {
            return Ok(false);
        };
        let now = Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc());
        Ok(opens.is_none_or(|o| now >= o) && closes.is_none_or(|c| now < c))
    })
}

/// Keeps a group condition: a group or a set of the course, or neither for any group.
fn store_group(write: ConditionWrite<'_>) -> ConditionFuture<'_, ()> {
    Box::pin(async move {
        let activities = write.services.service::<Activities>()?;
        let group = id(write.params, "group")?;
        let set = id(write.params, "group_set")?;
        if group.is_some() && set.is_some() {
            return Err(wrong("group_set", "exclusive"));
        }
        let course = CourseId::from_uuid(write.course);
        if let Some(group) = group {
            let found = activities
                .courses
                .group(GroupId::from_uuid(group))
                .await
                .map_err(|_| wrong("group", "exists"))?;
            if found.course != course {
                return Err(wrong("group", "same_course"));
            }
        }
        if let Some(set) = set {
            let found = activities
                .courses
                .group_set(GroupSetId::from_uuid(set))
                .await
                .map_err(|_| wrong("group_set", "exists"))?;
            if found.course != course {
                return Err(wrong("group_set", "same_course"));
            }
        }
        sqlx::query(sql(format!(
            "insert into {} (condition_id, group_id, group_set_id) values ($1, $2, $3)",
            activities.table("condition_group")
        )))
        .bind(write.condition)
        .bind(group)
        .bind(set)
        .execute(&mut *write.conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    })
}

/// The group and set of a leaf, when it exists.
async fn group_leaf(
    activities: &Activities,
    condition: uuid::Uuid,
) -> Result<Option<(Option<uuid::Uuid>, Option<uuid::Uuid>)>> {
    let row = sqlx::query(sql(format!(
        "select group_id, group_set_id from {} where condition_id = $1",
        activities.table("condition_group")
    )))
    .bind(condition)
    .fetch_optional(activities.db.pool())
    .await
    .map_err(kampus_db::classify)?;
    Ok(row.map(|r| (r.get("group_id"), r.get("group_set_id"))))
}

/// The parameters of a group condition.
fn group_params(check: ConditionCheck<'_>) -> ConditionFuture<'_, Option<Value>> {
    Box::pin(async move {
        let activities = check.services.service::<Activities>()?;
        Ok(group_leaf(&activities, check.condition)
            .await?
            .map(|(group, set)| json!({ "group": group, "group_set": set })))
    })
}

/// Whether the person belongs to the group, to a group of the set, or to any group of the course.
fn group_met(check: ConditionCheck<'_>) -> ConditionFuture<'_, bool> {
    Box::pin(async move {
        let activities = check.services.service::<Activities>()?;
        let Some((group, set)) = group_leaf(&activities, check.condition).await? else {
            return Ok(false);
        };
        let account = PrincipalId::from_uuid(check.account);
        let course = activities.course_of_condition(check.condition).await?;
        let mine = activities.courses.groups_of(course, account).await?;
        Ok(match (group, set) {
            (Some(group), _) => mine.contains(&GroupId::from_uuid(group)),
            (None, Some(set)) => {
                let set = activities.courses.group_set(GroupSetId::from_uuid(set)).await?;
                set.groups.iter().any(|g| mine.contains(g))
            }
            (None, None) => !mine.is_empty(),
        })
    })
}

/// The fields of the profile a condition may look at.
pub const PROFILE_FIELDS: &[&str] = &[
    "given_name",
    "family_name",
    "display_name",
    "language_code",
    "time_zone",
];

/// How a profile condition compares the field.
pub const PROFILE_OPERATORS: &[&str] = &[
    "is",
    "is_not",
    "contains",
    "starts_with",
    "ends_with",
    "empty",
    "not_empty",
];

/// Keeps a profile condition: a field, an operator and, but for `empty` and `not_empty`, a value.
fn store_profile(write: ConditionWrite<'_>) -> ConditionFuture<'_, ()> {
    Box::pin(async move {
        let activities = write.services.service::<Activities>()?;
        let text = |field: &str| write.params.get(field).and_then(Value::as_str);
        let field = text("field")
            .filter(|f| PROFILE_FIELDS.contains(f))
            .ok_or_else(|| wrong("field", "one_of"))?;
        let operator = text("operator")
            .filter(|o| PROFILE_OPERATORS.contains(o))
            .ok_or_else(|| wrong("operator", "one_of"))?;
        let value = text("value").map(str::trim).filter(|v| !v.is_empty());
        if value.is_none() && !matches!(operator, "empty" | "not_empty") {
            return Err(wrong("value", "required"));
        }
        sqlx::query(sql(format!(
            "insert into {} (condition_id, field, operator, value) values ($1, $2, $3, $4)",
            activities.table("condition_profile")
        )))
        .bind(write.condition)
        .bind(field)
        .bind(operator)
        .bind(value)
        .execute(&mut *write.conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(())
    })
}

/// The field, operator and value of a profile condition.
async fn profile_leaf(
    activities: &Activities,
    condition: uuid::Uuid,
) -> Result<Option<(String, String, Option<String>)>> {
    let row = sqlx::query(sql(format!(
        "select field, operator, value from {} where condition_id = $1",
        activities.table("condition_profile")
    )))
    .bind(condition)
    .fetch_optional(activities.db.pool())
    .await
    .map_err(kampus_db::classify)?;
    Ok(row.map(|r| (r.get("field"), r.get("operator"), r.get("value"))))
}

/// The parameters of a profile condition.
fn profile_params(check: ConditionCheck<'_>) -> ConditionFuture<'_, Option<Value>> {
    Box::pin(async move {
        let activities = check.services.service::<Activities>()?;
        Ok(profile_leaf(&activities, check.condition)
            .await?
            .map(|(field, operator, value)| json!({ "field": field, "operator": operator, "value": value })))
    })
}

/// Whether the field of the profile of the person compares as asked, ignoring case.
fn profile_met(check: ConditionCheck<'_>) -> ConditionFuture<'_, bool> {
    Box::pin(async move {
        let activities = check.services.service::<Activities>()?;
        let Some((field, operator, value)) = profile_leaf(&activities, check.condition).await? else {
            return Ok(false);
        };
        let users = check.services.service::<kampus_users::Users>()?;
        let account = kampus_identity::AccountId::from_uuid(check.account);
        let Ok(profile) = users
            .profile(
                &kampus_access::Actor::Principal(PrincipalId::from_uuid(check.account)),
                account,
            )
            .await
        else {
            return Ok(false);
        };
        let actual = match field.as_str() {
            "given_name" => profile.given_name,
            "family_name" => profile.family_name,
            "display_name" => profile.display_name,
            "language_code" => profile.language_code,
            "time_zone" => profile.time_zone,
            _ => None,
        }
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty());
        let wanted = value.unwrap_or_default().to_lowercase();
        Ok(match (operator.as_str(), actual) {
            ("empty", actual) => actual.is_none(),
            ("not_empty", actual) => actual.is_some(),
            (_, None) => operator == "is_not",
            ("is", Some(actual)) => actual == wanted,
            ("is_not", Some(actual)) => actual != wanted,
            ("contains", Some(actual)) => actual.contains(&wanted),
            ("starts_with", Some(actual)) => actual.starts_with(&wanted),
            ("ends_with", Some(actual)) => actual.ends_with(&wanted),
            _ => false,
        })
    })
}

impl Activities {
    /// The course a leaf belongs to, through the root of its tree.
    ///
    /// # Errors
    ///
    /// `not_found` (`activities.condition_not_found`) for an unknown leaf.
    pub async fn course_of_condition(&self, condition: uuid::Uuid) -> Result<CourseId> {
        let row = sqlx::query(sql(format!(
            "with recursive up as (
                 select id, parent_id, course_module_id, section_id from {c} where id = $1
                 union all
                 select c.id, c.parent_id, c.course_module_id, c.section_id from {c} c join up on c.id = up.parent_id
             ) select course_module_id, section_id from up where parent_id is null",
            c = self.table("condition")
        )))
        .bind(condition)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("activities.condition_not_found"))?;
        if let Some(module) = row.get::<Option<uuid::Uuid>, _>("course_module_id") {
            return self.course_of_module(crate::ModuleId::from_uuid(module)).await;
        }
        let section: uuid::Uuid = row.get("section_id");
        Ok(self
            .courses
            .section(kampus_courses::SectionId::from_uuid(section))
            .await?
            .course)
    }
}
