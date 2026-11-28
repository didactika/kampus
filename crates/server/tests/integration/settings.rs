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

//! Settings through the server: the identity policy of a campus comes from its settings, so an operator
//! opens sign-up and lengthens passwords without a new release.

use axum::http::{Method, StatusCode};
use kampus_settings::Settings;
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn the_policy_of_a_campus_follows_its_settings() {
    let site = site().await;
    let sign_up = json!({ "email": "lea@example.org", "accepted_terms": true });
    let closed = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/sign-up",
            None,
            &[],
            Some(sign_up.clone()),
        )
        .await;
    assert_eq!(closed.status, StatusCode::FORBIDDEN);
    assert_eq!(closed.code(), "identity.sign_up_closed");

    let settings = site.context.service::<Settings>().unwrap();
    settings
        .set_as_operator("identity.self_sign_up", site.campus, Some(json!(true)), false)
        .await
        .unwrap();
    settings
        .set_as_operator(
            "identity.password_min_length",
            site.campus,
            Some(json!(40)),
            false,
        )
        .await
        .unwrap();
    let open = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/sign-up",
            None,
            &[],
            Some(sign_up),
        )
        .await;
    assert_eq!(open.status, StatusCode::ACCEPTED, "{}", open.body);
    let token = site.token_sent_to("lea@example.org");
    let short = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/confirm",
            None,
            &[],
            Some(json!({ "token": token, "password": "only-twenty-six-characters" })),
        )
        .await;
    assert_eq!(
        short.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a longer password is asked now"
    );
    site.drop().await;
}

#[tokio::test]
async fn the_settings_of_a_scope_are_read_set_and_taken_back_through_the_server() {
    let site = site().await;
    let admin = site.session("admin").await;
    let campus = site.campus.to_string();
    let path = format!("/api/v1/settings/scopes/{campus}/identity.self_sign_up");

    let all = site
        .call(
            Method::GET,
            &format!("/api/v1/settings/scopes/{campus}?q=sign%20up"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(all.status, StatusCode::OK, "{}", all.body);
    let found = all.body.as_array().unwrap();
    assert!(
        found
            .iter()
            .all(|s| s["summary"].as_str().unwrap().to_lowercase().contains("sign up"))
    );
    let sign_up = found
        .iter()
        .find(|s| s["key"] == "identity.self_sign_up")
        .unwrap();
    assert_eq!(sign_up["kind"], json!({ "type": "boolean" }));
    assert_eq!(
        (sign_up["version"].as_i64(), sign_up["origin"]["from"].as_str()),
        (Some(0), Some("default"))
    );

    let without = site
        .call(
            Method::PUT,
            &path,
            Some(&admin),
            &[],
            Some(json!({ "value": true })),
        )
        .await;
    assert_eq!(
        without.status,
        StatusCode::PRECONDITION_FAILED,
        "a change says the version it read"
    );
    let set = site
        .call(
            Method::PUT,
            &path,
            Some(&admin),
            &[("if-match", "\"0\"")],
            Some(json!({ "value": true })),
        )
        .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.body);
    assert_eq!(
        (set.body["value"].as_bool(), set.body["version"].as_i64()),
        (Some(true), Some(1))
    );
    assert_eq!(set.headers["etag"], "\"1\"");
    let stale = site
        .call(
            Method::PUT,
            &path,
            Some(&admin),
            &[("if-match", "\"0\"")],
            Some(json!({ "value": false })),
        )
        .await;
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);
    let wrong = site
        .call(
            Method::PUT,
            &path,
            Some(&admin),
            &[("if-match", "\"1\"")],
            Some(json!({ "value": "yes" })),
        )
        .await;
    assert_eq!(wrong.code(), "settings.value_invalid");
    let changes = site
        .call(Method::GET, &format!("{path}/history"), Some(&admin), &[], None)
        .await;
    assert_eq!(changes.body.as_array().map(Vec::len), Some(1), "{}", changes.body);
    assert_eq!(changes.body[0]["after"], json!(true));

    let back = site.call(Method::DELETE, &path, Some(&admin), &[], None).await;
    assert_eq!(back.status, StatusCode::OK, "{}", back.body);
    assert_eq!(
        (
            back.body["version"].as_i64(),
            back.body["origin"]["from"].as_str()
        ),
        (Some(0), Some("default"))
    );
    site.drop().await;
}

#[tokio::test]
async fn a_setting_locked_above_reads_as_held_and_refuses_a_change_below() {
    let site = site().await;
    let admin = site.session("admin").await;
    let path = format!("/api/v1/settings/scopes/{}/identity.self_sign_up", site.campus);
    let platform = site
        .context
        .service::<kampus_access::Access>()
        .unwrap()
        .platform()
        .await
        .unwrap()
        .unwrap();
    site.context
        .service::<Settings>()
        .unwrap()
        .set_as_operator("identity.self_sign_up", platform, Some(json!(false)), true)
        .await
        .unwrap();
    let held = site.call(Method::GET, &path, Some(&admin), &[], None).await;
    assert_eq!(
        (held.body["locked_above"].as_bool(), held.body["value"].as_bool()),
        (Some(true), Some(false))
    );
    let refused = site
        .call(
            Method::PUT,
            &path,
            Some(&admin),
            &[("if-match", "\"0\"")],
            Some(json!({ "value": true })),
        )
        .await;
    assert_eq!(refused.code(), "settings.locked_above");
    site.context
        .service::<Settings>()
        .unwrap()
        .set_as_operator("identity.self_sign_up", platform, None, false)
        .await
        .unwrap();

    site.drop().await;
}

#[tokio::test]
async fn an_api_token_sees_only_the_settings_whose_permission_it_carries() {
    let site = site().await;
    let admin = site.session("admin").await;
    let created = site
        .call(
            Method::POST,
            "/api/v1/identity/me/tokens",
            Some(&admin),
            &[],
            Some(json!({
                "name": "reports",
                "scopes": [{ "permission": "identity.account.read", "scope": site.campus.to_string() }],
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let bearer = format!("Bearer {}", created.body["token"].as_str().unwrap());
    let campus = site.campus.to_string();
    let seen = site
        .call(
            Method::GET,
            &format!("/api/v1/settings/scopes/{campus}"),
            None,
            &[("authorization", &bearer)],
            None,
        )
        .await;
    assert_eq!(seen.status, StatusCode::OK, "{}", seen.body);
    assert_eq!(
        seen.body,
        json!([]),
        "the token carries no permission of a setting"
    );
    let one = site
        .call(
            Method::GET,
            &format!("/api/v1/settings/scopes/{campus}/identity.self_sign_up"),
            None,
            &[("authorization", &bearer)],
            None,
        )
        .await;
    assert_eq!(one.status, StatusCode::NOT_FOUND);
    let visitor = site
        .call(
            Method::GET,
            &format!("/api/v1/settings/scopes/{campus}"),
            None,
            &[],
            None,
        )
        .await;
    assert_eq!(visitor.body, json!([]));
    site.drop().await;
}
