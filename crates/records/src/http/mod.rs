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

//! The HTTP routes of records: the tree of a course, its categories and records, the grades people get,
//! overrides and locks, and the reads a learner and a grader make.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_access::{PrincipalId, ScopeId};
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::{CourseId, Place};
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::ServiceContext;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::permissions::{GRADE, HIDE, LOCK, MANAGE, OVERRIDE};
use crate::{CategoryFields, CategoryId, GradeWrite, RecordFields, RecordId, Records, Source};

/// The routes of records.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        records: context.service::<Records>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .route("/courses/{id}/records", get(tree))
        .route(
            "/courses/{id}/records/categories",
            axum::routing::post(add_category),
        )
        .route(
            "/record-categories/{id}",
            put(update_category).delete(delete_category),
        )
        .route("/courses/{id}/records/items", axum::routing::post(add_record))
        .route(
            "/records/{id}",
            get(read_record).put(update_record).delete(delete_record),
        )
        .route("/records/{id}/lock", put(lock_record).delete(unlock_record))
        .route("/records/{id}/grades", put(set_grades))
        .route("/records/{id}/grades/{account}", get(grade))
        .route(
            "/records/{id}/grades/{account}/override",
            put(override_grade).delete(clear_override),
        )
        .route(
            "/records/{id}/grades/{account}/lock",
            put(lock_grade).delete(unlock_grade),
        )
        .route(
            "/records/{id}/grades/{account}/hidden",
            put(hide_grade).delete(show_grade),
        )
        .route("/courses/{id}/grader-report", get(report))
        .route("/courses/{id}/my-grades", get(my_grades))
        .route("/courses/{id}/grades/{account}", get(grades_of))
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    records: Option<Records>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn records(&self) -> Result<&Records> {
        self.records
            .as_ref()
            .ok_or_else(|| Error::internal("records.service_missing"))
    }

    /// The actor of an operation that needs a permission in a scope.
    async fn acting<'c>(
        &self,
        caller: &'c Caller,
        permission: &str,
        scope: ScopeId,
    ) -> Result<&'c kampus_access::Actor> {
        let identity = self
            .identity
            .as_ref()
            .ok_or_else(|| Error::internal("records.identity_missing"))?;
        caller.acting(identity, permission, scope).await
    }
}

/// A category to add.
#[derive(Debug, Deserialize)]
struct NewCategory {
    /// The category it goes under; the root of the course when left out.
    #[serde(default)]
    parent: Option<CategoryId>,
    /// Where among its siblings.
    #[serde(default)]
    place: Place<CategoryId>,
    /// Its fields.
    #[serde(flatten)]
    fields: CategoryFields,
}

/// A grade record to add.
#[derive(Debug, Deserialize)]
struct NewRecord {
    /// The category it goes in; the root of the course when left out.
    #[serde(default)]
    category: Option<CategoryId>,
    /// Its fields.
    #[serde(flatten)]
    fields: RecordFields,
}

/// Grades to set.
#[derive(Debug, Deserialize)]
struct Grades {
    /// The grades.
    grades: Vec<GradeWrite>,
    /// Where they come from; those a teacher sets unless an import says otherwise.
    #[serde(default)]
    source: Option<Source>,
}

/// An override.
#[derive(Debug, Deserialize)]
struct Override {
    /// The grade that counts.
    value: Decimal,
    /// What the grader says about it.
    #[serde(default)]
    feedback: Option<String>,
}

/// A date a grade is hidden until.
#[derive(Debug, Deserialize)]
struct Hidden {
    /// The date.
    until: Timestamp,
}

/// A page of a report.
#[derive(Debug, Deserialize)]
struct Page {
    /// The most people.
    #[serde(default = "fifty")]
    limit: i64,
    /// The last person of the page before.
    #[serde(default)]
    after: Option<PrincipalId>,
}

/// The default size of a page.
const fn fifty() -> i64 {
    50
}

/// The records of a course.
async fn tree(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.records()?.tree(&caller.actor, id(&course)?).await?).into_response())
}

/// Adds a category.
async fn add_category(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<NewCategory>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, MANAGE, records.course_scope(course).await?)
        .await?;
    let created = records
        .define_category(actor, course, body.parent, body.fields, body.place)
        .await?;
    let version = created.version;
    Ok(with_etag((StatusCode::CREATED, Json(created)), version))
}

/// Replaces a category on the version it was read at.
async fn update_category(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<CategoryFields>,
) -> ApiResult<Response> {
    let category: CategoryId = id(&category)?;
    let records = shared.records()?;
    let expected = Some(if_match(&headers)?);
    let actor = shared
        .acting(&caller, MANAGE, records.category_scope(category).await?)
        .await?;
    let updated = records.update_category(actor, category, fields, expected).await?;
    let version = updated.version;
    Ok(with_etag(Json(updated), version))
}

/// Deletes an empty category.
async fn delete_category(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
) -> ApiResult<StatusCode> {
    let category: CategoryId = id(&category)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, MANAGE, records.category_scope(category).await?)
        .await?;
    records.delete_category(actor, category).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Adds a manual grade record.
async fn add_record(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<NewRecord>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, MANAGE, records.course_scope(course).await?)
        .await?;
    let created = records
        .define_record(actor, course, body.category, body.fields)
        .await?;
    let version = created.version;
    Ok(with_etag((StatusCode::CREATED, Json(created)), version))
}

/// A grade record.
async fn read_record(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(record): Path<String>,
) -> ApiResult<Response> {
    let found = shared.records()?.record(&caller.actor, id(&record)?).await?;
    let version = found.version;
    Ok(with_etag(Json(found), version))
}

/// Replaces the settings of a grade record on the version it was read at.
async fn update_record(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(record): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<RecordFields>,
) -> ApiResult<Response> {
    let record: RecordId = id(&record)?;
    let records = shared.records()?;
    let expected = Some(if_match(&headers)?);
    let actor = shared
        .acting(&caller, MANAGE, records.record_scope(record).await?)
        .await?;
    let updated = records.update_record(actor, record, fields, expected).await?;
    let version = updated.version;
    Ok(with_etag(Json(updated), version))
}

/// Deletes a manual grade record.
async fn delete_record(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(record): Path<String>,
) -> ApiResult<StatusCode> {
    let record: RecordId = id(&record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, MANAGE, records.record_scope(record).await?)
        .await?;
    records.delete_record(actor, record).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Locks a grade record.
async fn lock_record(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(record): Path<String>,
) -> ApiResult<Response> {
    locking(&shared, &caller, &record, true).await
}

/// Unlocks a grade record.
async fn unlock_record(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(record): Path<String>,
) -> ApiResult<Response> {
    locking(&shared, &caller, &record, false).await
}

/// Locks or unlocks a grade record.
async fn locking(shared: &Shared, caller: &Caller, record: &str, locked: bool) -> ApiResult<Response> {
    let record: RecordId = id(record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(caller, LOCK, records.record_scope(record).await?)
        .await?;
    Ok(Json(records.set_record_locked(actor, record, locked).await?).into_response())
}

/// Sets the grades of people on a record.
async fn set_grades(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(record): Path<String>,
    Body(body): Body<Grades>,
) -> ApiResult<Response> {
    let record: RecordId = id(&record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, GRADE, records.record_scope(record).await?)
        .await?;
    let changes = records
        .set_grades(actor, record, body.grades, body.source.unwrap_or(Source::Teacher))
        .await?;
    Ok(Json(changes).into_response())
}

/// One grade with its entries.
async fn grade(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    let detail = shared
        .records()?
        .grade_detail(&caller.actor, id(&record)?, id(&account)?)
        .await?;
    Ok(Json(detail).into_response())
}

/// Overrides a grade.
async fn override_grade(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
    Body(body): Body<Override>,
) -> ApiResult<Response> {
    let record: RecordId = id(&record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, OVERRIDE, records.record_scope(record).await?)
        .await?;
    let change = records
        .override_grade(actor, record, id(&account)?, body.value, body.feedback.as_deref())
        .await?;
    Ok(Json(change).into_response())
}

/// Removes the override of a grade.
async fn clear_override(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    let record: RecordId = id(&record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(&caller, OVERRIDE, records.record_scope(record).await?)
        .await?;
    let change = records.clear_override(actor, record, id(&account)?).await?;
    Ok(Json(change).into_response())
}

/// Locks a grade.
async fn lock_grade(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    locking_grade(&shared, &caller, &record, &account, true).await
}

/// Unlocks a grade.
async fn unlock_grade(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    locking_grade(&shared, &caller, &record, &account, false).await
}

/// Locks or unlocks a grade.
async fn locking_grade(
    shared: &Shared,
    caller: &Caller,
    record: &str,
    account: &str,
    locked: bool,
) -> ApiResult<Response> {
    let record: RecordId = id(record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(caller, LOCK, records.record_scope(record).await?)
        .await?;
    let found = records
        .set_grade_locked(actor, record, id(account)?, locked)
        .await?;
    Ok(Json(found).into_response())
}

/// Hides a grade until a date.
async fn hide_grade(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
    Body(body): Body<Hidden>,
) -> ApiResult<Response> {
    hiding(&shared, &caller, &record, &account, Some(body.until)).await
}

/// Shows a hidden grade.
async fn show_grade(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((record, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    hiding(&shared, &caller, &record, &account, None).await
}

/// Hides a grade until a date, or shows it.
async fn hiding(
    shared: &Shared,
    caller: &Caller,
    record: &str,
    account: &str,
    until: Option<Timestamp>,
) -> ApiResult<Response> {
    let record: RecordId = id(record)?;
    let records = shared.records()?;
    let actor = shared
        .acting(caller, HIDE, records.record_scope(record).await?)
        .await?;
    let found = records
        .set_grade_hidden(actor, record, id(account)?, until)
        .await?;
    Ok(Json(found).into_response())
}

/// A page of the grader report.
async fn report(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Query(page): Query<Page>,
) -> ApiResult<Response> {
    let found = shared
        .records()?
        .grader_report(&caller.actor, id(&course)?, page.limit, page.after)
        .await?;
    Ok(Json(found).into_response())
}

/// The grades of the caller in a course.
async fn my_grades(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let account = match &caller.actor {
        kampus_access::Actor::Principal(p) | kampus_access::Actor::ViewAs(p, _) => p.to_owned(),
        kampus_access::Actor::Visitor(_) => return Err(Error::permission_denied("access.denied").into()),
    };
    let found = shared
        .records()?
        .grades_for(&caller.actor, id(&course)?, account)
        .await?;
    Ok(Json(found).into_response())
}

/// The grades of a person in a course, as the caller may see them.
async fn grades_of(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((course, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    let found = shared
        .records()?
        .grades_for(&caller.actor, id(&course)?, id(&account)?)
        .await?;
    Ok(Json(found).into_response())
}
