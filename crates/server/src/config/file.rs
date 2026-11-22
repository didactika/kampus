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

//! The configuration file of an installation, `config.toml`: its sections and keys, how it is read and
//! written, and the environment variable of each key, which takes precedence over the file.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::PathBuf;

use kampus_common::FieldError;

/// One key of the configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// The section and the key, as written in the file (`database.url`).
    pub path: &'static str,
    /// What it sets, for the installer and the documentation.
    pub summary: &'static str,
    /// Whether it holds a secret, never printed.
    pub secret: bool,
}

/// A key that is not secret.
const fn key(path: &'static str, summary: &'static str) -> Key {
    Key {
        path,
        summary,
        secret: false,
    }
}

/// A key that holds a secret.
const fn secret(path: &'static str, summary: &'static str) -> Key {
    Key {
        path,
        summary,
        secret: true,
    }
}

/// Every key of the configuration, by section, in the order the file is written.
pub const KEYS: &[Key] = &[
    key(
        "site.url",
        "The public address of the site, used in links and mail",
    ),
    key(
        "server.role",
        "What this process runs: server, worker, scheduler or all",
    ),
    key("server.http_address", "The address the HTTP listener binds to"),
    key(
        "server.upgrade_on_start",
        "Whether the process applies pending upgrades when it starts",
    ),
    secret("database.url", "The connection to PostgreSQL"),
    key(
        "database.prefix",
        "The prefix of every schema of the installation",
    ),
    key("database.pool_size", "The connections of each process"),
    key(
        "storage.url",
        "Where files are kept: file:///path or s3://bucket/prefix",
    ),
    key("storage.s3_endpoint", "The endpoint of an S3-compatible store"),
    key("storage.s3_region", "The region of the S3 store"),
    secret("storage.s3_access_key_id", "The access key of the S3 store"),
    secret("storage.s3_secret_access_key", "The secret key of the S3 store"),
    key(
        "storage.public_url",
        "The address browsers reach the S3 store at; it then serves files that cannot run itself",
    ),
    secret(
        "cache.url",
        "The shared cache: redis:// or valkey://; in-process when absent",
    ),
    secret(
        "security.secret_key",
        "The key that seals secrets, 32 bytes in base64",
    ),
    key(
        "tenancy.mode",
        "single, one campus; or multi, a campus per domain",
    ),
    key(
        "tenancy.base_domain",
        "With multi, the domain under which each campus has its own",
    ),
    key("log.level", "error, warn, info, debug or trace"),
    key("log.format", "json or text"),
];

/// The environment variable of a key: `database.url` is `KAMPUS_DATABASE_URL`.
#[must_use]
pub fn variable(path: &str) -> String {
    format!("KAMPUS_{}", path.replace('.', "_").to_ascii_uppercase())
}

/// The variable that names another configuration file.
pub const FILE_VARIABLE: &str = "KAMPUS_CONFIG_FILE";

/// Where the configuration file is: `KAMPUS_CONFIG_FILE`, or `config.toml` in the working directory.
#[must_use]
pub fn path() -> PathBuf {
    std::env::var(FILE_VARIABLE).map_or_else(|_| PathBuf::from("config.toml"), PathBuf::from)
}

/// The values of a configuration file, by key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Values(BTreeMap<String, String>);

impl Values {
    /// Reads the text of a file; unknown keys and values that are not text, numbers or booleans are
    /// reported as fields of `KAMPUS_CONFIG_FILE`.
    #[must_use]
    pub fn parse(text: &str) -> (Self, Vec<FieldError>) {
        let mut fields = Vec::new();
        let Ok(table) = text.parse::<toml::Table>() else {
            fields.push(FieldError::new(FILE_VARIABLE, "toml"));
            return (Self::default(), fields);
        };
        let mut values = BTreeMap::new();
        for (section, content) in table {
            let Some(content) = content.as_table() else {
                fields.push(FieldError::new(format!("{FILE_VARIABLE}.{section}"), "section"));
                continue;
            };
            for (name, value) in content {
                let path = format!("{section}.{name}");
                if !KEYS.iter().any(|k| k.path == path) {
                    fields.push(FieldError::new(format!("{FILE_VARIABLE}.{path}"), "unknown"));
                    continue;
                }
                let text = match value {
                    toml::Value::String(text) => text.clone(),
                    toml::Value::Integer(number) => number.to_string(),
                    toml::Value::Boolean(flag) => flag.to_string(),
                    _ => {
                        fields.push(FieldError::new(format!("{FILE_VARIABLE}.{path}"), "scalar"));
                        continue;
                    }
                };
                values.insert(path, text);
            }
        }
        (Self(values), fields)
    }

    /// The value of a key.
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&str> {
        self.0.get(path).map(String::as_str)
    }

    /// Sets the value of a key; an empty value removes it.
    pub fn set(&mut self, path: &str, value: &str) {
        if value.is_empty() {
            self.0.remove(path);
        } else {
            self.0.insert(path.to_owned(), value.to_owned());
        }
    }

    /// The file, one table per section, in the order of [`KEYS`].
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        let mut section = "";
        for key in KEYS {
            let Some(value) = self.get(key.path) else {
                continue;
            };
            let (current, name) = key.path.split_once('.').unwrap_or(("", key.path));
            if current != section {
                if !out.is_empty() {
                    out.push('\n');
                }
                writeln!(out, "[{current}]").ok();
                section = current;
            }
            let literal = match value {
                "true" | "false" => value.to_owned(),
                _ if key.path == "database.pool_size" && value.parse::<u32>().is_ok() => value.to_owned(),
                _ => quoted(value),
            };
            writeln!(out, "{name} = {literal}").ok();
        }
        out
    }
}

/// A TOML basic string.
fn quoted(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => drop(write!(out, "\\u{:04X}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
