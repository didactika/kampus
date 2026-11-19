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
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use kampus_access::{Access, ScopeId, ScopeKind};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_identity::support::totp;
use kampus_identity::{MemoryCourier, Notice, PasswordCost, SecretKey};
use kampus_server::config::Tenancy;
use kampus_server::http::campus::Campuses;
use kampus_server::http::{Api, AppState, router};
use kampus_server::wiring::modules::{SCHEMAS, sync_declarations};
use kampus_server::wiring::ports::Ports;
use kampus_server::wiring::services::{Parts, Services};
use openidconnect::reqwest;
use openidconnect::url::Url;
use serde_json::{Value, json};
use tower::ServiceExt;

/// The password of every account of these tests.
const PASSWORD: &str = "a long sentence nobody guesses";

/// The address of the site.
const SITE: &str = "http://kampus.test";

/// An installed site with its first administrator.
struct Site {
    /// The router of the process.
    router: Router,
    /// The notices identity sent.
    courier: Arc<MemoryCourier>,
    /// The campus.
    campus: ScopeId,
    /// Keeps the files of the storage port.
    _files: tempfile::TempDir,
}

/// An answer.
struct Answer {
    /// Its status.
    status: StatusCode,
    /// Its headers.
    headers: HeaderMap,
    /// Its JSON body, `null` when empty.
    body: Value,
}

impl Answer {
    /// The value of the session cookie it sets.
    fn session(&self) -> String {
        self.headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|v| v.strip_prefix("kampus_session="))
            .and_then(|v| v.split(';').next())
            .expect("a session cookie")
            .to_owned()
    }

    /// The code of its problem details.
    fn code(&self) -> &str {
        self.body["code"].as_str().unwrap_or_default()
    }
}

/// Installs a site as `kampus-server install` does.
async fn site() -> Site {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("a{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    Upgrader::new(&db, &SCHEMAS, kampus_common::PLATFORM_VERSION)
        .apply(true)
        .await
        .unwrap();
    sync_declarations(&db).await.unwrap();
    let courier = Arc::new(MemoryCourier::default());
    let services = Services::build(Parts {
        password_cost: Some(PasswordCost {
            memory_kib: 64,
            iterations: 1,
            parallelism: 1,
        }),
        ..Parts::new(
            db.clone(),
            SITE.to_owned(),
            SecretKey::generate().unwrap(),
            courier.clone(),
        )
    });
    let access: Access = services.access.clone();
    let platform = access.create_platform().await.unwrap();
    let campus = access.create_scope(platform, ScopeKind::Campus).await.unwrap();
    access.install_campus_roles(campus).await.unwrap();
    services
        .identity
        .install_administrator(campus, "admin@example.org", "admin", PASSWORD)
        .await
        .unwrap();
    let files = tempfile::tempdir().unwrap();
    let ports = Ports::for_tests(files.path().to_str().unwrap()).await.unwrap();
    let campuses = Campuses::new(Tenancy::Single, access);
    let api = Api { services, campuses };
    let router = router(
        AppState {
            db,
            modules: SCHEMAS.as_slice(),
            ports,
        },
        Some(&api),
    );
    Site {
        router,
        courier,
        campus,
        _files: files,
    }
}

impl Site {
    /// Sends a request; `session` adds the cookie and the CSRF header.
    async fn call(
        &self,
        method: Method,
        uri: &str,
        session: Option<&str>,
        extra: &[(&str, &str)],
        body: Option<Value>,
    ) -> Answer {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(session) = session {
            request = request
                .header(header::COOKIE, format!("kampus_session={session}"))
                .header("x-kampus", "1");
        }
        for (name, value) in extra {
            request = request.header(*name, *value);
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        Answer {
            status,
            headers,
            body,
        }
    }

    /// Signs in with [`PASSWORD`].
    async fn sign_in(&self, name: &str) -> Answer {
        self.call(
            Method::POST,
            "/api/v1/auth/sign-in",
            None,
            &[],
            Some(json!({ "name": name, "password": PASSWORD })),
        )
        .await
    }

    /// Signs in with [`PASSWORD`] and returns the session cookie.
    async fn session(&self, name: &str) -> String {
        let answer = self.sign_in(name).await;
        assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
        assert_eq!(answer.body["status"], "signed_in");
        answer.session()
    }

    /// The token of the last link sent to an email.
    fn token_sent_to(&self, email: &str) -> String {
        match self.courier.last_to(email).map(|m| m.notice) {
            Some(Notice::Invitation { token } | Notice::ConfirmEmail { token }) => token,
            other => panic!("no link sent to {email}: {other:?}"),
        }
    }
}

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
        .call(Method::GET, "/api/v1/me", Some(&session), &[], None)
        .await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["username"], "admin");
    assert!(me.headers.contains_key(header::ETAG));

    let anonymous = site.call(Method::GET, "/api/v1/me", None, &[], None).await;
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
            "/api/v1/auth/sign-out",
            None,
            &[("cookie", &format!("kampus_session={session}"))],
            None,
        )
        .await;
    assert_eq!(forged.status, StatusCode::FORBIDDEN);
    assert_eq!(forged.code(), "identity.csrf_header_missing");

    let out = site
        .call(Method::POST, "/api/v1/auth/sign-out", Some(&session), &[], None)
        .await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    let after = site
        .call(Method::GET, "/api/v1/me", Some(&session), &[], None)
        .await;
    assert_eq!(after.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_profile_change_on_an_old_version_is_refused() {
    let site = site().await;
    let session = site.session("admin").await;
    let read = site
        .call(Method::GET, "/api/v1/me/profile", Some(&session), &[], None)
        .await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(read.headers[header::ETAG], "\"0\"");

    let fields = json!({ "given_name": "Ana", "time_zone": "Europe/Madrid" });
    let saved = site
        .call(
            Method::PUT,
            "/api/v1/me/profile",
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
            "/api/v1/me/profile",
            Some(&session),
            &[("if-match", "\"0\"")],
            Some(fields),
        )
        .await;
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);

    let invalid = site
        .call(
            Method::PUT,
            "/api/v1/me/profile",
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
            "/api/v1/me/profile",
            Some(&session),
            &[],
            Some(json!({ "given_name": "Eve" })),
        )
        .await;
    assert_eq!(unconditional.status, StatusCode::PRECONDITION_FAILED);
    assert_eq!(unconditional.code(), "module.if_match_required");

    let malformed = site
        .call(Method::PATCH, "/api/v1/me/preferences", Some(&session), &[], None)
        .await;
    assert_eq!(malformed.code(), "module.body_invalid");
}

#[tokio::test]
async fn an_invited_person_chooses_a_password_and_signs_in() {
    let site = site().await;
    let admin = site.session("admin").await;
    let invited = site
        .call(
            Method::POST,
            &format!("/api/v1/campuses/{}/accounts", site.campus),
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
            "/api/v1/auth/invitations/accept",
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
            &format!("/api/v1/accounts/{account}"),
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
            &format!("/api/v1/accounts/{account}/transitions"),
            Some(&admin),
            &[("if-match", &version)],
            Some(json!({ "to": "suspended", "reason": "testing" })),
        )
        .await;
    assert_eq!(suspended.status, StatusCode::NO_CONTENT, "{}", suspended.body);
    let again = site
        .call(
            Method::POST,
            &format!("/api/v1/accounts/{account}/transitions"),
            Some(&admin),
            &[("if-match", &version)],
            Some(json!({ "to": "active", "reason": "testing" })),
        )
        .await;
    assert_eq!(again.status, StatusCode::PRECONDITION_FAILED);
    let ended = site.call(Method::GET, "/api/v1/me", Some(&ana), &[], None).await;
    assert_eq!(
        ended.status,
        StatusCode::UNAUTHORIZED,
        "suspending ends the sessions"
    );

    let not_allowed = site
        .call(
            Method::POST,
            &format!("/api/v1/campuses/{}/accounts", site.campus),
            None,
            &[],
            Some(json!({ "email": "eve@example.org" })),
        )
        .await;
    assert_eq!(not_allowed.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn an_authenticator_app_is_asked_at_the_next_sign_in() {
    let site = site().await;
    let session = site.session("admin").await;
    let setup = site
        .call(
            Method::POST,
            "/api/v1/me/second-factors/totp",
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
            "/api/v1/me/second-factors/totp/confirm",
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
            "/api/v1/me/second-factors",
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
            "/api/v1/auth/second-factor",
            None,
            &[],
            Some(json!({ "challenge": challenge, "proof": { "kind": "recovery_code", "code": recovery } })),
        )
        .await;
    assert_eq!(completed.status, StatusCode::OK, "{}", completed.body);
    let strong = completed.session();
    let me = site
        .call(Method::GET, "/api/v1/me", Some(&strong), &[], None)
        .await;
    assert_eq!(me.status, StatusCode::OK);
}

#[tokio::test]
async fn an_api_token_acts_only_within_what_it_carries() {
    let site = site().await;
    let session = site.session("admin").await;
    let created = site
        .call(
            Method::POST,
            "/api/v1/me/tokens",
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
        .call(Method::GET, "/api/v1/me", Some(&session), &[], None)
        .await;
    let account = me.body["id"].as_str().unwrap().to_owned();

    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/accounts/{account}"),
            None,
            &[("authorization", &bearer)],
            None,
        )
        .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.body);
    let invite = site
        .call(
            Method::POST,
            &format!("/api/v1/campuses/{}/accounts", site.campus),
            None,
            &[("authorization", &bearer)],
            Some(json!({ "email": "eve@example.org" })),
        )
        .await;
    assert_eq!(invite.status, StatusCode::FORBIDDEN);
    assert_eq!(invite.code(), "identity.api_token_scope");
    let credentials = site
        .call(
            Method::GET,
            "/api/v1/me/sessions",
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
            "/api/v1/me",
            None,
            &[("authorization", "Bearer kmp_not_a_token")],
            None,
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_provider_signs_a_person_in_through_the_browser_flow() {
    let site = site().await;
    let admin = site.session("admin").await;
    let invited = site
        .call(
            Method::POST,
            &format!("/api/v1/campuses/{}/accounts", site.campus),
            Some(&admin),
            &[],
            Some(json!({ "email": "ana@example.org" })),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED);
    site.call(
        Method::POST,
        "/api/v1/auth/invitations/accept",
        None,
        &[],
        Some(json!({ "token": site.token_sent_to("ana@example.org"), "password": PASSWORD })),
    )
    .await;
    let base = std::env::var("KAMPUS_TEST_OIDC_URL").expect("KAMPUS_TEST_OIDC_URL is set");
    let added = site
        .call(
            Method::POST,
            &format!("/api/v1/campuses/{}/identity-providers", site.campus),
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
            &format!("/api/v1/campuses/{}/identity-providers", site.campus),
            None,
            &[],
            None,
        )
        .await;
    assert_eq!(listed.body.as_array().unwrap().len(), 1);

    let start = site
        .call(
            Method::GET,
            &format!("/api/v1/auth/oidc/{provider}/start"),
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
            .starts_with(&format!("{SITE}/api/v1/auth/oidc/{provider}/callback"))
    );
    let query: BTreeMap<String, String> = back.query_pairs().into_owned().collect();

    let callback = format!(
        "/api/v1/auth/oidc/{provider}/callback?code={}&state={}",
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
    let me = site.call(Method::GET, "/api/v1/me", Some(&ana), &[], None).await;
    assert_eq!(me.body["email"], "ana@example.org");
}
