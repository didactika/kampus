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

//! The rules of a widget plugin, checked on plugins that keep them and on plugins that break them.

use kampus_module::Module;
use kampus_widget::conformance::problems;
use kampus_widget::{Cells, Resize, WidgetDeclaration, WidgetSize};

/// A size of the clock on the home.
const SIZE: WidgetSize = WidgetSize {
    page_kind: "customise.dashboard",
    min: Cells::new(2, 2),
    default: Cells::new(3, 2),
    max: Cells::new(6, 4),
    resize: Resize::Both,
};

/// A widget named as its plugin, with an icon and a size.
const CLOCK: WidgetDeclaration = WidgetDeclaration {
    code: "widget_clock",
    summary: "A clock",
    icon: "schedule",
    single: false,
    required: false,
    sizes: &[SIZE],
    settings: &[],
};

/// A plugin with some widgets.
fn plugin(name: &'static str, widgets: &'static [WidgetDeclaration]) -> Module {
    Module {
        name,
        manifest: kampus_common::manifest!(),
        summary: "A clock",
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
        widgets,
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
fn a_plugin_with_one_widget_named_as_itself_keeps_the_rules() {
    assert_eq!(problems(&plugin("widget_clock", &[CLOCK])), Vec::<String>::new());
}

#[test]
fn a_plugin_breaks_the_rules_with_no_widget_two_a_wrong_name_or_no_size() {
    const NO_SIZE: WidgetDeclaration = WidgetDeclaration { sizes: &[], ..CLOCK };
    const REQUIRED_TWICE: WidgetDeclaration = WidgetDeclaration {
        required: true,
        ..CLOCK
    };
    const WIDE: WidgetDeclaration = WidgetDeclaration {
        sizes: &[WidgetSize {
            max: Cells::new(13, 4),
            ..SIZE
        }],
        ..CLOCK
    };
    for (name, widgets) in [
        ("widget_clock", &[][..]),
        ("widget_clock", &[CLOCK, CLOCK][..]),
        ("widget_watch", &[CLOCK][..]),
        ("clock", &[CLOCK][..]),
        ("widget_clock", &[NO_SIZE][..]),
        ("widget_clock", &[WIDE][..]),
        ("widget_clock", &[REQUIRED_TWICE][..]),
    ] {
        assert!(!problems(&plugin(name, widgets)).is_empty(), "{name}");
    }
}
