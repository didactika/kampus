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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of privacy: asking for an export or a deletion, deciding on it, following the request and
//! reading the package, the legal holds, the retention of each kind of data, and the policies of the campus.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_access::PrincipalId;
use kampus_common::{Error, Result};
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;

use crate::Privacy;

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[PRIVACY];

/// The operations of requests, holds, retention and policies.
const PRIVACY: &[Operation] = &[
    Operation::get(
        "/requests",
        "The privacy requests of the caller, or of everyone for an officer",
    )
    .returns::<Vec<crate::Request>>(),
    Operation::post(
        "/requests",
        "Ask for an export or the deletion of personal data; it runs in the background",
    )
    .body::<NewRequest>()
    .answers::<crate::Request>(202),
    Operation::get("/requests/{id}", "A privacy request").returns::<crate::Request>(),
    Operation::get("/requests/awaiting", "The deletions that wait for an approval")
        .returns::<Vec<crate::Request>>(),
    Operation::get("/requests/{id}/package", "The package of a finished export")
        .returns::<serde_json::Value>(),
    Operation::post("/requests/{id}/decision", "Approve or reject a deletion")
        .body::<Decision>()
        .returns::<crate::Request>(),
    Operation::get("/holds", "The legal holds").returns::<Vec<crate::Hold>>(),
    Operation::post("/holds", "Place a legal hold that keeps data from deletion")
        .body::<NewHold>()
        .creates::<crate::Hold>(),
    Operation::post("/holds/{id}/release", "Release a legal hold").returns::<crate::Hold>(),
    Operation::get("/retention", "How long each category of personal data is kept")
        .returns::<Vec<crate::Retention>>(),
    Operation::put(
        "/retention/{category}",
        "Set how long a category of personal data is kept",
    )
    .body::<KeepDays>()
    .returns::<Vec<crate::Retention>>(),
    Operation::get("/policies", "The published policies").returns::<Vec<crate::Policy>>(),
    Operation::post("/policies", "Publish a version of a policy")
        .body::<NewPolicy>()
        .creates::<crate::Policy>(),
    Operation::get("/policies/pending", "The policies the caller has yet to accept")
        .returns::<Vec<crate::Policy>>(),
    Operation::post(
        "/policies/{code}/acceptance",
        "Accept the current version of a policy",
    )
    .body::<Acceptance>()
    .returns::<crate::Policy>(),
];

/// The routes of privacy.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        privacy: context.service::<Privacy>().ok(),
    };
    Router::new()
        .route("/requests", get(list).post(create))
        .route("/requests/{id}", get(read))
        .route("/requests/awaiting", get(awaiting))
        .route("/requests/{id}/package", get(package))
        .route("/requests/{id}/decision", post(decide))
        .route("/holds", get(holds).post(place))
        .route("/holds/{id}/release", post(release))
        .route("/retention", get(retention))
        .route("/retention/{category}", put(set_retention))
        .route("/policies", get(policies).post(publish))
        .route("/policies/pending", get(pending))
        .route("/policies/{code}/acceptance", post(accept))
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    privacy: Option<Privacy>,
}

impl Shared {
    /// The service.
    fn privacy(&self) -> Result<&Privacy> {
        self.privacy
            .as_ref()
            .ok_or_else(|| Error::internal("privacy.service_missing"))
    }
}

/// A request to make.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewRequest {
    /// Whose data; the caller when left out.
    #[serde(default)]
    account: Option<PrincipalId>,
    /// `export` (the default) or `deletion`.
    #[serde(default)]
    kind: Option<String>,
    /// Why a deletion is asked for.
    #[serde(default)]
    reason: Option<String>,
}

/// The requests the caller made.
async fn list(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.requests(&caller.actor).await?).into_response())
}

/// Asks for an export.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<NewRequest>,
) -> ApiResult<Response> {
    let account = match (body.account, &caller.actor) {
        (Some(account), _) => account,
        (None, kampus_access::Actor::Principal(p) | kampus_access::Actor::ViewAs(p, _)) => *p,
        (None, kampus_access::Actor::Visitor(_)) => {
            return Err(Error::permission_denied("access.denied").into());
        }
    };
    let privacy = shared.privacy()?;
    let created = match body.kind.as_deref().unwrap_or("export") {
        "export" => privacy.request_export(&caller.actor, account).await?,
        "deletion" => {
            privacy
                .request_deletion(&caller.actor, account, body.reason.as_deref().unwrap_or_default())
                .await?
        }
        _ => {
            return Err(Error::validation(
                "privacy.kind_invalid",
                vec![kampus_common::FieldError::new("kind", "format")],
            )
            .into());
        }
    };
    Ok((StatusCode::ACCEPTED, Json(created)).into_response())
}

/// A request with its parts.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(request): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.request(&caller.actor, id(&request)?).await?).into_response())
}

/// The exported data.
async fn package(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(request): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.package(&caller.actor, id(&request)?).await?).into_response())
}

/// A version to publish.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewPolicy {
    /// The code of the policy.
    code: String,
    /// Its title.
    title: String,
    /// What it says.
    body: String,
}

/// The version being accepted.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Acceptance {
    /// The version.
    version: i32,
}

/// The policies of the campus of the caller.
async fn policies(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.policies(&caller.actor).await?).into_response())
}

/// The policies the caller has not accepted.
async fn pending(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.pending_policies(&caller.actor).await?).into_response())
}

/// Publishes a version of a policy.
async fn publish(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<NewPolicy>,
) -> ApiResult<Response> {
    let published = shared
        .privacy()?
        .publish_policy(&caller.actor, &body.code, &body.title, &body.body)
        .await?;
    Ok((StatusCode::CREATED, Json(published)).into_response())
}

/// Accepts a version of a policy.
async fn accept(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(code): Path<String>,
    Body(body): Body<Acceptance>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .privacy()?
            .accept_policy(&caller.actor, &code, body.version)
            .await?,
    )
    .into_response())
}

/// The decision on a deletion.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Decision {
    /// Whether the deletion goes ahead.
    approve: bool,
}

/// A hold to place.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewHold {
    /// The account it protects; the whole campus of the caller when left out.
    #[serde(default)]
    account: Option<PrincipalId>,
    /// Why deletion waits.
    reason: String,
    /// When it ends by itself.
    #[serde(default)]
    until: Option<kampus_common::Timestamp>,
}

/// The deletions waiting for a decision.
async fn awaiting(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.awaiting_approval(&caller.actor).await?).into_response())
}

/// Approves or rejects a deletion.
async fn decide(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(request): Path<String>,
    Body(body): Body<Decision>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .privacy()?
            .decide(&caller.actor, id(&request)?, body.approve)
            .await?,
    )
    .into_response())
}

/// The holds of the campus.
async fn holds(State(shared): State<Shared>, Extension(caller): Extension<Caller>) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.holds(&caller.actor).await?).into_response())
}

/// Places a hold.
async fn place(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<NewHold>,
) -> ApiResult<Response> {
    let placed = shared
        .privacy()?
        .place_hold(&caller.actor, body.account, &body.reason, body.until)
        .await?;
    Ok((StatusCode::CREATED, Json(placed)).into_response())
}

/// Releases a hold.
async fn release(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(hold): Path<String>,
) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.release_hold(&caller.actor, id(&hold)?).await?).into_response())
}

/// The days to keep a data category.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct KeepDays {
    /// Days after the account is deleted; none keeps the data for ever.
    #[serde(default)]
    keep_days: Option<i32>,
}

/// The data categories and their retention.
async fn retention(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    Ok(Json(shared.privacy()?.retention(&caller.actor).await?).into_response())
}

/// Sets the retention of a data category.
async fn set_retention(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(category): Path<String>,
    Body(body): Body<KeepDays>,
) -> ApiResult<Response> {
    Ok(Json(
        shared
            .privacy()?
            .set_retention(&caller.actor, &category, body.keep_days)
            .await?,
    )
    .into_response())
}
