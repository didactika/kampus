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
// @package   kampus-widget
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The rules every widget plugin keeps, which its own tests and the server's run: one widget, named as its plugin, with
//! an icon, at least one size that fits the standard grid, settings with unique keys, and, when every page holds it,
//! once and at one size. The sizes are checked again
//! against the grid of each page kind when the declarations are read.

use kampus_module::{Grid, Module};

/// What is wrong with a widget plugin; nothing when it keeps every rule.
#[must_use]
pub fn problems(plugin: &Module) -> Vec<String> {
    let mut problems = Vec::new();
    if !plugin.name.starts_with(&format!("{}_", crate::TYPE)) {
        problems.push(format!("{} is not named {}_<name>", plugin.name, crate::TYPE));
    }
    let [widget] = plugin.widgets else {
        problems.push(format!(
            "{} declares one widget, not {}",
            plugin.name,
            plugin.widgets.len()
        ));
        return problems;
    };
    if widget.code != plugin.name {
        problems.push(format!(
            "the widget of {} has the code {}, not the name of its plugin",
            plugin.name, widget.code
        ));
    }
    if widget.icon.is_empty() {
        problems.push(format!("{} has no icon", widget.code));
    }
    if widget.sizes.is_empty() {
        problems.push(format!("{} declares no size, so it goes on no page", widget.code));
    }
    for size in widget.sizes {
        if let Some(problem) = size.problem(Grid::STANDARD) {
            problems.push(format!("{} on {}: {problem}", widget.code, size.page_kind));
        }
    }
    // A widget every page holds can be neither removed nor resized, so it is single and of one size everywhere.
    if widget.required && !widget.single {
        problems.push(format!("{} is required, so a page holds it once", widget.code));
    }
    if widget.required && widget.sizes.iter().any(|s| s.min != s.max) {
        problems.push(format!("{} is required, so it keeps one size", widget.code));
    }
    for (at, setting) in widget.settings.iter().enumerate() {
        if widget.settings[..at].iter().any(|s| s.key == setting.key) {
            problems.push(format!(
                "{} declares the setting {} twice",
                widget.code, setting.key
            ));
        }
    }
    problems
}
