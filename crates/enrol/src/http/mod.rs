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

//! The HTTP routes of enrolment: the method instances of a course, its participants, enrolling people
//! and oneself, and the requests that wait for a decision.

mod references;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, Result};
use kampus_courses::{CourseId, Courses, GroupId};
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;

use crate::permissions::{ENROLMENT_MANAGE, ENROLMENT_READ, INSTANCE_MANAGE, REQUEST_DECIDE, SELF};
use crate::{Enrol, EnrolmentChange, InstanceFields, InstanceId, Joined, Method, Period};

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[ENROL, references::OPERATIONS];

/// The operations of instances, enrolments and requests.
const ENROL: &[Operation] = &[
    Operation::get(
        "/courses/{id}/enrolment-instances",
        "The enrolment instances of a course",
    )
    .returns::<Vec<crate::Instance>>(),
    Operation::post(
        "/courses/{id}/enrolment-instances",
        "Add an enrolment instance to a course",
    )
    .body::<NewInstance>()
    .creates::<crate::Instance>(),
    Operation::get(
        "/enrolment-instances/{id}",
        "An enrolment instance, with its version as ETag",
    )
    .returns::<crate::Instance>(),
    Operation::put(
        "/enrolment-instances/{id}",
        "Replace the settings of an enrolment instance",
    )
    .body::<crate::InstanceFields>()
    .returns::<crate::Instance>(),
    Operation::delete(
        "/enrolment-instances/{id}",
        "Delete an enrolment instance with its enrolments",
    )
    .no_content(),
    Operation::put(
        "/enrolment-instances/{id}/keys",
        "Set or clear the enrolment key of a self instance",
    )
    .body::<KeyBody>()
    .no_content(),
    Operation::post(
        "/enrolment-instances/{id}/sync",
        "Synchronise an instance with its source now",
    )
    .returns::<crate::Synced>(),
    Operation::post(
        "/enrolment-instances/{id}/enrolments",
        "Enrol people by hand, with an answer per person",
    )
    .body::<EnrolBody>()
    .returns::<Vec<Enrolled>>(),
    Operation::post(
        "/enrolment-instances/{id}/self",
        "Enrol the caller; 202 when it becomes a request",
    )
    .body::<SelfBody>()
    .creates::<crate::Joined>(),
    Operation::get(
        "/courses/{id}/participants",
        "The enrolments of a course (query `active`)",
    )
    .returns::<Vec<crate::Enrolment>>(),
    Operation::get("/enrolments/{id}", "An enrolment, with its version as ETag")
        .returns::<crate::Enrolment>(),
    Operation::put("/enrolments/{id}", "Change the period or status of an enrolment")
        .body::<crate::EnrolmentChange>()
        .returns::<crate::Enrolment>(),
    Operation::delete("/enrolments/{id}", "Remove an enrolment").no_content(),
    Operation::get(
        "/courses/{id}/enrolment-requests",
        "The pending enrolment requests of a course",
    )
    .returns::<Vec<crate::Request>>(),
    Operation::put(
        "/enrolment-requests/{id}/decision",
        "Approve or reject an enrolment request",
    )
    .body::<Decision>()
    .returns::<crate::Request>(),
    Operation::delete(
        "/enrolment-requests/{id}",
        "Withdraw the caller's own enrolment request",
    )
    .no_content(),
];

/// The answer for one person enrolled by hand: the enrolment, or the code of why it failed.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct Enrolled {
    /// The person.
    account: PrincipalId,
    /// The enrolment, when it was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    enrolment: Option<crate::EnrolmentId>,
    /// The code of the problem, when it failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// The routes of enrolment.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        enrol: context.service::<Enrol>().ok(),
        courses: context.service::<Courses>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .route(
            "/courses/{id}/enrolment-instances",
            get(instances).post(create_instance),
        )
        .route(
            "/enrolment-instances/{id}",
            get(instance).put(update_instance).delete(delete_instance),
        )
        .route("/enrolment-instances/{id}/keys", put(set_key))
        .route("/enrolment-instances/{id}/sync", post(sync))
        .route("/enrolment-instances/{id}/enrolments", post(enrol))
        .route("/enrolment-instances/{id}/self", post(self_enrol))
        .route("/courses/{id}/participants", get(participants))
        .route(
            "/enrolments/{id}",
            get(enrolment).put(update_enrolment).delete(remove_enrolment),
        )
        .route("/courses/{id}/enrolment-requests", get(requests))
        .route("/enrolment-requests/{id}/decision", put(decide))
        .route("/enrolment-requests/{id}", axum::routing::delete(withdraw))
        .merge(references::routes())
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    enrol: Option<Enrol>,
    /// Courses, for the scope of a course.
    courses: Option<Courses>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn enrol(&self) -> Result<&Enrol> {
        self.enrol
            .as_ref()
            .ok_or_else(|| Error::internal("enrol.service_missing"))
    }

    /// The actor of an operation that needs a permission in a course.
    async fn acting<'c>(&self, caller: &'c Caller, permission: &str, course: CourseId) -> Result<&'c Actor> {
        let courses = self
            .courses
            .as_ref()
            .ok_or_else(|| Error::internal("enrol.courses_missing"))?;
        let identity = self
            .identity
            .as_ref()
            .ok_or_else(|| Error::internal("enrol.identity_missing"))?;
        let scope: ScopeId = courses.course(course).await?.scope;
        caller.acting(identity, permission, scope).await
    }

    /// The course of an instance.
    async fn course_of(&self, instance: InstanceId) -> Result<CourseId> {
        Ok(self.enrol()?.instance(instance).await?.course)
    }
}

/// A new instance.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewInstance {
    /// Its method.
    method: Method,
    /// Its fields.
    #[serde(flatten)]
    fields: InstanceFields,
    /// The key of a self instance.
    #[serde(default)]
    key: Option<String>,
    /// The system group a system group instance follows.
    #[serde(default)]
    source: Option<kampus_users::GroupId>,
}

/// A key to set or remove.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct KeyBody {
    /// The group the key puts people in; the whole course when absent.
    #[serde(default)]
    group: Option<GroupId>,
    /// The key; removed when absent.
    #[serde(default)]
    key: Option<String>,
}

/// People to enrol.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EnrolBody {
    /// Their accounts.
    accounts: Vec<PrincipalId>,
    /// Their period.
    #[serde(flatten)]
    period: Period,
}

/// Enrolling oneself.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SelfBody {
    /// The key, when the instance has one.
    #[serde(default)]
    key: Option<String>,
    /// A message for whoever approves.
    #[serde(default)]
    message: Option<String>,
}

/// A decision.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Decision {
    /// Whether the request is approved.
    approve: bool,
    /// Why.
    #[serde(default)]
    reason: Option<String>,
}

/// Which participants to list.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Filter {
    /// Only those who take part now.
    #[serde(default)]
    active: bool,
}

/// The instances of a course.
async fn instances(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = shared.acting(&caller, INSTANCE_MANAGE, course).await?;
    Ok(Json(shared.enrol()?.instances(actor, course).await?).into_response())
}

/// Adds an instance to a course.
async fn create_instance(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<NewInstance>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = shared.acting(&caller, INSTANCE_MANAGE, course).await?;
    let instance = shared
        .enrol()?
        .create_instance(
            actor,
            course,
            body.method,
            body.fields,
            body.key.as_deref(),
            body.source,
        )
        .await?;
    let version = instance.version;
    Ok(with_etag((StatusCode::CREATED, Json(instance)), version))
}

/// An instance.
async fn instance(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
) -> ApiResult<Response> {
    let instance: InstanceId = id(&instance)?;
    let course = shared.course_of(instance).await?;
    let actor = shared.acting(&caller, INSTANCE_MANAGE, course).await?;
    let stored = shared
        .enrol()?
        .instances(actor, course)
        .await?
        .into_iter()
        .find(|i| i.id == instance)
        .ok_or_else(|| Error::not_found("enrol.instance_not_found"))?;
    let version = stored.version;
    Ok(with_etag(Json(stored), version))
}

/// Replaces the fields of an instance.
async fn update_instance(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<InstanceFields>,
) -> ApiResult<Response> {
    let instance: InstanceId = id(&instance)?;
    let expected = Some(if_match(&headers)?);
    let actor = shared
        .acting(&caller, INSTANCE_MANAGE, shared.course_of(instance).await?)
        .await?;
    let updated = shared
        .enrol()?
        .update_instance(actor, instance, fields, expected)
        .await?;
    let version = updated.version;
    Ok(with_etag(Json(updated), version))
}

/// Deletes an instance with its enrolments.
async fn delete_instance(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let instance: InstanceId = id(&instance)?;
    let expected = Some(if_match(&headers)?);
    let actor = shared
        .acting(&caller, INSTANCE_MANAGE, shared.course_of(instance).await?)
        .await?;
    shared.enrol()?.delete_instance(actor, instance, expected).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Sets or removes a key.
async fn set_key(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
    Body(body): Body<KeyBody>,
) -> ApiResult<StatusCode> {
    let instance: InstanceId = id(&instance)?;
    let actor = shared
        .acting(&caller, INSTANCE_MANAGE, shared.course_of(instance).await?)
        .await?;
    shared
        .enrol()?
        .set_key(actor, instance, body.group, body.key.as_deref())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Syncs a system group instance.
async fn sync(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
) -> ApiResult<Response> {
    let instance: InstanceId = id(&instance)?;
    let actor = shared
        .acting(&caller, INSTANCE_MANAGE, shared.course_of(instance).await?)
        .await?;
    Ok(Json(shared.enrol()?.sync_instance(actor, instance).await?).into_response())
}

/// Enrols people through a manual instance, with an answer per person.
async fn enrol(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
    Body(body): Body<EnrolBody>,
) -> ApiResult<Response> {
    let instance: InstanceId = id(&instance)?;
    let actor = shared
        .acting(&caller, ENROLMENT_MANAGE, shared.course_of(instance).await?)
        .await?;
    let report = shared
        .enrol()?
        .enrol(actor, instance, &body.accounts, body.period)
        .await?;
    let body: Vec<_> = report
        .into_iter()
        .map(|(account, result)| match result {
            Ok(enrolment) => Enrolled {
                account,
                enrolment: Some(enrolment),
                error: None,
            },
            Err(error) => Enrolled {
                account,
                enrolment: None,
                error: Some(error.code().to_owned()),
            },
        })
        .collect();
    Ok(Json(body).into_response())
}

/// Enrols the caller.
async fn self_enrol(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(instance): Path<String>,
    Body(body): Body<SelfBody>,
) -> ApiResult<Response> {
    let instance: InstanceId = id(&instance)?;
    let actor = shared
        .acting(&caller, SELF, shared.course_of(instance).await?)
        .await?;
    let joined = shared
        .enrol()?
        .self_enrol(
            actor,
            instance,
            body.key.as_deref(),
            body.message.as_deref(),
            caller.client.address.as_deref(),
        )
        .await?;
    let status = match joined {
        Joined::Enrolled { .. } => StatusCode::CREATED,
        Joined::Requested { .. } => StatusCode::ACCEPTED,
    };
    Ok((status, Json(joined)).into_response())
}

/// The participants of a course.
async fn participants(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Query(filter): Query<Filter>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = shared.acting(&caller, ENROLMENT_READ, course).await?;
    Ok(Json(shared.enrol()?.participants(actor, course, filter.active).await?).into_response())
}

/// An enrolment, for its owner or who reads the participants of its course.
async fn enrolment(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(enrolment): Path<String>,
) -> ApiResult<Response> {
    let stored = shared.enrol()?.enrolment(id(&enrolment)?).await?;
    if caller.account().ok() != Some(stored.account) {
        let actor = shared.acting(&caller, ENROLMENT_READ, stored.course).await?;
        shared.enrol()?.participants(actor, stored.course, false).await?;
    }
    let version = stored.version;
    Ok(with_etag(Json(stored), version))
}

/// Changes an enrolment.
async fn update_enrolment(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(enrolment): Path<String>,
    headers: HeaderMap,
    Body(change): Body<EnrolmentChange>,
) -> ApiResult<Response> {
    let stored = shared.enrol()?.enrolment(id(&enrolment)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, ENROLMENT_MANAGE, stored.course).await?;
    let updated = shared
        .enrol()?
        .update_enrolment(actor, stored.id, change, expected)
        .await?;
    let version = updated.version;
    Ok(with_etag(Json(updated), version))
}

/// Removes an enrolment.
async fn remove_enrolment(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(enrolment): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let stored = shared.enrol()?.enrolment(id(&enrolment)?).await?;
    let expected = Some(if_match(&headers)?);
    let actor = shared.acting(&caller, ENROLMENT_MANAGE, stored.course).await?;
    shared
        .enrol()?
        .remove_enrolment(actor, stored.id, expected)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The requests of a course that wait.
async fn requests(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let actor = shared.acting(&caller, REQUEST_DECIDE, course).await?;
    Ok(Json(shared.enrol()?.pending_requests(actor, course).await?).into_response())
}

/// Decides a request.
async fn decide(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(request): Path<String>,
    Body(body): Body<Decision>,
) -> ApiResult<Response> {
    let enrol = shared.enrol()?;
    let request = enrol.request(id(&request)?).await?;
    let course = shared.course_of(request.instance).await?;
    let actor = shared.acting(&caller, REQUEST_DECIDE, course).await?;
    let decided = enrol
        .decide_request(actor, request.id, body.approve, body.reason.as_deref())
        .await?;
    Ok(Json(decided).into_response())
}

/// Withdraws the caller's own request.
async fn withdraw(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(request): Path<String>,
) -> ApiResult<StatusCode> {
    shared
        .enrol()?
        .withdraw_request(&caller.actor, id(&request)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
