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
// @package   kampus-macros
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Macros of Kampus that read the files of the crate that uses them, at compile time: `version.toml` of
//! the platform and of each component, and the icon of a component.
//!
//! Use them through `kampus-common`, which re-exports them.

use std::path::Path;

use proc_macro::TokenStream;

/// Where a component keeps its icon unless its `version.toml` says otherwise.
const ICON: &str = "assets/icon.svg";

/// Writes the manifest of the calling component from its `version.toml`, next to its `Cargo.toml`, the
/// same file the platform has at the root: the version (which must be the package version), and the
/// optional maturity, supported Kampus releases, date, icon and requirements. Name, description and
/// licence come from `Cargo.toml`; the icon is `assets/icon.svg` when the file exists and `version.toml`
/// names no other.
#[proc_macro]
pub fn manifest(input: TokenStream) -> TokenStream {
    let code = if input.is_empty() {
        read_manifest()
    } else {
        Err(
            "kampus_common::manifest!() takes no fields: they belong in the component's version.toml"
                .to_owned(),
        )
    };
    code.unwrap_or_else(|message| format!("compile_error!({message:?})"))
        .parse()
        .unwrap_or_default()
}

/// The code of a component's manifest, or why its `version.toml` cannot be read.
fn read_manifest() -> Result<String, String> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?;
    let path = Path::new(&dir).join(VERSION_FILE);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let table: toml::Table = text.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let component = table
        .get("component")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("{}: the [component] table is missing", path.display()))?;
    let text_of = |name: &str| -> Result<Option<String>, String> {
        match component.get(name) {
            None => Ok(None),
            Some(toml::Value::String(value)) => Ok(Some(value.clone())),
            Some(_) => Err(format!("{}: component.{name} is not text", path.display())),
        }
    };
    let version =
        text_of("version")?.ok_or_else(|| format!("{}: component.version is missing", path.display()))?;
    let package = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    if version != package {
        return Err(format!(
            "{}: component.version {version} differs from the package version {package}",
            path.display()
        ));
    }
    let maturity = match text_of("maturity")? {
        None => "Stable".to_owned(),
        Some(text) => maturity_variant(&text).map_err(|m| format!("{}: component.{m}", path.display()))?,
    };
    let kampus = text_of("kampus")?.map_or_else(|| "None".to_owned(), |k| format!("Some({k:?})"));
    let date = text_of("date")?;
    if let Some(date) = &date {
        check_date(date).map_err(|m| format!("{}: component.{m}", path.display()))?;
    }
    let icon_path = Path::new(&dir).join(text_of("icon")?.unwrap_or_else(|| ICON.to_owned()));
    let icon = if icon_path.is_file() {
        format!("Some(include_str!({:?}))", icon_path.display().to_string())
    } else if component.contains_key("icon") {
        return Err(format!(
            "{}: the icon {} does not exist",
            path.display(),
            icon_path.display()
        ));
    } else {
        "None".to_owned()
    };
    let mut requires = Vec::new();
    if let Some(table) = table.get("requires") {
        let table = table
            .as_table()
            .ok_or_else(|| format!("{}: [requires] is not a table", path.display()))?;
        for (name, range) in table {
            let range = range
                .as_str()
                .ok_or_else(|| format!("{}: requires.{name} is not text", path.display()))?;
            requires.push(format!(
                "::kampus_common::component::Requirement {{ component: {name:?}, version: {range:?} }}"
            ));
        }
    }
    Ok(format!(
        "{{
            const _: &str = include_str!({file:?});
            ::kampus_common::component::Manifest {{
                package: env!(\"CARGO_PKG_NAME\"),
                version: {version:?},
                summary: env!(\"CARGO_PKG_DESCRIPTION\"),
                license: env!(\"CARGO_PKG_LICENSE\"),
                maturity: ::kampus_common::component::Maturity::{maturity},
                kampus: {kampus},
                date: {date},
                requires: &[{requires}],
                icon: {icon},
            }}
        }}",
        file = path.display().to_string(),
        date = date.map_or_else(|| "None".to_owned(), |d| format!("Some({d:?})")),
        requires = requires.join(", "),
    ))
}

/// The variant of a maturity, or what is wrong with it.
fn maturity_variant(text: &str) -> Result<String, String> {
    match text {
        "alpha" => Ok("Alpha".to_owned()),
        "beta" => Ok("Beta".to_owned()),
        "rc" => Ok("ReleaseCandidate".to_owned()),
        "stable" => Ok("Stable".to_owned()),
        other => Err(format!("maturity {other} is not alpha, beta, rc or stable")),
    }
}

/// Checks a `YYYY-MM-DD` date, or says what is wrong with it.
fn check_date(date: &str) -> Result<(), String> {
    let valid = date.len() == 10
        && date.char_indices().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    if valid {
        Ok(())
    } else {
        Err(format!("date {date} is not YYYY-MM-DD"))
    }
}

/// Where the platform keeps its version, at the root of the workspace.
const VERSION_FILE: &str = "version.toml";

/// Reads the platform's `version.toml` at compile time, from the first folder above the calling crate that
/// has one, checks it, and writes the `Platform` it describes. Its version must be the version of the
/// calling package, which every core crate takes from the workspace.
#[proc_macro]
pub fn platform(_input: TokenStream) -> TokenStream {
    read_platform()
        .unwrap_or_else(|message| format!("compile_error!({message:?})"))
        .parse()
        .unwrap_or_default()
}

/// The code of the platform, or why `version.toml` cannot be read.
fn read_platform() -> Result<String, String> {
    let start = std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?;
    let path = Path::new(&start)
        .ancestors()
        .map(|dir| dir.join(VERSION_FILE))
        .find(|path| path.is_file())
        .ok_or_else(|| format!("{VERSION_FILE} not found above {start}"))?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let table: toml::Table = text.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let platform = table
        .get("platform")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("{}: the [platform] table is missing", path.display()))?;
    let field = |name: &str| {
        platform
            .get(name)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("{}: platform.{name} is missing or not text", path.display()))
    };
    let (name, version, maturity, date, upgrade_from) = (
        field("name")?,
        field("version")?,
        field("maturity")?,
        field("date")?,
        field("upgrade_from")?,
    );
    let package = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    if version != package {
        return Err(format!(
            "{}: platform.version {version} differs from the workspace version {package}",
            path.display()
        ));
    }
    let maturity = maturity_variant(&maturity).map_err(|m| format!("{}: platform.{m}", path.display()))?;
    let is_date = date.len() == 10
        && date.char_indices().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    if !is_date {
        return Err(format!(
            "{}: platform.date {date} is not YYYY-MM-DD",
            path.display()
        ));
    }
    Ok(format!(
        "{{
            const _: &str = include_str!({file:?});
            ::kampus_common::platform::Platform {{
                name: {name:?},
                version: {version:?},
                maturity: ::kampus_common::component::Maturity::{maturity},
                date: {date:?},
                upgrade_from: {upgrade_from:?},
            }}
        }}",
        file = path.display().to_string()
    ))
}
