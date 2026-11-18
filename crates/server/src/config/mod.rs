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

//! The deployment configuration: a file plus `KAMPUS_*` environment variables, which take
//! precedence. Each role validates only what it reads, and every problem is reported at once.

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

use kampus_common::{Error, FieldError, Result};
use kampus_db::Prefix;

/// The role a process takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Serves the HTTP API.
    Server,
    /// Runs background jobs and listeners.
    Worker,
    /// Runs scheduled tasks.
    Scheduler,
    /// The three roles in one process, for small installations and development.
    All,
}

impl Role {
    /// Whether the role serves the HTTP API, which needs the public address of the campus.
    #[must_use]
    pub const fn serves_api(self) -> bool {
        matches!(self, Self::Server | Self::All)
    }

    /// The name of the role in the configuration.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Worker => "worker",
            Self::Scheduler => "scheduler",
            Self::All => "all",
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(text: &str) -> std::result::Result<Self, Self::Err> {
        match text {
            "server" => Ok(Self::Server),
            "worker" => Ok(Self::Worker),
            "scheduler" => Ok(Self::Scheduler),
            "all" => Ok(Self::All),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How log lines are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// One JSON object per line, for collectors.
    Json,
    /// Human-readable lines, for development.
    Text,
}

/// The validated configuration of a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The role of the process.
    pub role: Role,
    /// The public address of the campus, used in links and mail; set for roles that serve the API.
    pub public_url: Option<String>,
    /// The address the HTTP listener binds to.
    pub http_address: SocketAddr,
    /// The connection to the database.
    pub database_url: String,
    /// The installation prefix of every physical database name.
    pub database_prefix: Prefix,
    /// The connections of the pool of this process.
    pub database_pool_size: u32,
    /// Whether the process applies pending upgrade steps when it starts.
    pub upgrade_on_start: bool,
    /// The log level (`error`, `warn`, `info`, `debug`, `trace`), or a filter directive.
    pub log_level: String,
    /// How log lines are written.
    pub log_format: LogFormat,
    /// The file storage (`file:///path` or `s3://bucket/prefix`), for roles that handle files.
    pub storage_url: Option<String>,
    /// The options of the storage adapter, from `KAMPUS_STORAGE_<OPTION>` (`s3_endpoint` …).
    pub storage_options: std::collections::BTreeMap<String, String>,
    /// The shared cache (`redis://`, `valkey://`); without it the cache is in-process and one instance runs.
    pub cache_url: Option<String>,
}

/// The options of the storage adapters that the configuration passes on.
const STORAGE_OPTIONS: &[&str] = &[
    "s3_endpoint",
    "s3_region",
    "s3_access_key_id",
    "s3_secret_access_key",
];

/// The keys of the configuration, as written in the file; the variable of each is `KAMPUS_<KEY>`.
const KEYS: &[&str] = &[
    "role",
    "public_url",
    "http_address",
    "database_url",
    "database_prefix",
    "database_pool_size",
    "upgrade_on_start",
    "log_level",
    "log_format",
    "storage_url",
    "cache_url",
    "storage_s3_endpoint",
    "storage_s3_region",
    "storage_s3_access_key_id",
    "storage_s3_secret_access_key",
];

impl Config {
    /// Reads the configuration from the environment and the file it names (`KAMPUS_CONFIG_FILE`, or
    /// `kampus.toml` when it exists).
    ///
    /// # Errors
    ///
    /// A `validation` error listing every missing or invalid value.
    pub fn from_environment() -> Result<Self> {
        let path = std::env::var("KAMPUS_CONFIG_FILE").ok();
        let file = match &path {
            Some(path) => Some(std::fs::read_to_string(path).map_err(|error| {
                Error::validation(
                    "server.config_file",
                    vec![FieldError::new("KAMPUS_CONFIG_FILE", "readable")],
                )
                .with_param("path", path.as_str())
                .with_source(error)
            })?),
            None => std::fs::read_to_string("kampus.toml").ok(),
        };
        Self::load(|name| std::env::var(name).ok(), file.as_deref())
    }

    /// Builds the configuration from a variable lookup and the text of the file, if any.
    ///
    /// # Errors
    ///
    /// A `validation` error listing every missing or invalid value, named by its variable.
    pub fn load(env: impl Fn(&str) -> Option<String>, file: Option<&str>) -> Result<Self> {
        let mut fields = Vec::new();
        let from_file = parse_file(file, &mut fields);
        let value = |key: &str| env(&variable(key)).or_else(|| from_file.get(key).cloned());

        let role = match value("role") {
            None => Role::All,
            Some(text) => text.parse().unwrap_or_else(|()| {
                fields.push(FieldError::new(variable("role"), "one_of"));
                Role::All
            }),
        };

        let public_url = value("public_url");
        if role.serves_api() {
            match &public_url {
                None => fields.push(FieldError::new(variable("public_url"), "required")),
                Some(url) if !(url.starts_with("https://") || url.starts_with("http://")) => {
                    fields.push(FieldError::new(variable("public_url"), "url"));
                }
                Some(_) => {}
            }
        }

        let http_address = value("http_address")
            .unwrap_or_else(|| "0.0.0.0:8080".into())
            .parse()
            .unwrap_or_else(|_| {
                fields.push(FieldError::new(variable("http_address"), "socket_address"));
                SocketAddr::from(([0, 0, 0, 0], 8080))
            });

        let database_url = value("database_url").unwrap_or_default();
        if database_url.is_empty() {
            fields.push(FieldError::new(variable("database_url"), "required"));
        } else if !(database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")) {
            fields.push(FieldError::new(variable("database_url"), "scheme"));
        }

        let database_prefix = value("database_prefix")
            .unwrap_or_else(|| Prefix::DEFAULT.into())
            .parse()
            .unwrap_or_else(|_| {
                fields.push(FieldError::new(variable("database_prefix"), "format"));
                Prefix::default()
            });

        let database_pool_size = match value("database_pool_size").map(|text| text.parse::<u32>()) {
            None => 10,
            Some(Ok(size)) if (1..=1000).contains(&size) => size,
            Some(_) => {
                fields.push(FieldError::new(variable("database_pool_size"), "range"));
                10
            }
        };

        let upgrade_on_start = match value("upgrade_on_start").as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(_) => {
                fields.push(FieldError::new(variable("upgrade_on_start"), "boolean"));
                false
            }
        };

        let storage_url = value("storage_url");
        if matches!(role, Role::Server | Role::Worker | Role::All) && storage_url.is_none() {
            fields.push(FieldError::new(variable("storage_url"), "required"));
        }
        let storage_options = STORAGE_OPTIONS
            .iter()
            .filter_map(|option| value(&format!("storage_{option}")).map(|v| ((*option).to_owned(), v)))
            .collect();
        let cache_url = value("cache_url");

        let log_level = value("log_level").unwrap_or_else(|| "info".into());
        let log_format = match value("log_format").as_deref() {
            None | Some("json") => LogFormat::Json,
            Some("text") => LogFormat::Text,
            Some(_) => {
                fields.push(FieldError::new(variable("log_format"), "one_of"));
                LogFormat::Json
            }
        };

        if fields.is_empty() {
            Ok(Self {
                role,
                public_url,
                http_address,
                database_url,
                database_prefix,
                database_pool_size,
                upgrade_on_start,
                log_level,
                log_format,
                storage_url,
                storage_options,
                cache_url,
            })
        } else {
            Err(Error::validation("server.config_invalid", fields).with_param("role", role.as_str()))
        }
    }
}

/// The environment variable of a configuration key.
fn variable(key: &str) -> String {
    format!("KAMPUS_{}", key.to_ascii_uppercase())
}

/// Reads the configuration file: a flat table of the known keys with string, integer or boolean values.
fn parse_file(file: Option<&str>, fields: &mut Vec<FieldError>) -> HashMap<String, String> {
    let Some(text) = file else { return HashMap::new() };
    let Ok(table) = text.parse::<toml::Table>() else {
        fields.push(FieldError::new("KAMPUS_CONFIG_FILE", "toml"));
        return HashMap::new();
    };
    let mut values = HashMap::new();
    for (key, value) in table {
        if !KEYS.contains(&key.as_str()) {
            fields.push(FieldError::new(format!("KAMPUS_CONFIG_FILE.{key}"), "unknown"));
            continue;
        }
        let text = match value {
            toml::Value::String(text) => text,
            toml::Value::Integer(number) => number.to_string(),
            toml::Value::Boolean(flag) => flag.to_string(),
            _ => {
                fields.push(FieldError::new(format!("KAMPUS_CONFIG_FILE.{key}"), "scalar"));
                continue;
            }
        };
        values.insert(key, text);
    }
    values
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Config, LogFormat, Role};

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
}
