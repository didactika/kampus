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

//! What the operator sees of the components: each one in the build and in the installation, with its
//! state and what it lacks, and the history of runs.

use kampus_common::{PLATFORM, PLATFORM_VERSION, Result};
use kampus_db::Run;
use serde::Serialize;

use super::read_only;
use crate::config::Config;
use crate::wiring::components;

/// A component in the build, in the installation, or in both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentStatus {
    /// Its name.
    pub name: String,
    /// `module`, `plugin` or `adapter`.
    pub kind: String,
    /// `core` or `external`.
    pub origin: String,
    /// The version in the build.
    pub build_version: Option<String>,
    /// The version installed.
    pub installed_version: Option<String>,
    /// `current`, `to install`, `to upgrade`, `newer installed` or `not in the build`.
    pub state: &'static str,
    /// Its maturity.
    pub maturity: String,
    /// The Kampus releases it supports.
    pub kampus: Option<String>,
    /// What it does.
    pub summary: String,
    /// Whether it has its own icon.
    pub icon: bool,
    /// What is wrong or missing.
    pub problems: Vec<String>,
}

/// Every component of the build and of the installation.
///
/// # Errors
///
/// `unavailable` when the database cannot be read.
pub async fn statuses(config: &Config) -> Result<Vec<ComponentStatus>> {
    let db = read_only(config).await?;
    let installed = db.installed_components().await;
    db.close().await;
    let installed = installed?;
    let problems = components::problems(PLATFORM_VERSION);
    let mut statuses: Vec<ComponentStatus> = components::build()
        .into_iter()
        .map(|c| {
            let name = c.name();
            let recorded = installed
                .iter()
                .find(|i| i.name == name)
                .map(|i| i.version.clone());
            let state = match &recorded {
                None => "to install",
                Some(version) if version == c.manifest.version => "current",
                Some(version) if newer(version, c.manifest.version) => "newer installed",
                Some(_) => "to upgrade",
            };
            ComponentStatus {
                problems: problems
                    .iter()
                    .filter(|p| p.component == name)
                    .map(ToString::to_string)
                    .collect(),
                name,
                kind: c.kind.to_owned(),
                origin: if c.external { "external" } else { "core" }.to_owned(),
                build_version: Some(c.manifest.version.to_owned()),
                installed_version: recorded,
                state,
                maturity: c.manifest.maturity.as_str().to_owned(),
                kampus: c.manifest.kampus.map(str::to_owned),
                summary: c.manifest.summary.to_owned(),
                icon: c.manifest.icon.is_some(),
            }
        })
        .collect();
    let platform_installed = installed
        .iter()
        .find(|i| i.name == components::PLATFORM_COMPONENT)
        .map(|i| i.version.clone());
    statuses.insert(
        0,
        ComponentStatus {
            name: components::PLATFORM_COMPONENT.to_owned(),
            kind: "platform".to_owned(),
            origin: "core".to_owned(),
            build_version: Some(PLATFORM.version.to_owned()),
            state: match &platform_installed {
                None => "to install",
                Some(v) if v == PLATFORM.version => "current",
                Some(v) if newer(v, PLATFORM.version) => "newer installed",
                Some(_) => "to upgrade",
            },
            installed_version: platform_installed,
            maturity: PLATFORM.maturity.as_str().to_owned(),
            kampus: None,
            summary: format!(
                "{} {}, released {}",
                PLATFORM.name, PLATFORM.version, PLATFORM.date
            ),
            icon: false,
            problems: Vec::new(),
        },
    );
    let built: Vec<String> = statuses.iter().map(|s| s.name.clone()).collect();
    for gone in installed.iter().filter(|i| !built.contains(&i.name)) {
        statuses.push(ComponentStatus {
            name: gone.name.clone(),
            kind: gone.kind.clone(),
            origin: gone.origin.clone(),
            build_version: None,
            installed_version: Some(gone.version.clone()),
            state: "not in the build",
            maturity: gone.maturity.clone(),
            kampus: None,
            summary: String::new(),
            icon: false,
            problems: vec![format!(
                "installed but not in this build: add it back, or run `kampus-server uninstall {}`",
                gone.name
            )],
        });
    }
    Ok(statuses)
}

/// The latest runs, newest first.
///
/// # Errors
///
/// `unavailable` when the database cannot be read.
pub async fn history(config: &Config, limit: i64) -> Result<Vec<Run>> {
    let db = read_only(config).await?;
    let runs = db.upgrade_history(limit).await;
    db.close().await;
    runs
}

/// Whether version `a` is newer than `b`.
fn newer(a: &str, b: &str) -> bool {
    matches!((semver::Version::parse(a), semver::Version::parse(b)), (Ok(a), Ok(b)) if a > b)
}
