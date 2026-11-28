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

//! The API of the modules through the server, as a client sees it: the session cookie and its CSRF header,
//! the account and the profile with `If-Match`, invitations, second factors, API tokens and providers.

use std::collections::BTreeMap;

use axum::http::{Method, StatusCode, header};
use kampus_identity::support::totp;
use openidconnect::reqwest;
use openidconnect::url::Url;
use serde_json::json;

use crate::site::{PASSWORD, SITE, site, site_with_pool};

#[tokio::test]
async fn a_session_cookie_reads_the_account_and_needs_the_csrf_header_to_change_things() {
    let site = site().await;
    let answer = site.sign_in("admin").await;
    assert_eq!(answer.status, StatusCode::OK);
    let cookie = answer
        .headers
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"));
    assert!(
        !cookie.contains("Secure"),
        "the test site is not served over HTTPS"
    );
    let session = answer.session();

    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&session), &[], None)
        .await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["username"], "admin");
    assert!(me.headers.contains_key(header::ETAG));

    let anonymous = site
        .call(Method::GET, "/api/v1/identity/me", None, &[], None)
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous.code(), "identity.sign_in_required");
    assert_eq!(
        anonymous.body["request_id"].as_str(),
        anonymous.headers["x-request-id"].to_str().ok(),
        "problem details carry the request id"
    );

    let forged = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/sign-out",
            None,
            &[("cookie", &format!("kampus_session={session}"))],
            None,
        )
        .await;
    assert_eq!(forged.status, StatusCode::FORBIDDEN);
    assert_eq!(forged.code(), "identity.csrf_header_missing");

    let out = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/sign-out",
            Some(&session),
            &[],
            None,
        )
        .await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    let after = site
        .call(Method::GET, "/api/v1/identity/me", Some(&session), &[], None)
        .await;
    assert_eq!(after.status, StatusCode::UNAUTHORIZED);
    site.drop().await;
}

#[tokio::test]
async fn a_profile_change_on_an_old_version_is_refused() {
    let site = site().await;
    let session = site.session("admin").await;
    let read = site
        .call(Method::GET, "/api/v1/users/me/profile", Some(&session), &[], None)
        .await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(read.headers[header::ETAG], "\"0\"");

    let fields = json!({ "given_name": "Ana", "time_zone": "Europe/Madrid" });
    let saved = site
        .call(
            Method::PUT,
            "/api/v1/users/me/profile",
            Some(&session),
            &[("if-match", "\"0\"")],
            Some(fields.clone()),
        )
        .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    assert_eq!(saved.headers[header::ETAG], "\"1\"");
    assert_eq!(saved.body["given_name"], "Ana");

    let stale = site
        .call(
            Method::PUT,
            "/api/v1/users/me/profile",
            Some(&session),
            &[("if-match", "\"0\"")],
            Some(fields),
        )
        .await;
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);

    let invalid = site
        .call(
            Method::PUT,
            "/api/v1/users/me/profile",
            Some(&session),
            &[("if-match", "\"1\"")],
            Some(json!({ "time_zone": "Mars/Olympus" })),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid.body["fields"][0]["field"], "time_zone");
    let unconditional = site
        .call(
            Method::PUT,
            "/api/v1/users/me/profile",
            Some(&session),
            &[],
            Some(json!({ "given_name": "Eve" })),
        )
        .await;
    assert_eq!(unconditional.status, StatusCode::PRECONDITION_FAILED);
    assert_eq!(unconditional.code(), "module.if_match_required");

    let malformed = site
        .call(
            Method::PATCH,
            "/api/v1/users/me/preferences",
            Some(&session),
            &[],
            None,
        )
        .await;
    assert_eq!(malformed.code(), "module.body_invalid");
    site.drop().await;
}

#[tokio::test]
async fn an_invited_person_chooses_a_password_and_signs_in() {
    let site = site().await;
    let admin = site.session("admin").await;
    let invited = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/campuses/{}/accounts", site.campus),
            Some(&admin),
            &[],
            Some(json!({ "email": "ana@example.org" })),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
    let account = invited.body["id"].as_str().unwrap().to_owned();

    let token = site.token_sent_to("ana@example.org");
    let accepted = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/invitations/accept",
            None,
            &[],
            Some(json!({ "token": token, "password": PASSWORD })),
        )
        .await;
    assert_eq!(accepted.status, StatusCode::NO_CONTENT);
    let ana = site.session("ana@example.org").await;

    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/identity/accounts/{account}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(read.body["status"], "active");
    let version = read.headers[header::ETAG].to_str().unwrap().to_owned();
    let suspended = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/accounts/{account}/transitions"),
            Some(&admin),
            &[("if-match", &version)],
            Some(json!({ "to": "suspended", "reason": "testing" })),
        )
        .await;
    assert_eq!(suspended.status, StatusCode::NO_CONTENT, "{}", suspended.body);
    let again = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/accounts/{account}/transitions"),
            Some(&admin),
            &[("if-match", &version)],
            Some(json!({ "to": "active", "reason": "testing" })),
        )
        .await;
    assert_eq!(again.status, StatusCode::PRECONDITION_FAILED);
    let ended = site
        .call(Method::GET, "/api/v1/identity/me", Some(&ana), &[], None)
        .await;
    assert_eq!(
        ended.status,
        StatusCode::UNAUTHORIZED,
        "suspending ends the sessions"
    );

    let not_allowed = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/campuses/{}/accounts", site.campus),
            None,
            &[],
            Some(json!({ "email": "eve@example.org" })),
        )
        .await;
    assert_eq!(not_allowed.status, StatusCode::FORBIDDEN);
    site.drop().await;
}

#[tokio::test]
async fn an_authenticator_app_is_asked_at_the_next_sign_in() {
    let site = site().await;
    let session = site.session("admin").await;
    let setup = site
        .call(
            Method::POST,
            "/api/v1/identity/me/second-factors/totp",
            Some(&session),
            &[],
            None,
        )
        .await;
    assert_eq!(setup.status, StatusCode::OK, "{}", setup.body);
    let secret = setup.body["secret"].as_str().unwrap().to_owned();
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap();
    let code = totp::code(
        &totp::base32_decode(&secret).unwrap(),
        totp::step_of(now),
        totp::DIGITS,
    );
    let added = site
        .call(
            Method::POST,
            "/api/v1/identity/me/second-factors/totp/confirm",
            Some(&session),
            &[],
            Some(json!({ "label": "phone", "secret": secret, "code": code })),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.body);
    let recovery = added.body["recovery_codes"][0].as_str().unwrap().to_owned();
    let factors = site
        .call(
            Method::GET,
            "/api/v1/identity/me/second-factors",
            Some(&session),
            &[],
            None,
        )
        .await;
    assert_eq!(factors.body.as_array().unwrap().len(), 1);

    let asked = site.sign_in("admin").await;
    assert_eq!(asked.body["status"], "second_factor_required");
    assert_eq!(asked.body["methods"], json!(["totp"]));
    assert!(!asked.headers.contains_key(header::SET_COOKIE));
    let challenge = asked.body["challenge"].as_str().unwrap().to_owned();
    let completed = site
        .call(
            Method::POST,
            "/api/v1/identity/auth/second-factor",
            None,
            &[],
            Some(json!({ "challenge": challenge, "proof": { "kind": "recovery_code", "code": recovery } })),
        )
        .await;
    assert_eq!(completed.status, StatusCode::OK, "{}", completed.body);
    let strong = completed.session();
    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&strong), &[], None)
        .await;
    assert_eq!(me.status, StatusCode::OK);
    site.drop().await;
}

#[tokio::test]
async fn an_api_token_acts_only_within_what_it_carries() {
    let site = site().await;
    let session = site.session("admin").await;
    let created = site
        .call(
            Method::POST,
            "/api/v1/identity/me/tokens",
            Some(&session),
            &[],
            Some(json!({
                "name": "reports",
                "scopes": [{ "permission": "identity.account.read", "scope": site.campus.to_string() }],
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let bearer = format!("Bearer {}", created.body["token"].as_str().unwrap());
    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&session), &[], None)
        .await;
    let account = me.body["id"].as_str().unwrap().to_owned();

    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/identity/accounts/{account}"),
            None,
            &[("authorization", &bearer)],
            None,
        )
        .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.body);
    let invite = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/campuses/{}/accounts", site.campus),
            None,
            &[("authorization", &bearer)],
            Some(json!({ "email": "eve@example.org" })),
        )
        .await;
    assert_eq!(invite.status, StatusCode::FORBIDDEN);
    assert_eq!(invite.code(), "access.token_scope");
    let credentials = site
        .call(
            Method::GET,
            "/api/v1/identity/me/sessions",
            None,
            &[("authorization", &bearer)],
            None,
        )
        .await;
    assert_eq!(
        credentials.status,
        StatusCode::UNAUTHORIZED,
        "tokens do not manage credentials"
    );
    let invalid = site
        .call(
            Method::GET,
            "/api/v1/identity/me",
            None,
            &[("authorization", "Bearer kmp_not_a_token")],
            None,
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNAUTHORIZED);
    site.drop().await;
}

#[tokio::test]
async fn a_provider_signs_a_person_in_through_the_browser_flow() {
    let site = site().await;
    let admin = site.session("admin").await;
    let invited = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/campuses/{}/accounts", site.campus),
            Some(&admin),
            &[],
            Some(json!({ "email": "ana@example.org" })),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED);
    site.call(
        Method::POST,
        "/api/v1/identity/auth/invitations/accept",
        None,
        &[],
        Some(json!({ "token": site.token_sent_to("ana@example.org"), "password": PASSWORD })),
    )
    .await;
    let base = std::env::var("KAMPUS_TEST_OIDC_URL").expect("KAMPUS_TEST_OIDC_URL is set");
    let added = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/campuses/{}/identity-providers", site.campus),
            Some(&admin),
            &[],
            Some(json!({
                "issuer": format!("{base}/verified"),
                "client": "kampus",
                "client_secret": "provider secret",
                "linking_policy": "verified_email",
            })),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.body);
    let provider = added.body["id"].as_str().unwrap().to_owned();
    let listed = site
        .call(
            Method::GET,
            &format!("/api/v1/identity/campuses/{}/identity-providers", site.campus),
            None,
            &[],
            None,
        )
        .await;
    assert_eq!(listed.body.as_array().unwrap().len(), 1);

    let start = site
        .call(
            Method::GET,
            &format!("/api/v1/identity/auth/oidc/{provider}/start"),
            None,
            &[],
            None,
        )
        .await;
    assert_eq!(start.status, StatusCode::SEE_OTHER);
    let flow = start.headers[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    assert!(flow.starts_with("kampus_oidc="));
    let browser = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let at_provider = browser
        .get(start.headers[header::LOCATION].to_str().unwrap())
        .send()
        .await
        .unwrap();
    let back = Url::parse(at_provider.headers()["location"].to_str().unwrap()).unwrap();
    assert!(
        back.as_str()
            .starts_with(&format!("{SITE}/api/v1/identity/auth/oidc/{provider}/callback"))
    );
    let query: BTreeMap<String, String> = back.query_pairs().into_owned().collect();

    let callback = format!(
        "/api/v1/identity/auth/oidc/{provider}/callback?code={}&state={}",
        query["code"], query["state"]
    );
    let without_flow = site.call(Method::GET, &callback, None, &[], None).await;
    assert_eq!(without_flow.code(), "identity.oidc_flow_invalid");
    let signed_in = site
        .call(Method::GET, &callback, None, &[("cookie", &flow)], None)
        .await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.body);
    assert_eq!(signed_in.body["status"], "signed_in");
    let ana = signed_in.session();
    let me = site
        .call(Method::GET, "/api/v1/identity/me", Some(&ana), &[], None)
        .await;
    assert_eq!(me.body["email"], "ana@example.org");
    site.drop().await;
}

#[tokio::test]
async fn the_web_client_knows_the_api_it_talks_to() {
    let site = site().await;
    let answer = site
        .call(Method::GET, "/api/v1/openapi.json", None, &[], None)
        .await;
    let snapshot = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/sdk/openapi.json");
    let served = format!("{}\n", serde_json::to_string_pretty(&answer.body).unwrap());
    if std::env::var("KAMPUS_UPDATE_OPENAPI").is_ok_and(|v| v == "1") {
        std::fs::write(&snapshot, &served).unwrap();
    }
    let known = std::fs::read_to_string(&snapshot).unwrap_or_default();
    assert!(
        known == served,
        "web/sdk/openapi.json is not the API the server serves: run the server tests with KAMPUS_UPDATE_OPENAPI=1, then npm run generate -w @kampus/sdk"
    );
    site.drop().await;
}

#[tokio::test]
async fn the_pages_of_the_client_are_served_with_a_single_connection() {
    // A request that holds a connection while it waits for another one hangs here, where under load it would
    // starve the pool: with one connection, every answer comes at once or never.
    let site = site_with_pool(1).await;
    let wait = std::time::Duration::from_secs(10);
    let signed_in = tokio::time::timeout(wait, site.sign_in("admin"))
        .await
        .expect("signing in waits for no second connection");
    let session = signed_in.session();
    let campus = site.campus.as_uuid();
    let paths = [
        "/api/v1/identity/me".to_owned(),
        "/api/v1/users/me/profile".to_owned(),
        format!("/api/v1/customise/scopes/{campus}/menus/customise.main"),
        format!("/api/v1/customise/scopes/{campus}/layouts/customise.dashboard"),
        format!("/api/v1/customise/scopes/{campus}/theme"),
        "/api/v1/notify/inbox?limit=5".to_owned(),
        "/api/v1/notify/preferences".to_owned(),
        format!("/api/v1/courses/campuses/{campus}/categories"),
        "/api/v1/identity/me/sessions".to_owned(),
        "/api/v1/identity/me/second-factors".to_owned(),
    ];
    for path in &paths {
        let answer = tokio::time::timeout(wait, site.call(Method::GET, path, Some(&session), &[], None))
            .await
            .unwrap_or_else(|_| panic!("{path} waits for a second connection"));
        assert!(answer.status.is_success(), "{path}: {}", answer.body);
    }
    site.drop().await;
}

#[tokio::test]
async fn metrics_count_requests_by_route_template_not_by_path() {
    kampus_server::http::metrics::install();
    let site = site().await;
    for id in [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
    ] {
        site.call(
            Method::GET,
            &format!("/api/v1/courses/courses/{id}"),
            None,
            &[],
            None,
        )
        .await;
    }
    let answer = site.call(Method::GET, "/metrics", None, &[], None).await;
    assert_eq!(answer.status, StatusCode::OK);
    let text = answer.body.as_str().unwrap_or_default().to_owned();
    assert!(text.contains("kampus_http_requests_total"), "{text}");
    assert!(text.contains("route=\"/api/v1/courses/courses/{id}\""), "{text}");
    assert!(
        !text.contains("00000000-0000-0000-0000-000000000001"),
        "no path ever becomes a label"
    );
    assert!(text.contains("kampus_db_connections"));
    assert!(text.contains("kampus_build_info"));
    let me = site
        .call(Method::GET, "/api/v1/identity/me", None, &[], None)
        .await;
    assert_eq!(
        me.headers["cache-control"], "no-store",
        "answers of the API are never stored"
    );
    site.drop().await;
}

#[tokio::test]
async fn the_api_describes_the_routes_it_really_serves() {
    let site = site().await;
    let answer = site
        .call(Method::GET, "/api/v1/openapi.json", None, &[], None)
        .await;
    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.body["openapi"], "3.1.0");
    let privacy = &answer.body["paths"]["/privacy/requests"];
    assert!(privacy["get"].is_object() && privacy["post"].is_object());
    assert!(answer.body["paths"]["/privacy/holds/{id}/release"]["post"].is_object());
    assert!(answer.body["paths"]["/privacy/nothing"].is_null());
    assert!(privacy["post"]["requestBody"]["content"]["application/json"]["schema"].is_object());
    assert!(privacy["post"]["responses"]["202"]["content"]["application/json"]["schema"]["$ref"].is_string());
    assert!(answer.body["components"]["schemas"]["Problem"]["properties"]["code"].is_object());
    assert!(
        answer.body["tags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "records")
    );
    site.drop().await;
}
