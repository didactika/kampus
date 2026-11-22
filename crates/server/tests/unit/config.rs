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

//! Tests of `kampus-server`: the configuration, from `config.toml` and the environment.

use std::collections::HashMap;

use kampus_server::config::file::{KEYS, Values, variable};
use kampus_server::config::{LogFormat, Tenancy};
use kampus_server::{Config, Role};

/// A valid secret key, 32 bytes in base64.
const KEY: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8";

/// Loads a configuration from pairs of variables and an optional file.
fn load(vars: &[(&str, &str)], file: Option<&str>) -> kampus_common::Result<Config> {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    Config::load(|name| vars.get(name).cloned(), file)
}

/// The variables every test starts from.
const BASE: &[(&str, &str)] = &[
    ("KAMPUS_DATABASE_URL", "postgres://kampus@db/kampus"),
    ("KAMPUS_SITE_URL", "https://campus.example"),
    ("KAMPUS_STORAGE_URL", "file:///var/lib/kampus"),
    ("KAMPUS_SECURITY_SECRET_KEY", KEY),
];

#[test]
fn every_key_has_its_variable() {
    assert_eq!(variable("database.url"), "KAMPUS_DATABASE_URL");
    assert_eq!(variable("server.http_address"), "KAMPUS_SERVER_HTTP_ADDRESS");
    assert!(KEYS.iter().all(|k| k.path.contains('.') && !k.summary.is_empty()));
}

#[test]
fn defaults_apply_when_only_the_required_values_are_set() {
    let config = load(BASE, None).unwrap();
    assert_eq!(config.role, Role::All);
    assert_eq!(config.http_address.to_string(), "0.0.0.0:8080");
    assert_eq!(config.database_prefix.as_str(), "kmp_");
    assert_eq!(config.database_pool_size, 10);
    assert!(!config.upgrade_on_start);
    assert_eq!(config.log_format, LogFormat::Json);
    assert_eq!(config.tenancy, Tenancy::Single);
    assert!(config.secret_key.is_some());
}

#[test]
fn every_problem_is_reported_at_once() {
    let error = load(
        &[
            ("KAMPUS_DATABASE_PREFIX", "BAD"),
            ("KAMPUS_DATABASE_POOL_SIZE", "0"),
            ("KAMPUS_SERVER_ROLE", "everything"),
            ("KAMPUS_SECURITY_SECRET_KEY", "short"),
            ("KAMPUS_TENANCY_MODE", "multi"),
        ],
        None,
    )
    .unwrap_err();
    let fields: Vec<_> = error
        .fields()
        .iter()
        .map(|f| (f.field.as_str(), f.rule.as_str()))
        .collect();
    assert_eq!(
        fields,
        [
            ("KAMPUS_SERVER_ROLE", "one_of"),
            ("KAMPUS_SITE_URL", "required"),
            ("KAMPUS_DATABASE_URL", "required"),
            ("KAMPUS_DATABASE_PREFIX", "format"),
            ("KAMPUS_DATABASE_POOL_SIZE", "range"),
            ("KAMPUS_STORAGE_URL", "required"),
            ("KAMPUS_SECURITY_SECRET_KEY", "base64_32_bytes"),
            ("KAMPUS_TENANCY_BASE_DOMAIN", "required"),
        ]
    );
}

#[test]
fn a_scheduler_needs_neither_the_site_nor_the_storage_nor_the_key() {
    let config = load(
        &[
            ("KAMPUS_SERVER_ROLE", "scheduler"),
            ("KAMPUS_DATABASE_URL", "postgres://db/kampus"),
        ],
        None,
    )
    .unwrap();
    assert_eq!(config.role, Role::Scheduler);
    assert!(config.site_url.is_none());
    assert!(config.storage_url.is_none());
    assert!(config.secret_key.is_none());
}

#[test]
fn variables_take_precedence_over_the_sections_of_the_file() {
    let file = "[site]\nurl = \"https://file.example\"\n\n[server]\nupgrade_on_start = true\n\n[database]\nurl = \"postgres://file/kampus\"\npool_size = 4\n\n[storage]\nurl = \"s3://kampus/files\"\ns3_endpoint = \"http://s3:8333\"\n\n[security]\nsecret_key = \"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8\"\n\n[tenancy]\nmode = \"multi\"\nbase_domain = \"campuses.example\"\n";
    let config = load(&[("KAMPUS_SITE_URL", "https://env.example")], Some(file)).unwrap();
    assert_eq!(config.database_url, "postgres://file/kampus");
    assert_eq!(config.site_url.as_deref(), Some("https://env.example"));
    assert_eq!(config.database_pool_size, 4);
    assert!(config.upgrade_on_start);
    assert_eq!(config.storage_url.as_deref(), Some("s3://kampus/files"));
    assert_eq!(
        config.storage_options.get("s3_endpoint").map(String::as_str),
        Some("http://s3:8333")
    );
    assert_eq!(
        config.tenancy,
        Tenancy::Multi {
            base_domain: "campuses.example".to_owned()
        }
    );
}

#[test]
fn unknown_keys_and_sections_in_the_file_are_refused() {
    let error = load(BASE, Some("flat = 1\n\n[database]\nulr = \"x\"\n")).unwrap_err();
    let fields: Vec<_> = error.fields().iter().map(|f| f.field.as_str()).collect();
    assert_eq!(
        fields,
        ["KAMPUS_CONFIG_FILE.database.ulr", "KAMPUS_CONFIG_FILE.flat"]
    );
}

#[test]
fn the_file_is_written_by_sections_and_read_back_the_same() {
    let mut values = Values::default();
    values.set("database.url", "postgres://u:p\"w@db/kampus");
    values.set("site.url", "https://campus.example");
    values.set("database.pool_size", "12");
    values.set("server.upgrade_on_start", "true");
    values.set("cache.url", "");
    let text = values.render();
    assert_eq!(
        text,
        "[site]\nurl = \"https://campus.example\"\n\n[server]\nupgrade_on_start = true\n\n[database]\nurl = \"postgres://u:p\\\"w@db/kampus\"\npool_size = 12\n"
    );
    let (again, problems) = Values::parse(&text);
    assert_eq!(problems, []);
    assert_eq!(again, values);
}

#[test]
fn the_public_address_of_the_store_reaches_the_adapter_and_is_an_address() {
    let mut vars = BASE.to_vec();
    vars.push(("KAMPUS_STORAGE_PUBLIC_URL", "https://files.campus.example"));
    let config = load(&vars, None).unwrap();
    assert_eq!(
        config.storage_options.get("public_url").map(String::as_str),
        Some("https://files.campus.example")
    );
    let mut wrong = BASE.to_vec();
    wrong.push(("KAMPUS_STORAGE_PUBLIC_URL", "files.campus.example"));
    let error = load(&wrong, None).unwrap_err();
    let fields: Vec<_> = error
        .fields()
        .iter()
        .map(|f| (f.field.as_str(), f.rule.as_str()))
        .collect();
    assert_eq!(fields, [("KAMPUS_STORAGE_PUBLIC_URL", "url")]);
}
