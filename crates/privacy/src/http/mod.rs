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

//! The HTTP routes of privacy: asking for an export, following the request and reading the package.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use kampus_access::PrincipalId;
use kampus_common::{Error, Result};
use kampus_identity::http::Caller;
use kampus_module::ServiceContext;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use crate::Privacy;

/// The routes of privacy.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        privacy: context.service::<Privacy>().ok(),
    };
    Router::new()
        .route("/requests", get(list).post(create))
        .route("/requests/{id}", get(read))
        .route("/requests/{id}/package", get(package))
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
#[derive(Debug, Deserialize)]
struct NewRequest {
    /// Whose data; the caller when left out.
    #[serde(default)]
    account: Option<PrincipalId>,
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
    let created = shared.privacy()?.request_export(&caller.actor, account).await?;
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
#[derive(Debug, Deserialize)]
struct NewPolicy {
    /// The code of the policy.
    code: String,
    /// Its title.
    title: String,
    /// What it says.
    body: String,
}

/// The version being accepted.
#[derive(Debug, Deserialize)]
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
