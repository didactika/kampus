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

//! The HTTP routes of customisation: the words of a scope, its feature flags and its custom fields here,
//! its layouts, menus and theme in [`pages`], its rules in [`rules`], and its packages in [`packages`].

mod packages;
mod pages;
mod rules;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::{Error, Result};
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id};
use kampus_module::{Operation, ServiceContext};
use serde::Deserialize;

use crate::permissions::{FIELD_MANAGE, FLAG_MANAGE, TERM_MANAGE};
use crate::{Customise, FieldId, FieldSpec};

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[kampus_module::Operation]] = &[
    CUSTOMISE,
    pages::OPERATIONS,
    rules::OPERATIONS,
    packages::OPERATIONS,
];

/// The operations of terms, flags and custom fields.
const CUSTOMISE: &[kampus_module::Operation] = &[
    Operation::get(
        "/scopes/{scope}/terms/{code}",
        "A term of a scope in a language (query `language`)",
    )
    .returns::<crate::Term>(),
    Operation::put("/scopes/{scope}/terms/{code}", "Rename a term in a scope")
        .body::<Rename>()
        .returns::<crate::Term>(),
    Operation::delete(
        "/scopes/{scope}/terms/{code}",
        "Take back the name a scope gave a term (query `language`)",
    )
    .returns::<crate::Term>(),
    Operation::get("/scopes/{scope}/flags", "The features of a scope, on or off")
        .returns::<Vec<crate::Flag>>(),
    Operation::get("/scopes/{scope}/flags/{code}", "A feature of a scope").returns::<crate::Flag>(),
    Operation::put(
        "/scopes/{scope}/flags/{code}",
        "Turn a feature of a scope on or off, and lock it",
    )
    .body::<FlagBody>()
    .returns::<crate::Flag>(),
    Operation::get(
        "/scopes/{scope}/fields",
        "The custom fields of a kind of entity in a scope (query `kind`)",
    )
    .returns::<Vec<crate::Field>>(),
    Operation::post("/scopes/{scope}/fields", "Create a custom field in a scope")
        .body::<NewField>()
        .creates::<crate::Field>(),
    Operation::get("/fields/{id}", "A custom field").returns::<crate::Field>(),
    Operation::delete("/fields/{id}", "Delete a custom field with its values").no_content(),
];

/// The routes of customisation.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        customise: context.service::<Customise>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .route(
            "/scopes/{scope}/terms/{code}",
            get(term).put(rename_term).delete(reset_term),
        )
        .route("/scopes/{scope}/flags", get(flags))
        .route("/scopes/{scope}/flags/{code}", get(flag).put(set_flag))
        .route("/scopes/{scope}/fields", get(fields).post(create_field))
        .route("/fields/{id}", get(field).delete(delete_field))
        .merge(pages::routes())
        .merge(rules::routes())
        .merge(packages::routes())
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    customise: Option<Customise>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn customise(&self) -> Result<&Customise> {
        self.customise
            .as_ref()
            .ok_or_else(|| Error::internal("customise.service_missing"))
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
            .ok_or_else(|| Error::internal("customise.identity_missing"))?;
        caller.acting(identity, permission, scope).await
    }
}

/// The language a term is read in.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Language {
    /// A BCP 47 tag; English when absent.
    #[serde(default = "english")]
    language: String,
}

/// English.
fn english() -> String {
    "en".to_owned()
}

/// The word a scope gives a term.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Rename {
    /// The language.
    #[serde(default = "english")]
    language: String,
    /// The word for one.
    singular: String,
    /// The word for several.
    plural: String,
    /// Whether the scopes below keep it.
    #[serde(default)]
    locked: bool,
}

/// The value a scope gives a flag.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FlagBody {
    /// Whether the feature is on.
    enabled: bool,
    /// Whether the scopes below keep it.
    #[serde(default)]
    locked: bool,
}

/// The kind of entity fields are listed for.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Kind {
    /// The kind, such as `courses.course`.
    kind: String,
}

/// A custom field to add.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NewField {
    /// The kind of entity it belongs to.
    kind: String,
    /// The field.
    #[serde(flatten)]
    spec: FieldSpec,
}

/// The word for a term in a scope.
async fn term(
    State(shared): State<Shared>,
    Path((scope, code)): Path<(String, String)>,
    Query(language): Query<Language>,
) -> ApiResult<Response> {
    let term = shared
        .customise()?
        .term(&code, id(&scope)?, &language.language)
        .await?;
    Ok(Json(term).into_response())
}

/// Renames a term in a scope.
async fn rename_term(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, code)): Path<(String, String)>,
    Body(body): Body<Rename>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, TERM_MANAGE, scope).await?;
    let term = shared
        .customise()?
        .rename_term(
            actor,
            &code,
            scope,
            &body.language,
            (&body.singular, &body.plural),
            body.locked,
        )
        .await?;
    Ok(Json(term).into_response())
}

/// Removes the word a scope gave a term.
async fn reset_term(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, code)): Path<(String, String)>,
    Query(language): Query<Language>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, TERM_MANAGE, scope).await?;
    let term = shared
        .customise()?
        .reset_term(actor, &code, scope, &language.language)
        .await?;
    Ok(Json(term).into_response())
}

/// The feature flags of a scope.
async fn flags(State(shared): State<Shared>, Path(scope): Path<String>) -> ApiResult<Response> {
    Ok(Json(shared.customise()?.flags(id(&scope)?).await?).into_response())
}

/// One feature flag of a scope.
async fn flag(
    State(shared): State<Shared>,
    Path((scope, code)): Path<(String, String)>,
) -> ApiResult<Response> {
    Ok(Json(shared.customise()?.flag(&code, id(&scope)?).await?).into_response())
}

/// Turns a feature on or off in a scope.
async fn set_flag(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((scope, code)): Path<(String, String)>,
    Body(body): Body<FlagBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, FLAG_MANAGE, scope).await?;
    let flag = shared
        .customise()?
        .set_flag(actor, &code, scope, body.enabled, body.locked)
        .await?;
    Ok(Json(flag).into_response())
}

/// The custom fields of a kind of entity that apply in a scope.
async fn fields(
    State(shared): State<Shared>,
    Path(scope): Path<String>,
    Query(kind): Query<Kind>,
) -> ApiResult<Response> {
    let fields = shared.customise()?.fields_for(&kind.kind, id(&scope)?).await?;
    Ok(Json(fields).into_response())
}

/// Adds a custom field in a scope.
async fn create_field(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(body): Body<NewField>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, FIELD_MANAGE, scope).await?;
    let field = shared
        .customise()?
        .create_field(actor, &body.kind, scope, body.spec)
        .await?;
    Ok((StatusCode::CREATED, Json(field)).into_response())
}

/// A custom field.
async fn field(State(shared): State<Shared>, Path(field): Path<String>) -> ApiResult<Response> {
    let field: FieldId = id(&field)?;
    Ok(Json(shared.customise()?.field(field).await?).into_response())
}

/// Removes a custom field with its values.
async fn delete_field(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(field): Path<String>,
) -> ApiResult<StatusCode> {
    let customise = shared.customise()?;
    let stored = customise.field(id(&field)?).await?;
    let actor = shared.acting(&caller, FIELD_MANAGE, stored.scope).await?;
    customise.delete_field(actor, stored.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
