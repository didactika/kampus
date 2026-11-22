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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of users: profiles with `ETag` and `If-Match`, preferences, and system groups with their
//! members and rules. Every route reads the caller identity authenticated.

mod picture;
mod references;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use kampus_access::ScopeId;
use kampus_common::{Error, Result};
use kampus_identity::http::Caller;
use kampus_identity::permissions::ACCOUNT_READ;
use kampus_identity::{AccountId, Identity};
use kampus_module::Operation;
use kampus_module::ServiceContext;
use kampus_module::http::{ApiResult, Body, Identified, id, if_match, with_etag};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::permissions::{GROUP_MANAGE, PROFILE_UPDATE_ANY, PROFILE_UPDATE_OWN};
use crate::{GroupId, GroupKind, GroupRule, Member, Profile, ProfileFields, SystemGroup, Users};

/// The routes of users.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        users: context.service::<Users>().ok(),
        identity: context.service::<Identity>().ok(),
        access: context.service::<kampus_access::Access>().ok(),
    };
    Router::new()
        .route("/me/profile", get(my_profile).put(update_my_profile))
        .route("/accounts/{id}/profile", get(profile).put(update_profile))
        .route("/me/preferences", get(preferences).patch(set_preferences))
        .route("/scopes/{scope}/system-groups", post(create_group))
        .route("/system-groups/{id}", get(group))
        .route("/system-groups/{id}/members", get(members))
        .route(
            "/system-groups/{id}/members/{account}",
            put(add_member).delete(remove_member),
        )
        .route("/system-groups/{id}/apply-rule", post(apply_rule))
        .merge(references::routes())
        .merge(picture::routes())
        .with_state(shared)
}

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[USERS, picture::OPERATIONS, references::OPERATIONS];

/// The operations of profiles, preferences and system groups.
const USERS: &[Operation] = &[
    Operation::get(
        "/me/profile",
        "The profile of the caller, with its version as ETag",
    )
    .returns::<Profile>(),
    Operation::put("/me/profile", "Replace the profile of the caller")
        .body::<ProfileFields>()
        .returns::<Profile>(),
    Operation::get(
        "/accounts/{id}/profile",
        "The profile of an account, with its version as ETag",
    )
    .returns::<Profile>(),
    Operation::put("/accounts/{id}/profile", "Replace the profile of an account")
        .body::<ProfileFields>()
        .returns::<Profile>(),
    Operation::get(
        "/me/preferences",
        "The preferences of the caller, with their defaults",
    )
    .returns::<Map<String, Value>>(),
    Operation::patch(
        "/me/preferences",
        "Set some preferences of the caller; null clears one",
    )
    .body::<Map<String, Value>>()
    .no_content(),
    Operation::post(
        "/scopes/{scope}/system-groups",
        "Create a system group in a scope",
    )
    .body::<GroupBody>()
    .creates::<Identified<GroupId>>(),
    Operation::get("/system-groups/{id}", "A system group").returns::<SystemGroup>(),
    Operation::get("/system-groups/{id}/members", "The members of a system group").returns::<Vec<Member>>(),
    Operation::put("/system-groups/{id}/members/{account}", "Add a member by hand").no_content(),
    Operation::delete("/system-groups/{id}/members/{account}", "Remove a member").no_content(),
    Operation::post(
        "/system-groups/{id}/apply-rule",
        "Apply the rule of a group to its members now",
    )
    .returns::<Applied>(),
];

/// How many members a group has after its rule was applied.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct Applied {
    /// The members.
    members: usize,
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    users: Option<Users>,
    /// Identity, for the scopes API tokens carry.
    identity: Option<Identity>,
    /// Access, for the checks the services leave to their callers.
    access: Option<kampus_access::Access>,
}

impl Shared {
    /// The service.
    fn users(&self) -> Result<&Users> {
        self.users
            .as_ref()
            .ok_or_else(|| Error::internal("users.service_missing"))
    }

    /// Identity.
    fn identity(&self) -> Result<&Identity> {
        self.identity
            .as_ref()
            .ok_or_else(|| Error::internal("users.identity_missing"))
    }

    /// Access.
    fn access(&self) -> Result<&kampus_access::Access> {
        self.access
            .as_ref()
            .ok_or_else(|| Error::internal("users.access_missing"))
    }

    /// The actor of an operation on an account: one's own needs `own`, someone else's needs `any`.
    async fn on_account<'c>(
        &self,
        caller: &'c Caller,
        account: AccountId,
        own: &str,
        any: &str,
    ) -> Result<&'c kampus_access::Actor> {
        let identity = self.identity()?;
        let campus = identity.campus_of(account).await?;
        let permission = if caller.account().ok() == Some(account) {
            own
        } else {
            any
        };
        caller.acting(identity, permission, campus).await
    }
}

/// A new system group.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GroupBody {
    /// Its code.
    code: String,
    /// Its name.
    name: String,
    /// Its rule; a group by hand without one.
    rule: Option<GroupRule>,
}

/// A profile with its version as `ETag`.
fn tagged(profile: crate::Profile) -> Response {
    let version = profile.version;
    with_etag(Json(profile), version)
}

/// Reads a profile.
async fn read(shared: &Shared, caller: &Caller, account: AccountId) -> ApiResult<Response> {
    let actor = shared
        .on_account(caller, account, ACCOUNT_READ, ACCOUNT_READ)
        .await?;
    Ok(tagged(shared.users()?.profile(actor, account).await?))
}

/// Saves a profile; `If-Match` refuses a change made on an old version.
async fn save(
    shared: &Shared,
    caller: &Caller,
    account: AccountId,
    headers: &HeaderMap,
    fields: ProfileFields,
) -> ApiResult<Response> {
    let expected = Some(if_match(headers)?);
    let actor = shared
        .on_account(caller, account, PROFILE_UPDATE_OWN, PROFILE_UPDATE_ANY)
        .await?;
    Ok(tagged(
        shared
            .users()?
            .update_profile(actor, account, fields, expected)
            .await?,
    ))
}

/// The profile of the caller.
async fn my_profile(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    read(&shared, &caller, caller.account()?).await
}

/// Saves the profile of the caller.
async fn update_my_profile(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    Body(fields): Body<ProfileFields>,
) -> ApiResult<Response> {
    save(&shared, &caller, caller.account()?, &headers, fields).await
}

/// The profile of an account.
async fn profile(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
) -> ApiResult<Response> {
    read(&shared, &caller, id(&account)?).await
}

/// Saves the profile of an account.
async fn update_profile(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(account): Path<String>,
    headers: HeaderMap,
    Body(fields): Body<ProfileFields>,
) -> ApiResult<Response> {
    save(&shared, &caller, id(&account)?, &headers, fields).await
}

/// The preferences of the caller, by key.
async fn preferences(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
) -> ApiResult<Response> {
    let account = caller.account()?;
    let actor = shared
        .on_account(&caller, account, ACCOUNT_READ, ACCOUNT_READ)
        .await?;
    let values = shared.users()?.preferences(actor, account).await?;
    let map: Map<String, Value> = values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
    Ok(Json(map).into_response())
}

/// Sets preferences of the caller; `null` goes back to the default.
async fn set_preferences(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Body(changes): Body<Map<String, Value>>,
) -> ApiResult<StatusCode> {
    let account = caller.account()?;
    let actor = shared
        .on_account(&caller, account, PROFILE_UPDATE_OWN, PROFILE_UPDATE_ANY)
        .await?;
    let users = shared.users()?;
    for (key, value) in changes {
        let value = if value.is_null() { None } else { Some(value) };
        users.set_preference(actor, account, &key, value).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Creates a system group in a campus or a category.
async fn create_group(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(scope): Path<String>,
    Body(body): Body<GroupBody>,
) -> ApiResult<Response> {
    let scope: ScopeId = id(&scope)?;
    let actor = caller.acting(shared.identity()?, GROUP_MANAGE, scope).await?;
    let kind = body.rule.map_or(GroupKind::Manual, GroupKind::Rule);
    let group = shared
        .users()?
        .create_group(actor, scope, &body.code, &body.name, kind)
        .await?;
    Ok((StatusCode::CREATED, Json(Identified { id: group })).into_response())
}

/// The actor managing a group, with the group's scope.
async fn managing<'c>(
    shared: &Shared,
    caller: &'c Caller,
    group: GroupId,
) -> Result<&'c kampus_access::Actor> {
    let scope = shared.users()?.group(group).await?.scope;
    let actor = caller.acting(shared.identity()?, GROUP_MANAGE, scope).await?;
    shared.access()?.require(actor, GROUP_MANAGE, scope).await?;
    Ok(actor)
}

/// A system group.
async fn group(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
) -> ApiResult<Response> {
    let group: GroupId = id(&group)?;
    managing(&shared, &caller, group).await?;
    Ok(Json(shared.users()?.group(group).await?).into_response())
}

/// The members of a system group.
async fn members(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
) -> ApiResult<Response> {
    let group: GroupId = id(&group)?;
    let actor = managing(&shared, &caller, group).await?;
    Ok(Json(shared.users()?.members(actor, group).await?).into_response())
}

/// Adds a member by hand.
async fn add_member(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((group, account)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let group: GroupId = id(&group)?;
    let actor = managing(&shared, &caller, group).await?;
    shared.users()?.add_member(actor, group, id(&account)?).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes a member added by hand.
async fn remove_member(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path((group, account)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let group: GroupId = id(&group)?;
    let actor = managing(&shared, &caller, group).await?;
    shared.users()?.remove_member(actor, group, id(&account)?).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Applies the rule of a system group.
async fn apply_rule(
    State(shared): State<Shared>,
    Extension(caller): Extension<Caller>,
    Path(group): Path<String>,
) -> ApiResult<Response> {
    let group: GroupId = id(&group)?;
    let actor = managing(&shared, &caller, group).await?;
    let members = shared.users()?.apply_rule(actor, group).await?;
    Ok(Json(Applied { members }).into_response())
}
