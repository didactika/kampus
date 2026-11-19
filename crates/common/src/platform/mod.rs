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

//! The platform itself: what `version.toml`, at the root of the repository, says about the release this
//! build belongs to. The installation records it, so the platform knows when it was upgraded.

use crate::component::Maturity;

/// A release of the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    /// The product.
    pub name: &'static str,
    /// The release, semantic.
    pub version: &'static str,
    /// How ready the release is.
    pub maturity: Maturity,
    /// The date of the release, or of the start of its development.
    pub date: &'static str,
    /// The oldest release an installation can be upgraded from in one run.
    pub upgrade_from: &'static str,
}

/// The release this build belongs to, read from `version.toml` when the build is compiled.
pub const PLATFORM: Platform = kampus_macros::platform!();

#[cfg(test)]
mod tests {
    use super::PLATFORM;

    #[test]
    fn the_platform_is_the_workspace_release() {
        assert_eq!(PLATFORM.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(PLATFORM.name, "Kampus");
    }
}
