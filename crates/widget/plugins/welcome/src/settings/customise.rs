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
// @package   kampus-widget-welcome
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The widget this plugin offers: its icon, the sizes it takes on each page kind and its settings.

use kampus_widget::{Band, Cells, Resize, ValueKind, WidgetDeclaration, WidgetSetting, WidgetSize};

/// The home, which customise declares.
const DASHBOARD: &str = "customise.dashboard";

/// The widget of this plugin.
pub const WIDGETS: &[WidgetDeclaration] = &[WidgetDeclaration {
    code: crate::module::NAME,
    summary: "A greeting in the colours of the brand, with the figures of the reader when it is tall enough",
    icon: "home",
    single: true,
    required: false,
    sizes: &[WidgetSize {
        page_kind: DASHBOARD,
        min: Cells::new(6, 3),
        default: Cells::new(8, 5),
        max: Cells::new(12, 6),
        resize: Resize::Both,
        // Up to two thirds of the page the greeting wraps and the figures need up to six rows; wider, four hold all.
        bands: &[Band::new((6, 8), (3, 6)), Band::new((9, 12), (3, 4))],
    }],
    settings: &[WidgetSetting {
        key: "heading",
        kind: ValueKind::Text {
            max: 100,
            default: "",
        },
    }],
}];
