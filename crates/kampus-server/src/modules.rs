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

//! The composition of modules: the upgrade steps of every module, in the order of the module table of the
//! architecture. `kampus-db` adds its own steps first.

use kampus_db::ModuleSteps;

/// The upgrade steps of every module of this build, in dependency order.
pub const MODULES: &[ModuleSteps] = &[];

/// The oldest release still supported: columns may be dropped only when it no longer reads them (DEC-081).
pub const OLDEST_SUPPORTED: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::{MODULES, OLDEST_SUPPORTED};

    #[test]
    fn the_schema_of_every_module_passes_the_linter() {
        let mut modules = vec![kampus_db::OWN_STEPS];
        modules.extend_from_slice(MODULES);
        let findings = kampus_db::lint::lint(&modules, OLDEST_SUPPORTED);
        assert!(
            findings.is_empty(),
            "{}",
            findings
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
