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

//! The routes of the external references of enrolments: the identifiers systems give an enrolment, an
//! enrolment found by one, and a person enrolled by one.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use kampus_access::{PrincipalId, ScopeId};
use kampus_db::references::ExternalReference;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::Shared;
use crate::permissions::{ENROLMENT_MANAGE, ENROLMENT_READ};
use crate::{EnrolmentId, InstanceId, Period};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/enrolments/{id}/references", get(list))
        .route("/enrolments/{id}/references/{system}", put(set).delete(remove))
        .route(
            "/campuses/{campus}/enrolments/by-reference/{system}/{external_id}",
            get(find),
        )
        .route(
            "/enrolment-instances/{id}/enrolments/by-reference/{system}/{external_id}",
            put(upsert),
        )
}

/// The identifier a system gives an enrolment.
#[derive(Debug, Deserialize)]
struct ExternalId {
    /// The identifier.
    external_id: String,
}

/// A person to enrol, or whose enrolment changes, by its identifier.
#[derive(Debug, Deserialize)]
struct EnrolmentByReference {
    /// The account.
    account: PrincipalId,
    /// The period.
    #[serde(flatten)]
    period: Period,
}

/// The identifiers systems give an enrolment.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(enrolment): Path<String>,
) -> ApiResult<Response> {
    let enrolment: EnrolmentId = id(&enrolment)?;
    let course = shared.enrol()?.enrolment(enrolment).await?.course;
    let actor = shared.acting(&caller, ENROLMENT_READ, course).await?;
    let references = shared
        .enrol()?
        .enrolment_external_references(actor, enrolment)
        .await?;
    Ok(Json(references).into_response())
}

/// Gives an enrolment the identifier a system uses for it.
async fn set(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((enrolment, system)): Path<(String, String)>,
    Body(body): Body<ExternalId>,
) -> ApiResult<Response> {
    let enrolment: EnrolmentId = id(&enrolment)?;
    let reference = ExternalReference::new(&system, &body.external_id)?;
    let course = shared.enrol()?.enrolment(enrolment).await?.course;
    let actor = shared.acting(&caller, ENROLMENT_MANAGE, course).await?;
    let references = shared
        .enrol()?
        .set_enrolment_reference(actor, enrolment, &reference)
        .await?;
    Ok(Json(references).into_response())
}

/// Removes the identifier a system gave an enrolment.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((enrolment, system)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let enrolment: EnrolmentId = id(&enrolment)?;
    let course = shared.enrol()?.enrolment(enrolment).await?.course;
    let actor = shared.acting(&caller, ENROLMENT_MANAGE, course).await?;
    shared
        .enrol()?
        .remove_enrolment_reference(actor, enrolment, &system)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The enrolment a system knows by an identifier in a campus.
async fn find(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((campus, system, external_id)): Path<(String, String, String)>,
) -> ApiResult<Response> {
    let campus: ScopeId = id(&campus)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let enrol = shared.enrol()?;
    let found = enrol
        .enrolment_by_reference(campus, &reference)
        .await?
        .ok_or_else(|| kampus_common::Error::not_found("references.not_found"))?;
    let enrolment = enrol.enrolment(found).await?;
    let actor = shared.acting(&caller, ENROLMENT_READ, enrolment.course).await?;
    enrol.enrolment_external_references(actor, found).await?;
    Ok(Json(enrolment).into_response())
}

/// Enrols a person through a manual instance by an identifier, or changes the period of the enrolment
/// that has it.
async fn upsert(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((instance, system, external_id)): Path<(String, String, String)>,
    Body(body): Body<EnrolmentByReference>,
) -> ApiResult<Response> {
    let instance: InstanceId = id(&instance)?;
    let reference = ExternalReference::new(&system, &external_id)?;
    let actor = shared
        .acting(&caller, ENROLMENT_MANAGE, shared.course_of(instance).await?)
        .await?;
    let (enrolment, created) = shared
        .enrol()?
        .upsert_enrolment_by_reference(actor, instance, body.account, body.period, &reference)
        .await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(enrolment)).into_response())
}
