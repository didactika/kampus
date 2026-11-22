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

//! The HTTP routes of progress: the rules of a course module, what a person does about them, the states
//! people read, and the decisions teachers take.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_access::{Actor, PrincipalId};
use kampus_activities::ModuleId;
use kampus_common::{Error, Result};
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;

use crate::permissions::{EXCEPTION, MANAGE};
use crate::{Progress, RuleKind, State as ProgressState};

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[ACTIVITIES, COURSES];

/// The operations of the progress of course modules.
const ACTIVITIES: &[Operation] = &[
    Operation::get(
        "/course-modules/{id}/rules",
        "The kinds of rule a course module requires to be complete",
    )
    .returns::<Vec<crate::RuleKind>>(),
    Operation::put(
        "/course-modules/{id}/rules",
        "Replace the kinds of rule a course module requires",
    )
    .body::<Rules>()
    .returns::<Vec<crate::RuleKind>>(),
    Operation::get(
        "/course-modules/{id}/type-rules",
        "The rules of its type a course module requires, and those its type offers",
    )
    .returns::<TypeRulesAnswer>(),
    Operation::put(
        "/course-modules/{id}/type-rules",
        "Replace the rules of its type a course module requires",
    )
    .body::<TypeRules>()
    .returns::<Vec<String>>(),
    Operation::post(
        "/course-modules/{id}/views",
        "Record that the caller viewed a course module",
    )
    .returns::<crate::ActivityState>(),
    Operation::put("/course-modules/{id}/mark", "Mark a course module done by hand")
        .returns::<crate::ActivityState>(),
    Operation::delete("/course-modules/{id}/mark", "Take back a mark by hand")
        .returns::<crate::ActivityState>(),
    Operation::get(
        "/course-modules/{id}/states/{account}",
        "Where a person stands in a course module",
    )
    .returns::<crate::ActivityState>(),
    Operation::get(
        "/course-modules/{id}/exceptions",
        "The decisions teachers took on the progress of people",
    )
    .returns::<Vec<crate::Exception>>(),
    Operation::put(
        "/course-modules/{id}/exceptions/{account}",
        "Decide the state of a person in a course module",
    )
    .body::<Decision>()
    .returns::<crate::ActivityState>(),
    Operation::delete(
        "/course-modules/{id}/exceptions/{account}",
        "Clear the decision on a person",
    )
    .no_content(),
];

/// The operations of the progress of courses.
const COURSES: &[Operation] = &[
    Operation::get(
        "/courses/{id}/states/{account}",
        "Where a person stands in every course module of a course",
    )
    .returns::<Vec<crate::ActivityState>>(),
    Operation::get(
        "/courses/{id}/my-progress",
        "Where the caller stands in every course module of a course",
    )
    .returns::<Vec<crate::ActivityState>>(),
    Operation::get("/courses/{id}/criteria", "What completes a course").returns::<crate::CourseCriteria>(),
    Operation::put("/courses/{id}/criteria", "Replace what completes a course")
        .body::<crate::CourseCriteria>()
        .returns::<crate::CourseCriteria>(),
    Operation::get(
        "/courses/{id}/prerequisites",
        "The courses that must be complete before a course",
    )
    .returns::<crate::Prerequisites>(),
    Operation::put(
        "/courses/{id}/prerequisites",
        "Replace the prerequisites of a course",
    )
    .body::<PrerequisitesBody>()
    .returns::<crate::Prerequisites>(),
    Operation::get(
        "/courses/{id}/course-progress",
        "The progress of every participant in a course",
    )
    .returns::<Vec<crate::CourseProgress>>(),
    Operation::get(
        "/courses/{id}/course-progress/{account}",
        "The progress of a person in a course",
    )
    .returns::<crate::CourseProgress>(),
    Operation::get(
        "/courses/{id}/my-course-progress",
        "The progress of the caller in a course",
    )
    .returns::<crate::CourseProgress>(),
    Operation::put(
        "/criteria/{id}/approvals/{account}",
        "Approve a person on a criterion that needs a person",
    )
    .no_content(),
];

/// The rules of its type a course module requires, and those its type offers.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct TypeRulesAnswer {
    /// The codes of the rules it requires.
    required: Vec<String>,
    /// The rules its type offers.
    offered: Vec<OfferedRule>,
}

/// A rule the type of a course module offers.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct OfferedRule {
    /// Its code.
    code: &'static str,
    /// What it asks for.
    summary: &'static str,
}

/// The routes of progress.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        progress: context.service::<Progress>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .route("/course-modules/{id}/rules", get(rules).put(define_rules))
        .route(
            "/course-modules/{id}/type-rules",
            get(type_rules).put(define_type_rules),
        )
        .route("/course-modules/{id}/views", post(view))
        .route("/course-modules/{id}/mark", put(mark).delete(unmark))
        .route("/course-modules/{id}/states/{account}", get(state_of))
        .route("/course-modules/{id}/exceptions", get(exceptions))
        .route(
            "/course-modules/{id}/exceptions/{account}",
            put(set_exception).delete(clear_exception),
        )
        .route("/courses/{id}/states/{account}", get(states_for))
        .route("/courses/{id}/my-progress", get(my_progress))
        .route("/courses/{id}/criteria", get(criteria).put(define_criteria))
        .route(
            "/courses/{id}/prerequisites",
            get(prerequisites).put(set_prerequisites),
        )
        .route("/courses/{id}/course-progress", get(course_report))
        .route("/courses/{id}/course-progress/{account}", get(course_progress))
        .route("/courses/{id}/my-course-progress", get(my_course_progress))
        .route("/criteria/{id}/approvals/{account}", put(approve))
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    progress: Option<Progress>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn progress(&self) -> Result<&Progress> {
        self.progress
            .as_ref()
            .ok_or_else(|| Error::internal("progress.service_missing"))
    }

    /// The actor of an operation that needs a permission in a course module.
    async fn acting<'c>(&self, caller: &'c Caller, permission: &str, module: ModuleId) -> Result<&'c Actor> {
        let identity = self
            .identity
            .as_ref()
            .ok_or_else(|| Error::internal("progress.identity_missing"))?;
        let scope = self.progress()?.module_scope(module).await?;
        caller.acting(identity, permission, scope).await
    }
}

/// The rules to set.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Rules {
    /// What the course module requires.
    kinds: Vec<RuleKind>,
}

/// A decision about a person.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Decision {
    /// The state it sets.
    state: ProgressState,
    /// Why.
    reason: String,
}

/// The rules of a course module.
async fn rules(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared.progress()?.rules(&caller.actor, id(&module)?).await?;
    Ok(Json(found).into_response())
}

/// Replaces the rules of a course module.
async fn define_rules(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
    Body(body): Body<Rules>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let actor = shared.acting(&caller, MANAGE, module).await?;
    let set = shared.progress()?.define_rules(actor, module, body.kinds).await?;
    Ok(Json(set).into_response())
}

/// The caller opened a course module.
async fn view(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared.progress()?.view(&caller.actor, id(&module)?).await?;
    Ok(Json(found).into_response())
}

/// The caller marks a course module done.
async fn mark(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared.progress()?.mark(&caller.actor, id(&module)?, true).await?;
    Ok(Json(found).into_response())
}

/// The caller takes back the mark.
async fn unmark(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared
        .progress()?
        .mark(&caller.actor, id(&module)?, false)
        .await?;
    Ok(Json(found).into_response())
}

/// The state of a person on a course module.
async fn state_of(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    let found = shared
        .progress()?
        .state_of(&caller.actor, id(&module)?, id(&account)?)
        .await?;
    Ok(Json(found).into_response())
}

/// The decisions about a course module.
async fn exceptions(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared.progress()?.exceptions(&caller.actor, id(&module)?).await?;
    Ok(Json(found).into_response())
}

/// Decides the state of a person.
async fn set_exception(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, account)): Path<(String, String)>,
    Body(body): Body<Decision>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let actor = shared.acting(&caller, EXCEPTION, module).await?;
    let found = shared
        .progress()?
        .set_exception(actor, module, id(&account)?, body.state, &body.reason)
        .await?;
    Ok(Json(found).into_response())
}

/// Removes the decision about a person.
async fn clear_exception(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, account)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let module: ModuleId = id(&module)?;
    let actor = shared.acting(&caller, EXCEPTION, module).await?;
    shared
        .progress()?
        .clear_exception(actor, module, id(&account)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The states of a person in a course.
async fn states_for(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((course, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    let found = shared
        .progress()?
        .states_for(&caller.actor, id(&course)?, id(&account)?)
        .await?;
    Ok(Json(found).into_response())
}

/// The states of the caller in a course.
async fn my_progress(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let account: PrincipalId = match &caller.actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => p.to_owned(),
        Actor::Visitor(_) => return Err(Error::permission_denied("access.denied").into()),
    };
    let found = shared
        .progress()?
        .states_for(&caller.actor, id(&course)?, account)
        .await?;
    Ok(Json(found).into_response())
}

/// The person of the caller.
fn me(caller: &Caller) -> Result<PrincipalId> {
    match &caller.actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Ok(*p),
        Actor::Visitor(_) => Err(Error::permission_denied("access.denied")),
    }
}

/// What a course requires.
async fn criteria(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .progress()?
            .course_criteria(&caller.actor, id(&course)?)
            .await?,
    )
    .into_response())
}

/// Replaces what a course requires.
async fn define_criteria(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<crate::CourseCriteria>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .progress()?
            .define_criteria(&caller.actor, id(&course)?, body)
            .await?,
    )
    .into_response())
}

/// The progress of everyone a course follows.
async fn course_report(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .progress()?
            .course_report(&caller.actor, id(&course)?)
            .await?,
    )
    .into_response())
}

/// The progress of a person in a course.
async fn course_progress(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((course, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .progress()?
            .course_progress(&caller.actor, id(&course)?, id(&account)?)
            .await?,
    )
    .into_response())
}

/// The progress of the caller in a course.
async fn my_course_progress(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let account = me(&caller)?;
    Ok(Json(
        shared
            .progress()?
            .course_progress(&caller.actor, id(&course)?, account)
            .await?,
    )
    .into_response())
}

/// Approves a person for a criterion.
async fn approve(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((criterion, account)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let criterion =
        uuid::Uuid::parse_str(&criterion).map_err(|_| Error::not_found("progress.criterion_not_found"))?;
    shared
        .progress()?
        .approve(&caller.actor, criterion, id(&account)?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The courses a course requires first.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PrerequisitesBody {
    /// The courses.
    requires: Vec<kampus_courses::CourseId>,
}

/// The prerequisites around a course.
async fn prerequisites(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .progress()?
            .prerequisites(&caller.actor, id(&course)?)
            .await?,
    )
    .into_response())
}

/// Replaces the courses a course requires first.
async fn set_prerequisites(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
    Body(body): Body<PrerequisitesBody>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .progress()?
            .set_prerequisites(&caller.actor, id(&course)?, body.requires)
            .await?,
    )
    .into_response())
}

/// The rules of its type a course module requires.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypeRules {
    /// Their codes.
    rules: Vec<String>,
}

/// The rules of its type a course module requires, and those its type offers.
async fn type_rules(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let (required, offered) = shared.progress()?.type_rules(&caller.actor, id(&module)?).await?;
    let offered = offered
        .iter()
        .map(|rule| OfferedRule {
            code: rule.code,
            summary: rule.summary,
        })
        .collect();
    Ok(Json(TypeRulesAnswer { required, offered }).into_response())
}

/// Replaces the rules of its type a course module requires.
async fn define_type_rules(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
    Body(body): Body<TypeRules>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let actor = shared.acting(&caller, MANAGE, module).await?;
    Ok(Json(
        shared
            .progress()?
            .define_type_rules(actor, module, body.rules)
            .await?,
    )
    .into_response())
}
