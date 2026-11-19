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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What every component of a build says about itself, and the rules a set of components must follow.
//!
//! A component is a core module, a plugin of a type a core component defines, or an adapter of a port.
//! Its name comes from its package: `kampus-identity` is `identity`, `kampus-cache-redis` is
//! `cache_redis`. Plugins and adapters carry their type as prefix and live in the folder of the component
//! that defines the type.

mod check;

use std::fmt;

pub use check::{Problem, Registered, check};

/// How ready a component is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Maturity {
    /// Under construction; anything may change.
    Alpha,
    /// Complete, still being tested.
    Beta,
    /// A candidate for a stable release.
    ReleaseCandidate,
    /// Stable.
    Stable,
}

impl Maturity {
    /// The maturity as stored and shown.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::ReleaseCandidate => "rc",
            Self::Stable => "stable",
        }
    }
}

impl fmt::Display for Maturity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Another component this one needs, with the versions it works with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirement {
    /// The name of the component (`access`, `cache_redis`).
    pub component: &'static str,
    /// The versions it works with, as a Cargo version requirement (`>=0.1.0`).
    pub version: &'static str,
}

/// What a component says about itself, read from its `version.toml` by [`crate::manifest!`] when it is built:
/// the version, maturity, supported Kampus releases, date and requirements; the name, description and
/// licence come from `Cargo.toml` and the icon from `assets/icon.svg`. Only the version is required: a
/// component without the rest builds and runs, and `kampus-server components` shows what it lacks.
///
/// ```toml
/// [component]
/// version = "1.2.0"
/// maturity = "beta"
/// kampus = ">=0.1.0, <0.3.0"
///
/// [requires]
/// access = ">=0.1.0"
/// ```
///
/// ```ignore
/// const MANIFEST: kampus_common::component::Manifest = kampus_common::manifest!();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Manifest {
    /// The package (`kampus-identity`).
    pub package: &'static str,
    /// The version, semantic (`1.4.0`).
    pub version: &'static str,
    /// What it does, in one sentence: the package description.
    pub summary: &'static str,
    /// The licence, as an SPDX expression.
    pub license: &'static str,
    /// How ready it is; stable when not said.
    pub maturity: Maturity,
    /// The Kampus releases it works with, as a Cargo version requirement (`>=0.1.0, <0.3.0`); any release
    /// when not said.
    pub kampus: Option<&'static str>,
    /// The date of its release, `YYYY-MM-DD`.
    pub date: Option<&'static str>,
    /// The components it needs.
    pub requires: &'static [Requirement],
    /// Its icon, an SVG document, usually `include_str!("../assets/icon.svg")`; a generic icon when absent.
    pub icon: Option<&'static str>,
}

impl Manifest {
    /// The values of the optional fields when a component does not give them; the macro fills the rest.
    pub const DEFAULTS: Self = Self {
        package: "",
        version: "",
        summary: "",
        license: "",
        maturity: Maturity::Stable,
        kampus: None,
        date: None,
        requires: &[],
        icon: None,
    };

    /// The name of the component: the package without `kampus-`, with `_` for `-`.
    #[must_use]
    pub fn component(&self) -> String {
        self.package
            .strip_prefix("kampus-")
            .unwrap_or(self.package)
            .replace('-', "_")
    }
}
