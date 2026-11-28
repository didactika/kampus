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

//! The rules of an outline plugin, checked on plugins that keep them and on plugins that break them.

use kampus_module::Module;
use kampus_outline::OutlineDeclaration;
use kampus_outline::conformance::problems;

/// An outline named as its plugin.
const WEEKS: OutlineDeclaration = OutlineDeclaration {
    code: "outline_weeks",
    summary: "One section per week",
    icon: "date_range",
};

/// A plugin with some outlines.
fn plugin(name: &'static str, outlines: &'static [OutlineDeclaration]) -> Module {
    Module {
        name,
        manifest: kampus_common::manifest!(),
        summary: "An outline",
        steps: &[],
        permissions: &[],
        templates: &[],
        plugin_types: &[],
        commands: &[],
        preferences: &[],
        settings: &[],
        terms: &[],
        flags: &[],
        field_kinds: &[],
        page_kinds: &[],
        widgets: &[],
        outlines,
        menu_kinds: &[],
        rule_actions: &[],
        file_areas: &[],
        activity_types: &[],
        notification_types: &[],
        privacy_handlers: &[],
        condition_kinds: &[],
        progress_rules: &[],
        events: &[],
        listeners: &[],
        jobs: &[],
        schedules: &[],
        routes: None,
        operations: &[],
    }
}

#[test]
fn a_plugin_with_one_outline_named_as_itself_keeps_the_rules() {
    assert_eq!(problems(&plugin("outline_weeks", &[WEEKS])), Vec::<String>::new());
}

#[test]
fn a_plugin_breaks_the_rules_with_no_outline_two_or_a_wrong_name() {
    for (name, outlines) in [
        ("outline_weeks", &[][..]),
        ("outline_weeks", &[WEEKS, WEEKS][..]),
        ("outline_days", &[WEEKS][..]),
        ("weeks", &[WEEKS][..]),
    ] {
        assert!(!problems(&plugin(name, outlines)).is_empty(), "{name}");
    }
}
