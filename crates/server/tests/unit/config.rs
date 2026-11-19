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

//! Tests of `kampus-server`: config.

use std::collections::HashMap;

use kampus_server::config::LogFormat;
use kampus_server::{Config, Role};

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
    ("KAMPUS_PUBLIC_URL", "https://campus.example"),
    ("KAMPUS_STORAGE_URL", "file:///var/lib/kampus"),
];

#[test]
fn defaults_apply_when_only_the_required_values_are_set() {
    let config = load(BASE, None).unwrap();
    assert_eq!(config.role, Role::All);
    assert_eq!(config.http_address.to_string(), "0.0.0.0:8080");
    assert_eq!(config.database_prefix.as_str(), "kmp_");
    assert_eq!(config.database_pool_size, 10);
    assert!(!config.upgrade_on_start);
    assert_eq!(config.log_format, LogFormat::Json);
}

#[test]
fn every_problem_is_reported_at_once() {
    let error = load(
        &[
            ("KAMPUS_DATABASE_PREFIX", "BAD"),
            ("KAMPUS_DATABASE_POOL_SIZE", "0"),
            ("KAMPUS_ROLE", "everything"),
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
            ("KAMPUS_ROLE", "one_of"),
            ("KAMPUS_PUBLIC_URL", "required"),
            ("KAMPUS_DATABASE_URL", "required"),
            ("KAMPUS_DATABASE_PREFIX", "format"),
            ("KAMPUS_DATABASE_POOL_SIZE", "range"),
            ("KAMPUS_STORAGE_URL", "required"),
        ]
    );
}

#[test]
fn a_scheduler_needs_neither_the_public_address_nor_the_storage() {
    let config = load(
        &[
            ("KAMPUS_ROLE", "scheduler"),
            ("KAMPUS_DATABASE_URL", "postgres://db/kampus"),
        ],
        None,
    )
    .unwrap();
    assert_eq!(config.role, Role::Scheduler);
    assert!(config.public_url.is_none());
    assert!(config.storage_url.is_none());
}

#[test]
fn variables_take_precedence_over_the_file() {
    let file = "database_url = \"postgres://file/kampus\"\npublic_url = \"https://file.example\"\ndatabase_pool_size = 4\nupgrade_on_start = true\nstorage_url = \"s3://kampus/files\"\nstorage_s3_endpoint = \"http://s3:8333\"\n";
    let config = load(&[("KAMPUS_PUBLIC_URL", "https://env.example")], Some(file)).unwrap();
    assert_eq!(config.database_url, "postgres://file/kampus");
    assert_eq!(config.public_url.as_deref(), Some("https://env.example"));
    assert_eq!(config.database_pool_size, 4);
    assert!(config.upgrade_on_start);
    assert_eq!(config.storage_url.as_deref(), Some("s3://kampus/files"));
    assert_eq!(
        config.storage_options.get("s3_endpoint").map(String::as_str),
        Some("http://s3:8333")
    );
}

#[test]
fn unknown_keys_in_the_file_are_refused() {
    let error = load(BASE, Some("databse_url = \"x\"\n")).unwrap_err();
    assert_eq!(error.fields()[0].field, "KAMPUS_CONFIG_FILE.databse_url");
}
