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

//! Tests of `kampus-server`: wiring modules.

use kampus_server::wiring::modules::{OLDEST_SUPPORTED, SCHEMAS};

#[test]
fn the_schema_of_every_module_passes_the_linter() {
    let mut modules = vec![kampus_db::OWN_STEPS];
    modules.extend_from_slice(&SCHEMAS);
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

#[test]
fn every_widget_declares_sizes_that_fit_the_pages_it_goes_on() {
    use kampus_server::wiring::registry::MODULES;
    let pages: Vec<_> = MODULES.iter().flat_map(|m| m.page_kinds.iter()).collect();
    let mut problems = Vec::new();
    for widget in MODULES.iter().flat_map(|m| m.widgets.iter()) {
        if widget.sizes.is_empty() {
            problems.push(format!("{} declares no size, so it goes on no page", widget.code));
        }
        for size in widget.sizes {
            match pages.iter().find(|p| p.code == size.page_kind) {
                None => problems.push(format!(
                    "{} has a size for {}, which no module declares",
                    widget.code, size.page_kind
                )),
                Some(page) => {
                    if let Some(problem) = size.problem(page.grid) {
                        problems.push(format!("{} on {}: {problem}", widget.code, size.page_kind));
                    }
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn only_widget_plugins_declare_widgets_one_each_named_as_their_plugin() {
    use kampus_server::wiring::registry::MODULES;
    let mut problems = Vec::new();
    for module in MODULES {
        let plugin = module.name.starts_with("widget_");
        match (plugin, module.widgets) {
            (false, []) => {}
            (false, _) => problems.push(format!(
                "{} declares widgets, but only widget plugins do",
                module.name
            )),
            (true, [widget]) if widget.code == module.name => {}
            (true, _) => problems.push(format!(
                "{} declares one widget, whose code is its name",
                module.name
            )),
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
