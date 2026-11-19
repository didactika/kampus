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

//! Installing Kampus: the configuration file first, asked for interactively or taken from the file and
//! the environment, then the database, the first campus and its administrator. An installation that
//! exists is left as it is, except that one without a campus yet, such as one created by an upgrade, gets
//! its first campus and administrator; so a filled `config.toml` pointing at an installed database is all a
//! new server needs.

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kampus_access::{Access, ScopeKind};
use kampus_common::{Error, FieldError, PLATFORM, Result, SystemClock};
use kampus_identity::{FixedPolicies, Identity, MemoryCourier, SecretKey};

use super::{connect, upgrader};
use crate::config::Config;
use crate::config::file::{self, Values};

/// The questions of the installer: the key, its question, and its default when there is one.
const QUESTIONS: &[(&str, &str, Option<&str>)] = &[
    ("site.url", "Public address of the site", None),
    (
        "database.url",
        "PostgreSQL connection (postgres://user:password@host:5432/database)",
        None,
    ),
    (
        "database.prefix",
        "Prefix of the schemas of this installation",
        Some("kmp_"),
    ),
    (
        "storage.url",
        "Where files are kept (file:///path or s3://bucket/prefix)",
        Some("file:///var/lib/kampus/files"),
    ),
    (
        "cache.url",
        "Shared cache (redis:// or valkey://), empty for an in-process cache",
        Some(""),
    ),
    (
        "tenancy.mode",
        "Campuses: single, or multi (one per domain)",
        Some("single"),
    ),
];

/// Variables that give the first administrator without asking.
const ADMIN_EMAIL: &str = "KAMPUS_ADMIN_EMAIL";
/// The username of the first administrator.
const ADMIN_USERNAME: &str = "KAMPUS_ADMIN_USERNAME";
/// The password of the first administrator, read only by the installer.
const ADMIN_PASSWORD: &str = "KAMPUS_ADMIN_PASSWORD";

/// What the installer did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installed {
    /// The database already held the installation: nothing was changed in it.
    Existing {
        /// The configuration file used.
        config: PathBuf,
        /// Whether the installation matches this build; otherwise `kampus-server upgrade` is due.
        current: bool,
    },
    /// A new installation.
    New {
        /// The configuration file used.
        config: PathBuf,
        /// The release installed.
        release: &'static str,
        /// The username of the first administrator.
        administrator: String,
    },
}

/// Talks to the operator on the terminal, and reads the environment it was given.
struct Terminal<'a> {
    /// Whether questions may be asked.
    interactive: bool,
    /// Looks a variable up.
    env: &'a dyn Fn(&str) -> Option<String>,
}

impl Terminal<'_> {
    /// Asks a question, with its default; the default when the answer is empty.
    #[allow(
        clippy::print_stderr,
        reason = "the installer talks to the operator on standard error"
    )]
    fn ask(&self, question: &str, default: Option<&str>) -> Result<Option<String>> {
        if !self.interactive {
            return Ok(default.filter(|d| !d.is_empty()).map(str::to_owned));
        }
        match default {
            Some(d) if !d.is_empty() => eprint!("{question} [{d}]: "),
            _ => eprint!("{question}: "),
        }
        std::io::stderr().flush().ok();
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| Error::unavailable("server.install_input").with_source(e))?;
        let answer = line.trim();
        Ok(if answer.is_empty() {
            default.filter(|d| !d.is_empty()).map(str::to_owned)
        } else {
            Some(answer.to_owned())
        })
    }

    /// Asks a password twice, without showing it.
    fn password(&self) -> Result<String> {
        if let Some(password) = (self.env)(ADMIN_PASSWORD) {
            return Ok(password);
        }
        if !self.interactive {
            return Err(Error::validation(
                "server.install_incomplete",
                vec![FieldError::new(ADMIN_PASSWORD, "required")],
            ));
        }
        let read = |prompt: &str| {
            rpassword::prompt_password(prompt)
                .map_err(|e| Error::unavailable("server.install_input").with_source(e))
        };
        loop {
            let first = read("Password of the administrator: ")?;
            if first == read("The same password again: ")? {
                return Ok(first);
            }
            Self::say("The passwords differ; again.");
        }
    }

    /// Tells the operator something.
    #[allow(
        clippy::print_stderr,
        reason = "the installer talks to the operator on standard error"
    )]
    fn say(line: &str) {
        eprintln!("{line}");
    }
}

/// Installs Kampus with the configuration file at `path` and the variables `env` looks up, asking for what
/// is missing when `interactive` and the input is a terminal.
///
/// # Errors
///
/// A `validation` error naming every value still missing or invalid, and the errors of the database, of
/// the upgrade and of the first administrator.
pub async fn install(
    path: &Path,
    env: &dyn Fn(&str) -> Option<String>,
    interactive: bool,
) -> Result<Installed> {
    let terminal = Terminal {
        interactive: interactive && std::io::stdin().is_terminal(),
        env,
    };
    let path = path.to_path_buf();
    let mut values = read(&path)?;
    let changed = complete(&terminal, &mut values)?;
    let text = values.render();
    if changed {
        write(&path, &text)?;
        Terminal::say(&format!("Configuration written to {}.", path.display()));
    }
    let config = Config::load(env, Some(&text))?;

    let db = connect(&config).await?;
    let upgrader = upgrader(&db)?;
    let plan = upgrader.plan().await?;
    let access = Access::new(db.clone(), Arc::new(SystemClock));
    if plan.installed && !(plan.is_current() && access.campuses().await?.is_empty()) {
        let current = plan.is_current();
        db.close().await;
        return Ok(Installed::Existing {
            config: path,
            current,
        });
    }
    let username = administrator(&terminal)?;
    if !plan.installed {
        upgrader.apply(true).await?;
    }
    crate::wiring::modules::sync_declarations(&db).await?;
    let platform = match access.platform().await? {
        Some(platform) => platform,
        None => access.create_platform().await?,
    };
    let campus = access.create_scope(platform, ScopeKind::Campus).await?;
    access.install_campus_roles(campus).await?;
    let key = config
        .secret_key
        .clone()
        .ok_or_else(|| Error::internal("server.install_without_key"))?;
    let identity = Identity::new(
        db.clone(),
        access,
        Arc::new(SystemClock),
        Arc::new(FixedPolicies::default()),
        Arc::new(MemoryCourier::default()),
        key,
    );
    identity
        .install_administrator(campus, &username.0, &username.1, &terminal.password()?)
        .await?;
    db.close().await;
    Ok(Installed::New {
        config: path,
        release: PLATFORM.version,
        administrator: username.1,
    })
}

/// The email and username of the first administrator, from the environment or asked.
fn administrator(terminal: &Terminal) -> Result<(String, String)> {
    let email = match (terminal.env)(ADMIN_EMAIL) {
        Some(email) => Some(email),
        None => terminal.ask("Email of the administrator", None)?,
    };
    let Some(email) = email else {
        return Err(Error::validation(
            "server.install_incomplete",
            vec![FieldError::new(ADMIN_EMAIL, "required")],
        ));
    };
    let suggested = email.split('@').next().unwrap_or_default().to_owned();
    let username = match (terminal.env)(ADMIN_USERNAME) {
        Some(username) => username,
        None => terminal
            .ask("Username of the administrator", Some(&suggested))?
            .unwrap_or(suggested),
    };
    Ok((email, username))
}

/// The values of the configuration file, if it exists.
fn read(path: &Path) -> Result<Values> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(Values::default());
    };
    let (values, fields) = Values::parse(&text);
    if fields.is_empty() {
        Ok(values)
    } else {
        Err(Error::validation("server.config_invalid", fields).with_param("path", path.display().to_string()))
    }
}

/// Fills what the file and the environment do not give, asking when allowed; a missing secret key is
/// generated. Returns whether the file changed.
fn complete(terminal: &Terminal, values: &mut Values) -> Result<bool> {
    let given = |values: &Values, key: &str| {
        values.get(key).is_some() || (terminal.env)(&file::variable(key)).is_some()
    };
    let mut changed = false;
    let mut missing = Vec::new();
    for (key, question, default) in QUESTIONS {
        if given(values, key) {
            continue;
        }
        match terminal.ask(question, *default)? {
            Some(answer) => {
                values.set(key, &answer);
                changed = true;
            }
            None if default.is_none() => missing.push(FieldError::new(file::variable(key), "required")),
            None => {}
        }
    }
    if values.get("tenancy.mode") == Some("multi") && !given(values, "tenancy.base_domain") {
        match terminal.ask("Base domain of the campuses (campuses.example.org)", None)? {
            Some(domain) => {
                values.set("tenancy.base_domain", &domain);
                changed = true;
            }
            None => missing.push(FieldError::new(file::variable("tenancy.base_domain"), "required")),
        }
    }
    if !given(values, "security.secret_key") {
        values.set("security.secret_key", &SecretKey::generate()?.to_base64());
        changed = true;
    }
    if missing.is_empty() {
        Ok(changed)
    } else {
        Err(Error::validation("server.install_incomplete", missing))
    }
}

/// Writes the configuration file, readable only by its owner where the system allows it.
fn write(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::unavailable("server.config_write").with_source(e))?;
    }
    std::fs::write(path, text).map_err(|e| Error::unavailable("server.config_write").with_source(e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| Error::unavailable("server.config_write").with_source(e))?;
    }
    Ok(())
}
