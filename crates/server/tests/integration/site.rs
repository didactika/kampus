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

//! An installed site as a client sees it: its router, the notices it sent and its campus, with helpers to
//! send requests, sign in and read the links identity mailed.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use kampus_access::{Access, ScopeId, ScopeKind};
use kampus_db::{Database, Prefix, Upgrader};
use kampus_identity::{MemoryCourier, Notice, PasswordCost, SecretKey};
use kampus_server::config::Tenancy;
use kampus_server::http::campus::Campuses;
use kampus_server::http::{Api, AppState, router};
use kampus_server::wiring::modules::{SCHEMAS, sync_declarations};
use kampus_server::wiring::ports::Ports;
use kampus_server::wiring::services::{Parts, Services};
use serde_json::{Value, json};
use tower::ServiceExt;

/// The password of every account of these tests.
pub(crate) const PASSWORD: &str = "a-long-sentence-nobody-guesses";

/// The address of the site.
pub(crate) const SITE: &str = "http://kampus.test";

/// An installed site with its first administrator.
pub(crate) struct Site {
    /// The router of the process.
    pub(crate) router: Router,
    /// The notices identity sent.
    pub(crate) courier: Arc<MemoryCourier>,
    /// The campus.
    pub(crate) campus: ScopeId,
    /// The roles of the campus, by template.
    pub(crate) roles: std::collections::BTreeMap<&'static str, kampus_access::RoleId>,
    /// The services of the site.
    pub(crate) context: kampus_module::ServiceContext,
    /// The database of the site.
    pub(crate) db: Database,
    /// Keeps the files of the storage port.
    pub(crate) _files: tempfile::TempDir,
}

/// An answer.
pub(crate) struct Answer {
    /// Its status.
    pub(crate) status: StatusCode,
    /// Its headers.
    pub(crate) headers: HeaderMap,
    /// Its JSON body, `null` when empty, or its text when it is not JSON.
    pub(crate) body: Value,
}

impl Answer {
    /// The value of the session cookie it sets.
    pub(crate) fn session(&self) -> String {
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
    pub(crate) fn code(&self) -> &str {
        self.body["code"].as_str().unwrap_or_default()
    }
}

/// Installs a site as `kampus-server install` does.
pub(crate) async fn site() -> Site {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix: Prefix = format!("a{}_", &simple[simple.len() - 6..]).parse().unwrap();
    let db = Database::connect(&url, prefix, 4).await.unwrap();
    Upgrader::new(&db, &SCHEMAS, kampus_common::PLATFORM_VERSION)
        .apply(true)
        .await
        .unwrap();
    sync_declarations(&db).await.unwrap();
    let files = tempfile::tempdir().unwrap();
    let ports = Ports::for_tests(files.path().to_str().unwrap()).await.unwrap();
    let courier = Arc::new(MemoryCourier::default());
    let services = Services::build(Parts {
        storage: ports.storage.clone(),
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
    let roles = access.install_campus_roles(campus).await.unwrap();
    services
        .identity
        .install_administrator(campus, "admin@example.org", "admin", PASSWORD)
        .await
        .unwrap();
    let campuses = Campuses::new(Tenancy::Single, access);
    let context = services.context.clone();
    let api = Api { services, campuses };
    let router = router(
        AppState {
            db: db.clone(),
            modules: SCHEMAS.as_slice(),
            ports,
        },
        Some(&api),
    );
    Site {
        router,
        courier,
        campus,
        roles,
        context,
        db,
        _files: files,
    }
}

impl Site {
    /// Sends a request; `session` adds the cookie and the CSRF header.
    pub(crate) async fn call(
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
        self.exchange(request).await
    }

    /// Sends raw content, signed in with a session.
    pub(crate) async fn send_bytes(
        &self,
        method: Method,
        uri: &str,
        session: &str,
        content: &'static [u8],
    ) -> Answer {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::COOKIE, format!("kampus_session={session}"))
            .header("x-kampus", "1")
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(content))
            .unwrap();
        self.exchange(request).await
    }

    /// Sends a request to the router and reads its answer.
    async fn exchange(&self, request: Request<Body>) -> Answer {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        Answer {
            status,
            headers,
            body,
        }
    }

    /// Signs in with [`PASSWORD`].
    pub(crate) async fn sign_in(&self, name: &str) -> Answer {
        self.call(
            Method::POST,
            "/api/v1/identity/auth/sign-in",
            None,
            &[],
            Some(json!({ "name": name, "password": PASSWORD })),
        )
        .await
    }

    /// Signs in with [`PASSWORD`] and returns the session cookie.
    pub(crate) async fn session(&self, name: &str) -> String {
        let answer = self.sign_in(name).await;
        assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
        assert_eq!(answer.body["status"], "signed_in");
        answer.session()
    }

    /// The token of the last link sent to an email.
    pub(crate) fn token_sent_to(&self, email: &str) -> String {
        match self.courier.last_to(email).map(|m| m.notice) {
            Some(Notice::Invitation { token } | Notice::ConfirmEmail { token }) => token,
            other => panic!("no link sent to {email}: {other:?}"),
        }
    }
}

impl Site {
    /// Enrols an account in a course through a new manual instance giving a role.
    pub(crate) async fn enrol(&self, admin: &str, course: &str, account: &str, role: &str) -> String {
        let instance = self
            .call(
                Method::POST,
                &format!("/api/v1/enrol/courses/{course}/enrolment-instances"),
                Some(admin),
                &[],
                Some(json!({ "method": "manual", "name": format!("Manual {account}"), "role": self.roles[role] })),
            )
            .await;
        assert_eq!(instance.status, StatusCode::CREATED, "{}", instance.body);
        let instance = instance.body["id"].as_str().unwrap().to_owned();
        let enrolled = self
            .call(
                Method::POST,
                &format!("/api/v1/enrol/enrolment-instances/{instance}/enrolments"),
                Some(admin),
                &[],
                Some(json!({ "accounts": [account] })),
            )
            .await;
        assert_eq!(enrolled.status, StatusCode::OK, "{}", enrolled.body);
        assert!(enrolled.body[0]["enrolment"].is_string(), "{}", enrolled.body);
        instance
    }
}

impl Site {
    /// Drops the installation, every schema of its prefix.
    pub(crate) async fn drop(self) {
        let names = kampus_server::wiring::registry::MODULES
            .iter()
            .map(|m| m.name)
            .chain(["db"]);
        for name in names {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "drop schema if exists {} cascade",
                self.db.prefix().schema(name)
            )))
            .execute(self.db.pool())
            .await
            .unwrap();
        }
        self.db.close().await;
    }
}
