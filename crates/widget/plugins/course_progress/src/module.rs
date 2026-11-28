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
// @package   kampus-widget-course-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The descriptor of the progress widget, which the server reads to install it.

use kampus_module::Module;

/// The name of the plugin, which is also the code of its widget.
pub const NAME: &str = "widget_course_progress";

/// The plugin as the server reads it.
pub const MODULE: Module = Module {
    name: NAME,
    manifest: kampus_common::manifest!(),
    summary: "The progress of the reader in the course",
    steps: crate::db::STEPS,
    permissions: crate::permissions::PERMISSIONS,
    templates: crate::permissions::TEMPLATES,
    plugin_types: &[],
    commands: &[],
    preferences: &[],
    settings: &[],
    terms: &[],
    flags: &[],
    field_kinds: &[],
    page_kinds: &[],
    widgets: crate::settings::customise::WIDGETS,
    outlines: &[],
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
};
