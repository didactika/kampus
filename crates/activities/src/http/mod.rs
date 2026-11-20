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

//! The HTTP routes of activities: the types, the course modules of a section and of one course module,
//! their files, the recycle bin and the external references of course modules.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::{Error, Result};
use kampus_courses::{CourseId, Place, SectionId};
use kampus_db::references::ExternalReference;
use kampus_files::FileId;
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::ServiceContext;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};
use serde::Deserialize;
use serde_json::json;

use crate::permissions::{EXCEPTION_MANAGE, MODULE_MANAGE};
use crate::{Activities, ExceptionId, ModuleFields, ModuleId, Target, Values};

/// The routes of activities.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        activities: context.service::<Activities>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .route("/types", get(types))
        .route("/sections/{id}/course-modules", get(list).post(create))
        .route("/course-modules/{id}", get(read).put(update).delete(remove))
        .route("/course-modules/{id}/move", post(move_to))
        .route("/course-modules/{id}/copy", post(copy))
        .route("/course-modules/{id}/restore", post(restore))
        .route("/course-modules/{id}/file", get(file).put(set_file))
        .route("/courses/{id}/bin", get(bin))
        .route("/course-modules/{id}/exceptions", get(exceptions))
        .route(
            "/course-modules/{id}/exceptions/accounts/{account}",
            put(set_person_exception),
        )
        .route(
            "/course-modules/{id}/exceptions/groups/{group}",
            put(set_group_exception),
        )
        .route(
            "/course-modules/{id}/effective-values/{account}",
            get(effective_values),
        )
        .route("/exceptions/{id}", axum::routing::delete(remove_exception))
        .route("/course-modules/{id}/references", get(references))
        .route(
            "/course-modules/{id}/references/{system}",
            put(set_reference).delete(remove_reference),
        )
        .route(
            "/campuses/{campus}/course-modules/by-reference/{system}/{external_id}",
            get(by_reference),
        )
        .route(
            "/sections/{id}/course-modules/by-reference/{system}/{external_id}",
            put(upsert),
        )
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    activities: Option<Activities>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn activities(&self) -> Result<&Activities> {
        self.activities
            .as_ref()
            .ok_or_else(|| Error::internal("activities.service_missing"))
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
            .ok_or_else(|| Error::internal("activities.identity_missing"))?;
        caller.acting(identity, permission, scope).await
    }
}

/// A course module to add.
#[derive(Debug, Deserialize)]
struct NewModule {
    /// Its type.
    #[serde(rename = "type")]
    type_code: String,
    /// Where among the course modules of the section.
    #[serde(default)]
    place: Place<ModuleId>,
    /// Its fields.
    #[serde(flatten)]
    fields: ModuleFields,
}

/// Where a course module goes.
#[derive(Debug, Deserialize)]
struct Destination {
    /// The section.
    section: SectionId,
    /// Where among its course modules.
    #[serde(default)]
    place: Place<ModuleId>,
}

/// The file to make the file of a material.
#[derive(Debug, Deserialize)]
struct FileBody {
    /// A file uploaded into `activities.material`.
    file: FileId,
}

/// The identifier a system gives a course module.
#[derive(Debug, Deserialize)]
struct ExternalId {
    /// The identifier.
    external_id: String,
}

/// The types of activity and material.
async fn types(State(shared): State<Shared>) -> ApiResult<Response> {
    Ok(Json(shared.activities()?.types().await?).into_response())
}

/// The course modules of a section the caller sees.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
) -> ApiResult<Response> {
    let modules = shared.activities()?.modules(&caller.actor, id(&section)?).await?;
    Ok(Json(modules).into_response())
}

/// Adds a course module to a section.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(section): Path<String>,
    Body(body): Body<NewModule>,
) -> ApiResult<Response> {
    let section: SectionId = id(&section)?;
    let activities = shared.activities()?;
    let scope = activities.section_scope(section).await?;
    let actor = shared.acting(&caller, MODULE_MANAGE, scope).await?;
    let created = activities
        .create_module(actor, section, &body.type_code, body.fields, body.place)
        .await?;
    let version = created.version;
    Ok(with_etag((StatusCode::CREATED, Json(created)), version))
}

/// A course module the caller sees.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared.activities()?.module(&caller.actor, id(&module)?).await?;
    let version = found.version;
    Ok(with_etag(Json(found), version))
}

/// Replaces a course module, on the version it was read at.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<ModuleFields>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let expected = Some(if_match(&headers)?);
    let actor = shared
        .acting(&caller, MODULE_MANAGE, activities.scope_of(module).await?)
        .await?;
    let updated = activities.update_module(actor, module, fields, expected).await?;
    let version = updated.version;
    Ok(with_etag(Json(updated), version))
}

/// Deletes a course module to the recycle bin.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<StatusCode> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let actor = shared
        .acting(&caller, MODULE_MANAGE, activities.scope_of(module).await?)
        .await?;
    activities.delete_module(actor, module).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Moves a course module.
async fn move_to(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
    Body(to): Body<Destination>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let actor = shared
        .acting(&caller, MODULE_MANAGE, activities.scope_of(module).await?)
        .await?;
    let moved = activities
        .move_module(actor, module, to.section, to.place)
        .await?;
    let version = moved.version;
    Ok(with_etag(Json(moved), version))
}

/// Copies a course module.
async fn copy(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
    Body(to): Body<Destination>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let scope = activities.section_scope(to.section).await?;
    let actor = shared.acting(&caller, MODULE_MANAGE, scope).await?;
    let copied = activities
        .copy_module(actor, module, to.section, to.place)
        .await?;
    Ok((StatusCode::CREATED, Json(copied)).into_response())
}

/// Brings a course module back from the recycle bin.
async fn restore(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let actor = shared
        .acting(&caller, MODULE_MANAGE, activities.scope_of(module).await?)
        .await?;
    Ok(Json(activities.restore_module(actor, module).await?).into_response())
}

/// The file of a material.
async fn file(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared
        .activities()?
        .module_file(&caller.actor, id(&module)?)
        .await?
        .ok_or_else(|| Error::not_found("activities.file_not_found"))?;
    Ok(Json(json!({ "file": found })).into_response())
}

/// Makes an uploaded file the file of a material.
async fn set_file(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
    Body(body): Body<FileBody>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let actor = shared
        .acting(&caller, MODULE_MANAGE, activities.scope_of(module).await?)
        .await?;
    let file = activities.set_module_file(actor, module, body.file).await?;
    Ok(Json(json!({ "file": file })).into_response())
}

/// The recycle bin of a course.
async fn bin(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(course): Path<String>,
) -> ApiResult<Response> {
    let course: CourseId = id(&course)?;
    let items = shared.activities()?.bin(&caller.actor, course).await?;
    Ok(Json(items).into_response())
}

/// The identifiers systems give a course module.
async fn references(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let found = shared
        .activities()?
        .module_external_references(&caller.actor, id(&module)?)
        .await?;
    Ok(Json(found).into_response())
}

/// Gives a course module the identifier a system uses for it.
async fn set_reference(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, system)): Path<(String, String)>,
    Body(body): Body<ExternalId>,
) -> ApiResult<Response> {
    let reference = ExternalReference::new(&system, &body.external_id)?;
    let found = shared
        .activities()?
        .set_module_reference(&caller.actor, id(&module)?, &reference)
        .await?;
    Ok(Json(found).into_response())
}

/// Removes the identifier a system gave a course module.
async fn remove_reference(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, system)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    shared
        .activities()?
        .remove_module_reference(&caller.actor, id(&module)?, &system)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The course module a system knows by an identifier in a campus.
async fn by_reference(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((campus, system, external_id)): Path<(String, String, String)>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let activities = shared.activities()?;
    let found = activities
        .module_by_reference(campus, &reference)
        .await?
        .ok_or_else(|| Error::not_found("references.not_found"))?;
    activities
        .module_external_references(&caller.actor, found)
        .await?;
    Ok(Json(activities.module(&caller.actor, found).await?).into_response())
}

/// Updates the course module a system knows by an identifier, or adds it to the section.
async fn upsert(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((section, system, external_id)): Path<(String, String, String)>,
    Body(body): Body<NewModule>,
) -> ApiResult<Response> {
    let section: SectionId = id(&section)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let activities = shared.activities()?;
    let scope = activities.section_scope(section).await?;
    let actor = shared.acting(&caller, MODULE_MANAGE, scope).await?;
    let (module, created) = activities
        .upsert_module_by_reference(actor, section, &body.type_code, &reference, body.fields)
        .await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(module)).into_response())
}

/// An exception to grant.
#[derive(Debug, Deserialize)]
struct ExceptionBody {
    /// What it changes.
    #[serde(flatten)]
    values: Values,
    /// Why it is granted.
    #[serde(default)]
    reason: Option<String>,
}

/// The exceptions of a course module.
async fn exceptions(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(module): Path<String>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let scope = activities.scope_of(module).await?;
    let actor = shared.acting(&caller, EXCEPTION_MANAGE, scope).await?;
    Ok(Json(activities.exceptions(actor, module).await?).into_response())
}

/// Grants an exception to a person.
async fn set_person_exception(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, account)): Path<(String, String)>,
    Body(body): Body<ExceptionBody>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let scope = activities.scope_of(module).await?;
    let actor = shared.acting(&caller, EXCEPTION_MANAGE, scope).await?;
    let target = Target::Person(id(&account)?);
    let granted = activities
        .set_exception(actor, module, target, body.values, body.reason.as_deref())
        .await?;
    Ok(Json(granted).into_response())
}

/// Grants an exception to a group.
async fn set_group_exception(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, group)): Path<(String, String)>,
    Body(body): Body<ExceptionBody>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let scope = activities.scope_of(module).await?;
    let actor = shared.acting(&caller, EXCEPTION_MANAGE, scope).await?;
    let target = Target::Group(id(&group)?);
    let granted = activities
        .set_exception(actor, module, target, body.values, body.reason.as_deref())
        .await?;
    Ok(Json(granted).into_response())
}

/// Removes an exception.
async fn remove_exception(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(exception): Path<String>,
) -> ApiResult<StatusCode> {
    let exception: ExceptionId = id(&exception)?;
    let activities = shared.activities()?;
    let scope = activities.exception_scope(exception).await?;
    let actor = shared.acting(&caller, EXCEPTION_MANAGE, scope).await?;
    activities.remove_exception(actor, exception).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// What applies to a person on a course module; a person asks for their own, a manager for anyone's.
async fn effective_values(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((module, account)): Path<(String, String)>,
) -> ApiResult<Response> {
    let module: ModuleId = id(&module)?;
    let activities = shared.activities()?;
    let values = activities
        .effective_values_for(&caller.actor, module, id(&account)?)
        .await?;
    Ok(Json(values).into_response())
}
