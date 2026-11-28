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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Access through the server, as its administration uses it: the catalogue of permissions, the roles of a
//! campus and their values, grants, overrides and the explanation of a check.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn the_roles_of_a_campus_are_read_created_and_changed_through_the_server() {
    let site = site().await;
    let admin = site.session("admin").await;
    let campus = site.campus.to_string();

    let permissions = site
        .call(
            Method::GET,
            &format!("/api/v1/access/scopes/{campus}/permissions"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(permissions.status, StatusCode::OK, "{}", permissions.body);
    let review = permissions
        .body
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["code"] == "access.review")
        .unwrap();
    assert_eq!(
        (review["kind"].as_str(), review["risks"][0].as_str()),
        (Some("read"), Some("personal_data"))
    );

    let roles = site
        .call(
            Method::GET,
            &format!("/api/v1/access/scopes/{campus}/roles"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(roles.status, StatusCode::OK, "{}", roles.body);
    assert!(
        roles
            .body
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["template"] == "teacher" && r["values"].as_object().is_some_and(|v| !v.is_empty()))
    );

    let created = site
        .call(
            Method::POST,
            &format!("/api/v1/access/scopes/{campus}/roles"),
            Some(&admin),
            &[],
            Some(
                json!({ "short_name": "assistant", "template": "non_editing_teacher", "kinds": ["course"] }),
            ),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_eq!(created.body["kinds"], json!(["course"]));
    let role = created.body["id"].as_str().unwrap().to_owned();

    let forbidden = site
        .call(
            Method::PUT,
            &format!("/api/v1/access/roles/{role}/permissions/access.review"),
            Some(&admin),
            &[],
            Some(json!({ "value": "forbid" })),
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::OK, "{}", forbidden.body);
    assert_eq!(forbidden.body["values"]["access.review"], "forbid");
    let taken = site
        .call(
            Method::PUT,
            &format!("/api/v1/access/roles/{role}/permissions/access.review"),
            Some(&admin),
            &[],
            Some(json!({ "value": null })),
        )
        .await;
    assert!(
        taken.body["values"].get("access.review").is_none(),
        "{}",
        taken.body
    );
    let factor = site
        .call(
            Method::PUT,
            &format!("/api/v1/access/roles/{role}/second-factor"),
            Some(&admin),
            &[],
            Some(json!({ "required": true })),
        )
        .await;
    assert_eq!(factor.body["second_factor_required"], true, "{}", factor.body);

    site.drop().await;
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one scenario told from start to end")]
async fn a_scope_grants_overrides_and_explains_through_the_server() {
    let site = site().await;
    let admin = site.session("admin").await;
    let campus = site.campus.to_string();
    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&admin), &[], None)
        .await;
    let principal = me.body["id"].as_str().unwrap().to_owned();
    let manager = site.roles["manager"].to_string();

    let granted = site
        .call(
            Method::POST,
            &format!("/api/v1/access/scopes/{campus}/grants"),
            Some(&admin),
            &[],
            Some(json!({ "role": manager, "principal": principal })),
        )
        .await;
    assert_eq!(granted.status, StatusCode::CREATED, "{}", granted.body);
    let grant = granted.body["id"].as_str().unwrap().to_owned();
    let grants = site
        .call(
            Method::GET,
            &format!("/api/v1/access/scopes/{campus}/grants"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert!(
        grants
            .body
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["id"] == grant.as_str() && g["origin"].is_string()),
        "{}",
        grants.body
    );

    let overridden = site
        .call(
            Method::PUT,
            &format!("/api/v1/access/scopes/{campus}/overrides/{manager}/access.review"),
            Some(&admin),
            &[],
            Some(json!({ "value": "deny" })),
        )
        .await;
    assert_eq!(overridden.status, StatusCode::NO_CONTENT, "{}", overridden.body);
    let overrides = site
        .call(
            Method::GET,
            &format!("/api/v1/access/scopes/{campus}/overrides"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(overrides.body[0]["value"], "deny", "{}", overrides.body);

    let explained = site
        .call(
            Method::GET,
            &format!("/api/v1/access/scopes/{campus}/explain?principal={principal}&permission=access.review"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(explained.status, StatusCode::OK, "{}", explained.body);
    assert_eq!(
        (
            explained.body["allowed"].as_bool(),
            explained.body["rule"].as_str()
        ),
        (Some(true), Some("allowed"))
    );
    assert!(
        explained.body["grants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["role"] == manager.as_str() && g["value"] == "deny")
    );

    let held = site
        .call(
            Method::GET,
            &format!("/api/v1/access/principals/{principal}/grants"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert!(
        held.body
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["id"] == grant.as_str() && g["scope_kind"] == "campus"),
        "{}",
        held.body
    );

    let revoked = site
        .call(
            Method::DELETE,
            &format!("/api/v1/access/grants/{grant}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT, "{}", revoked.body);
    site.drop().await;
}

#[tokio::test]
async fn a_visitor_reviews_nothing_of_access() {
    let site = site().await;
    let campus = site.campus.to_string();
    let visitor = site
        .call(
            Method::GET,
            &format!("/api/v1/access/scopes/{campus}/roles"),
            None,
            &[],
            None,
        )
        .await;
    assert_eq!(visitor.status, StatusCode::FORBIDDEN, "a visitor reviews nothing");
    site.drop().await;
}
