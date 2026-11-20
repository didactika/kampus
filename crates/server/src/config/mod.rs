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

//! The configuration of a process: `config.toml` plus `KAMPUS_*` environment variables, which take
//! precedence. Each role validates only what it reads, and every problem is reported at once.

pub mod file;

use std::collections::BTreeMap;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

use kampus_common::{Error, FieldError, Result};
use kampus_db::Prefix;
use kampus_identity::SecretKey;

use self::file::Values;

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

    /// Whether the role runs background jobs.
    #[must_use]
    pub const fn runs_jobs(self) -> bool {
        matches!(self, Self::Worker | Self::All)
    }

    /// Whether the role fires schedules.
    #[must_use]
    pub const fn runs_schedules(self) -> bool {
        matches!(self, Self::Scheduler | Self::All)
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

/// How the installation serves campuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tenancy {
    /// One campus, at the site's address.
    Single,
    /// One campus per domain under a base domain (`campus-a.example.org`).
    Multi {
        /// The domain under which each campus has its own.
        base_domain: String,
    },
}

/// The validated configuration of a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The role of the process.
    pub role: Role,
    /// The public address of the site, used in links and mail; set for roles that serve the API.
    pub site_url: Option<String>,
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
    /// The options of the storage adapter (`s3_endpoint` …).
    pub storage_options: BTreeMap<String, String>,
    /// The shared cache (`redis://`, `valkey://`); without it the cache is in-process and one instance runs.
    pub cache_url: Option<String>,
    /// The malware scanner (`clamd://host:port`); without it uploads are kept unscanned.
    pub scan_url: Option<String>,
    /// The key that seals secrets, for roles that run the services.
    pub secret_key: Option<SecretKey>,
    /// How campuses are served.
    pub tenancy: Tenancy,
}

/// The options of the storage adapters that the configuration passes on.
const STORAGE_OPTIONS: &[&str] = &[
    "s3_endpoint",
    "s3_region",
    "s3_access_key_id",
    "s3_secret_access_key",
];

/// Reads values by key, from the environment first, and gathers every problem.
struct Reader<'a, F> {
    /// Looks a variable up.
    env: F,
    /// The values of the file.
    file: &'a Values,
    /// The problems found so far.
    fields: Vec<FieldError>,
}

impl<F: Fn(&str) -> Option<String>> Reader<'_, F> {
    /// The value of a key.
    fn value(&self, path: &str) -> Option<String> {
        (self.env)(&file::variable(path)).or_else(|| self.file.get(path).map(str::to_owned))
    }

    /// Records a problem with a key.
    fn fail(&mut self, path: &str, rule: &str) {
        self.fields.push(FieldError::new(file::variable(path), rule));
    }

    /// The `[server]` section and the site's address.
    fn server(&mut self) -> (Role, Option<String>, SocketAddr, bool) {
        let role = match self.value("server.role") {
            None => Role::All,
            Some(text) => text.parse().unwrap_or_else(|()| {
                self.fail("server.role", "one_of");
                Role::All
            }),
        };
        let site_url = self.value("site.url");
        match &site_url {
            None if role.serves_api() => self.fail("site.url", "required"),
            Some(url) if !(url.starts_with("https://") || url.starts_with("http://")) => {
                self.fail("site.url", "url");
            }
            _ => {}
        }
        let http_address = self
            .value("server.http_address")
            .unwrap_or_else(|| "0.0.0.0:8080".into())
            .parse()
            .unwrap_or_else(|_| {
                self.fail("server.http_address", "socket_address");
                SocketAddr::from(([0, 0, 0, 0], 8080))
            });
        let upgrade_on_start = match self.value("server.upgrade_on_start").as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(_) => {
                self.fail("server.upgrade_on_start", "boolean");
                false
            }
        };
        (role, site_url, http_address, upgrade_on_start)
    }

    /// The `[database]` section.
    fn database(&mut self) -> (String, Prefix, u32) {
        let url = self.value("database.url").unwrap_or_default();
        if url.is_empty() {
            self.fail("database.url", "required");
        } else if !(url.starts_with("postgres://") || url.starts_with("postgresql://")) {
            self.fail("database.url", "scheme");
        }
        let prefix = self
            .value("database.prefix")
            .unwrap_or_else(|| Prefix::DEFAULT.into())
            .parse()
            .unwrap_or_else(|_| {
                self.fail("database.prefix", "format");
                Prefix::default()
            });
        let pool_size = match self.value("database.pool_size").map(|text| text.parse::<u32>()) {
            None => 10,
            Some(Ok(size)) if (1..=1000).contains(&size) => size,
            Some(_) => {
                self.fail("database.pool_size", "range");
                10
            }
        };
        (url, prefix, pool_size)
    }

    /// The `[storage]` and `[cache]` sections.
    fn storage(&mut self, runs_services: bool) -> (Option<String>, BTreeMap<String, String>, Option<String>) {
        let url = self.value("storage.url");
        if runs_services && url.is_none() {
            self.fail("storage.url", "required");
        }
        let options = STORAGE_OPTIONS
            .iter()
            .filter_map(|option| {
                self.value(&format!("storage.{option}"))
                    .map(|v| ((*option).to_owned(), v))
            })
            .collect();
        (url, options, self.value("cache.url"))
    }

    /// The `[security]` and `[tenancy]` sections.
    fn security(&mut self, runs_services: bool) -> (Option<SecretKey>, Tenancy) {
        let secret_key = match self.value("security.secret_key") {
            None => {
                if runs_services {
                    self.fail("security.secret_key", "required");
                }
                None
            }
            Some(text) => SecretKey::from_base64(&text).map_or_else(
                |_| {
                    self.fail("security.secret_key", "base64_32_bytes");
                    None
                },
                Some,
            ),
        };
        let tenancy = match (
            self.value("tenancy.mode").as_deref(),
            self.value("tenancy.base_domain"),
        ) {
            (None | Some("single"), _) => Tenancy::Single,
            (Some("multi"), Some(base_domain)) if base_domain.contains('.') => Tenancy::Multi { base_domain },
            (Some("multi"), _) => {
                self.fail("tenancy.base_domain", "required");
                Tenancy::Single
            }
            (Some(_), _) => {
                self.fail("tenancy.mode", "one_of");
                Tenancy::Single
            }
        };
        (secret_key, tenancy)
    }

    /// The `[log]` section.
    fn log(&mut self) -> (String, LogFormat) {
        let level = self.value("log.level").unwrap_or_else(|| "info".into());
        let format = match self.value("log.format").as_deref() {
            None | Some("json") => LogFormat::Json,
            Some("text") => LogFormat::Text,
            Some(_) => {
                self.fail("log.format", "one_of");
                LogFormat::Json
            }
        };
        (level, format)
    }
}

impl Config {
    /// Reads the configuration from the environment and the configuration file (`KAMPUS_CONFIG_FILE`, or
    /// `config.toml` when it exists).
    ///
    /// # Errors
    ///
    /// A `validation` error listing every missing or invalid value.
    pub fn from_environment() -> Result<Self> {
        let path = file::path();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => Some(text),
            Err(_) if !path.exists() => None,
            Err(error) => {
                return Err(Error::validation(
                    "server.config_file",
                    vec![FieldError::new(file::FILE_VARIABLE, "readable")],
                )
                .with_param("path", path.display().to_string())
                .with_source(error));
            }
        };
        Self::load(|name| std::env::var(name).ok(), text.as_deref())
    }

    /// Builds the configuration from a variable lookup and the text of the file, if any.
    ///
    /// # Errors
    ///
    /// A `validation` error listing every missing or invalid value, named by its variable.
    pub fn load(env: impl Fn(&str) -> Option<String>, text: Option<&str>) -> Result<Self> {
        let (values, fields) = text.map_or_else(|| (Values::default(), Vec::new()), Values::parse);
        let mut reader = Reader {
            env,
            file: &values,
            fields,
        };
        let (role, site_url, http_address, upgrade_on_start) = reader.server();
        let runs_services = matches!(role, Role::Server | Role::Worker | Role::All);
        let (database_url, database_prefix, database_pool_size) = reader.database();
        let (storage_url, storage_options, cache_url) = reader.storage(runs_services);
        let scan_url = reader.value("scan.url");
        let (secret_key, tenancy) = reader.security(runs_services);
        let (log_level, log_format) = reader.log();
        if !reader.fields.is_empty() {
            return Err(
                Error::validation("server.config_invalid", reader.fields).with_param("role", role.as_str())
            );
        }
        Ok(Self {
            role,
            site_url,
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
            scan_url,
            secret_key,
            tenancy,
        })
    }
}
