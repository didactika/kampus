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
// @package   kampus-files
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of files: opening an upload, sending its content and completing it; reading a file and
//! a link to it; and serving the content behind a link, with headers that never let it run in the
//! platform's origin.

mod quota;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::{Error, Result};
use kampus_identity::http::Caller;
use kampus_module::Operation;
use kampus_module::ServiceContext;
use kampus_module::http::{ApiResult, Body, id};
use serde::Deserialize;

use crate::service::principal;
use crate::service::serve::Part;
use crate::support::sniff;
use crate::{FileId, Files, UploadId};

/// The routes of files.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        files: context.service::<Files>().ok(),
        services: context.clone(),
    };
    Router::new()
        .route("/uploads", post(open))
        .route("/uploads/{id}", axum::routing::put(send).post(complete))
        .route("/files/{id}", get(read))
        .route("/files/{id}/url", post(url))
        .route("/files/{id}/conversions/{kind}", get(conversion))
        .route("/content/{id}", get(content))
        .merge(quota::routes())
        .with_state(shared)
}

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[FILES, quota::OPERATIONS];

/// The operations of uploads, files and their content.
const FILES: &[Operation] = &[
    Operation::post("/uploads", "Open an upload into an area of a scope")
        .body::<OpenBody>()
        .creates::<crate::Upload>(),
    Operation::put("/uploads/{id}", "Send the content of an open upload")
        .body_media("application/octet-stream")
        .returns::<crate::Upload>(),
    Operation::post("/uploads/{id}", "Complete an upload into a file").creates::<crate::File>(),
    Operation::get("/files/{id}", "A file the caller may read").returns::<crate::File>(),
    Operation::post(
        "/files/{id}/url",
        "A signed link that reads the file for a few minutes",
    )
    .returns::<crate::SignedUrl>(),
    Operation::get(
        "/files/{id}/conversions/{kind}",
        "A conversion of a file, such as its thumbnail; 202 while it is queued",
    )
    .returns::<crate::service::convert::Conversion>(),
    Operation::get("/content/{id}", "The content behind a signed link")
        .returns_media("application/octet-stream"),
];

/// What the handlers share.
#[derive(Clone)]
struct Shared {
    /// The service.
    files: Option<Files>,
    /// The services of the process, for the modules that answer whether a file is readable.
    services: ServiceContext,
}

impl Shared {
    /// The service.
    fn files(&self) -> Result<&Files> {
        self.files
            .as_ref()
            .ok_or_else(|| Error::internal("files.service_missing"))
    }
}

/// An upload to open.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OpenBody {
    /// The area.
    area: String,
    /// The scope it is for.
    scope: ScopeId,
    /// The name of the file.
    name: String,
    /// Its size in bytes.
    size: i64,
    /// The type the sender says it has; the content decides.
    #[serde(default)]
    media_type: Option<String>,
}

/// The signature of a link.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Link {
    /// When it stops working, as a Unix time.
    expires: i64,
    /// Its signature.
    signature: String,
    /// The conversion it reads, such as `thumbnail`; the original when absent.
    #[serde(default)]
    kind: Option<String>,
}

/// Opens an upload.
async fn open(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(body): Body<OpenBody>,
) -> ApiResult<Response> {
    let upload = shared
        .files()?
        .open_upload(
            &caller.actor,
            (&body.area, body.scope),
            &body.name,
            body.size,
            body.media_type.as_deref(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(upload)).into_response())
}

/// Receives the content of an upload, as the raw body.
async fn send(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(upload): Path<String>,
    content: Bytes,
) -> ApiResult<Response> {
    let upload: UploadId = id(&upload)?;
    let upload = shared
        .files()?
        .send_upload(&caller.actor, upload, content)
        .await?;
    Ok(Json(upload).into_response())
}

/// Completes an upload into a file.
async fn complete(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(upload): Path<String>,
) -> ApiResult<Response> {
    let upload: UploadId = id(&upload)?;
    let file = shared.files()?.complete_upload(&caller.actor, upload).await?;
    Ok((StatusCode::CREATED, Json(file)).into_response())
}

/// A file the caller may read.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(file): Path<String>,
) -> ApiResult<Response> {
    let file: FileId = id(&file)?;
    let found = shared
        .files()?
        .file(principal(&caller.actor), file, &shared.services)
        .await?;
    Ok(Json(found).into_response())
}

/// A link that reads a file for a few minutes.
async fn url(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(file): Path<String>,
) -> ApiResult<Response> {
    let file: FileId = id(&file)?;
    let link = shared
        .files()?
        .signed_url(principal(&caller.actor), file, &shared.services)
        .await?;
    Ok(Json(link).into_response())
}

/// The content of a file behind a valid link: never sniffed by the browser, sandboxed, and an attachment
/// when it could run.
async fn content(
    State(shared): State<Shared>,
    Path(file): Path<String>,
    Query(link): Query<Link>,
) -> ApiResult<Response> {
    let file: FileId = id(&file)?;
    let part = link.kind.as_deref().map_or(Part::Original, Part::Conversion);
    let (found, media_type, data) = shared
        .files()?
        .content(file, part, link.expires, &link.signature)
        .await?;
    let name = found.name.replace(['"', '\\'], "_");
    let disposition = if sniff::active(&media_type) {
        "attachment"
    } else {
        "inline"
    };
    Ok((
        [
            (header::CONTENT_TYPE, media_type),
            (
                header::CONTENT_DISPOSITION,
                format!("{disposition}; filename=\"{name}\""),
            ),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; sandbox".to_owned(),
            ),
            (header::CACHE_CONTROL, "private, max-age=300".to_owned()),
        ],
        data,
    )
        .into_response())
}

/// A conversion of a file the caller may read: `202` while it is made, and a link to it once done.
async fn conversion(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((file, kind)): Path<(String, String)>,
) -> ApiResult<Response> {
    let file: FileId = id(&file)?;
    let found = shared
        .files()?
        .conversion(principal(&caller.actor), file, &kind, &shared.services)
        .await?;
    let status = if found.state == "queued" {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(found)).into_response())
}
