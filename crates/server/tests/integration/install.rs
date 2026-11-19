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

//! Installing: the configuration file, a new installation with its administrator, and an installed
//! database left as it is.

use std::collections::HashMap;
use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{ErrorKind, SystemClock};
use kampus_db::{Database, Prefix};
use kampus_identity::service::session::Client;
use kampus_identity::service::sign_in::{Credential, SignIn, SignInOutcome};
use kampus_identity::{FixedPolicies, Identity, MemoryCourier, SecretKey};
use kampus_server::Config;
use kampus_server::commands::install::{Installed, install};

/// The variables of an unattended installation, in its own prefix.
fn variables(prefix: &str, files: &str) -> HashMap<String, String> {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    [
        ("KAMPUS_DATABASE_URL", url.as_str()),
        ("KAMPUS_DATABASE_PREFIX", prefix),
        ("KAMPUS_SITE_URL", "https://campus.example"),
        ("KAMPUS_STORAGE_URL", files),
        ("KAMPUS_ADMIN_EMAIL", "admin@campus.example"),
        ("KAMPUS_ADMIN_PASSWORD", "a long and quiet river"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect()
}

#[tokio::test]
async fn an_unattended_install_without_values_names_every_missing_one() {
    let dir = tempfile::tempdir().unwrap();
    let error = install(&dir.path().join("config.toml"), &|_| None, false)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Validation);
    let fields: Vec<_> = error.fields().iter().map(|f| f.field.as_str()).collect();
    assert_eq!(fields, ["KAMPUS_SITE_URL", "KAMPUS_DATABASE_URL"]);
}

#[tokio::test]
async fn install_creates_the_installation_once_and_its_administrator_signs_in() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix = format!("n{}_", &simple[simple.len() - 6..]);
    let files = format!("file://{}", dir.path().join("files").display());
    let vars = variables(&prefix, &files);
    let env = |name: &str| vars.get(name).cloned();

    let installed = install(&path, &env, false).await.unwrap();
    assert!(matches!(&installed, Installed::New { administrator, .. } if administrator == "admin"));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("[security]\nsecret_key = \""), "{text}");
    assert!(text.contains("[tenancy]\nmode = \"single\""), "{text}");
    assert!(
        !text.contains("KAMPUS"),
        "values given by variables stay in the variables"
    );

    let again = install(&path, &env, false).await.unwrap();
    assert!(matches!(again, Installed::Existing { current: true, .. }));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        text,
        "nothing changes the second time"
    );

    let config = Config::load(env, Some(&text)).unwrap();
    let key: SecretKey = config.secret_key.clone().unwrap();
    let db = Database::connect(&config.database_url, prefix.parse::<Prefix>().unwrap(), 2)
        .await
        .unwrap();
    let campus: uuid::Uuid = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select id from {}.scope where kind = 'campus'",
        db.prefix().schema("access")
    )))
    .fetch_one(db.pool())
    .await
    .unwrap();
    let access = Access::new(db.clone(), Arc::new(SystemClock));
    let identity = Identity::new(
        db.clone(),
        access,
        Arc::new(SystemClock),
        Arc::new(FixedPolicies::default()),
        Arc::new(MemoryCourier::default()),
        key,
    );
    let outcome = identity
        .sign_in(
            kampus_access::ScopeId::from_uuid(campus),
            SignIn {
                name: "admin".to_owned(),
                credential: Credential::Password("a long and quiet river".to_owned()),
                client: Client::default(),
            },
        )
        .await
        .unwrap();
    assert!(matches!(outcome, SignInOutcome::SignedIn(_)));
    for module in ["enrol", "courses", "users", "identity", "access", "db"] {
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "drop schema if exists {} cascade",
            db.prefix().schema(module)
        )))
        .execute(db.pool())
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn an_installation_created_by_an_upgrade_gets_its_first_campus_from_install() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let simple = uuid::Uuid::now_v7().simple().to_string();
    let prefix = format!("u{}_", &simple[simple.len() - 6..]);
    let files = format!("file://{}", dir.path().join("files").display());
    let mut vars = variables(&prefix, &files);
    vars.insert(
        "KAMPUS_SECURITY_SECRET_KEY".to_owned(),
        SecretKey::generate().unwrap().to_base64(),
    );
    let env = |name: &str| vars.get(name).cloned();
    let config = Config::load(env, None).unwrap();
    kampus_server::commands::upgrade(&config, true).await.unwrap();

    let installed = install(&path, &env, false).await.unwrap();
    assert!(
        matches!(&installed, Installed::New { administrator, .. } if administrator == "admin"),
        "an installation without a campus gets its first one: {installed:?}"
    );
    let again = install(&path, &env, false).await.unwrap();
    assert!(matches!(again, Installed::Existing { current: true, .. }));

    let db = Database::connect(&config.database_url, prefix.parse::<Prefix>().unwrap(), 2)
        .await
        .unwrap();
    let campuses = Access::new(db.clone(), Arc::new(SystemClock))
        .campuses()
        .await
        .unwrap();
    assert_eq!(campuses.len(), 1);
    for module in ["enrol", "courses", "users", "identity", "access", "db"] {
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "drop schema if exists {} cascade",
            db.prefix().schema(module)
        )))
        .execute(db.pool())
        .await
        .unwrap();
    }
}
