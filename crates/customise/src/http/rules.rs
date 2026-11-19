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

//! The routes of rules: the rules of a scope and their runs, read and written by who holds
//! `customise.rule.manage` there; a rule is replaced with `If-Match`.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_identity::http::Caller;
use kampus_module::http::{ApiResult, Body, id, if_match, with_etag};

use super::Shared;
use crate::permissions::RULE_MANAGE;
use crate::{RuleId, RuleSpec};

/// The routes.
pub(super) fn routes() -> Router<Shared> {
    Router::new()
        .route("/scopes/{scope}/rules", get(list).post(create))
        .route("/rules/{id}", get(read).put(update).delete(remove))
        .route("/rules/{id}/runs", get(runs))
}

/// The rules a scope writes.
async fn list(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    shared.acting(&caller, RULE_MANAGE, scope).await?;
    Ok(Json(shared.customise()?.rules(scope).await?).into_response())
}

/// Writes a rule in a scope.
async fn create(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(spec): Body<RuleSpec>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = shared.acting(&caller, RULE_MANAGE, scope).await?;
    let rule = shared.customise()?.create_rule(actor, scope, spec).await?;
    let version = rule.version;
    Ok(with_etag((StatusCode::CREATED, Json(rule)), version))
}

/// A rule.
async fn read(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(rule): Path<String>,
) -> ApiResult<Response> {
    let rule = shared.customise()?.rule(id::<RuleId>(&rule)?).await?;
    shared.acting(&caller, RULE_MANAGE, rule.scope).await?;
    let version = rule.version;
    Ok(with_etag(Json(rule), version))
}

/// Replaces a rule, on the version it was read at.
async fn update(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(rule): Path<String>,
    headers: HeaderMap,
    Body(spec): Body<RuleSpec>,
) -> ApiResult<Response> {
    let customise = shared.customise()?;
    let stored = customise.rule(id::<RuleId>(&rule)?).await?;
    let expected = if_match(&headers)?;
    let actor = shared.acting(&caller, RULE_MANAGE, stored.scope).await?;
    let rule = customise.update_rule(actor, stored.id, spec, expected).await?;
    let version = rule.version;
    Ok(with_etag(Json(rule), version))
}

/// Removes a rule with its runs.
async fn remove(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(rule): Path<String>,
) -> ApiResult<StatusCode> {
    let customise = shared.customise()?;
    let stored = customise.rule(id::<RuleId>(&rule)?).await?;
    let actor = shared.acting(&caller, RULE_MANAGE, stored.scope).await?;
    customise.delete_rule(actor, stored.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The latest runs of a rule.
async fn runs(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(rule): Path<String>,
) -> ApiResult<Response> {
    let customise = shared.customise()?;
    let stored = customise.rule(id::<RuleId>(&rule)?).await?;
    shared.acting(&caller, RULE_MANAGE, stored.scope).await?;
    Ok(Json(customise.rule_runs(stored.id).await?).into_response())
}
