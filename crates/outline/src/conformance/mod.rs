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
// @package   kampus-outline
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The rules every outline plugin keeps, which its own tests and the server's run: one outline, named as its plugin,
//! with an icon and a summary, and nothing else that a plugin of another type offers.

use kampus_module::Module;

/// What is wrong with an outline plugin; nothing when it keeps every rule.
#[must_use]
pub fn problems(plugin: &Module) -> Vec<String> {
    let mut problems = Vec::new();
    if !plugin.name.starts_with(&format!("{}_", crate::TYPE)) {
        problems.push(format!("{} is not named {}_<name>", plugin.name, crate::TYPE));
    }
    let [outline] = plugin.outlines else {
        problems.push(format!(
            "{} declares one outline, not {}",
            plugin.name,
            plugin.outlines.len()
        ));
        return problems;
    };
    if outline.code != plugin.name {
        problems.push(format!(
            "the outline of {} has the code {}, not the name of its plugin",
            plugin.name, outline.code
        ));
    }
    if outline.icon.is_empty() || outline.summary.is_empty() {
        problems.push(format!("{} has no icon or no summary", outline.code));
    }
    if !plugin.widgets.is_empty() {
        problems.push(format!(
            "{} offers widgets, which only widget plugins do",
            plugin.name
        ));
    }
    problems
}
