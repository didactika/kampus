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

//! The routes of packages: exporting what a scope sets, and importing a package after reviewing its
//! differences.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use super::Shared;
use crate::permissions::PACKAGE_MANAGE;
use crate::{ImportId, Package, PackageKind};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/scopes/{scope}/packages", post(export))
        .route("/scopes/{scope}/imports", post(preview))
        .route("/imports/{id}", get(read))
        .route("/imports/{id}/apply", post(apply))
}

/// The kinds to export.
#[derive(Debug, Deserialize)]
struct ExportBody {
    /// The kinds of customisation the package holds.
    kinds: Vec<PackageKind>,
}

/// Exports what a scope sets of the kinds chosen.
async fn export(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(body): Body<ExportBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, PACKAGE_MANAGE, scope).await?;
    Ok(Json(shared.customise()?.export(actor, scope, &body.kinds).await?).into_response())
}

/// Reads a package for a scope and answers its differences.
async fn preview(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(package): Body<Package>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, PACKAGE_MANAGE, scope).await?;
    let import = shared.customise()?.preview_import(actor, scope, package).await?;
    Ok((StatusCode::CREATED, Json(import)).into_response())
}

/// An import with its differences.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(import): Path<String>,
) -> ApiResult<Response> {
    let import = shared.customise()?.import(id::<ImportId>(&import)?).await?;
    shared.acting(&caller, PACKAGE_MANAGE, import.scope).await?;
    Ok(Json(import).into_response())
}

/// Applies an import in one transaction.
async fn apply(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(import): Path<String>,
) -> ApiResult<Response> {
    let customise = shared.customise()?;
    let stored = customise.import(id::<ImportId>(&import)?).await?;
    let actor = shared.acting(&caller, PACKAGE_MANAGE, stored.scope).await?;
    Ok(Json(customise.apply_import(actor, stored.id).await?).into_response())
}
