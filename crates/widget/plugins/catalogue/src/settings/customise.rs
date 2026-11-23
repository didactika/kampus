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
// @package   kampus-widget-catalogue
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The widget this plugin offers: its icon, the sizes it takes on each page kind and its settings.

use kampus_widget::{Cells, Resize, ValueKind, WidgetDeclaration, WidgetSetting, WidgetSize};

/// The home, which customise declares.
const DASHBOARD: &str = "customise.dashboard";

/// The widget of this plugin.
pub const WIDGETS: &[WidgetDeclaration] = &[WidgetDeclaration {
    code: crate::module::NAME,
    summary: "The courses of the campus, to search, filter and open",
    icon: "school",
    single: false,
    required: false,
    sizes: &[WidgetSize {
        page_kind: DASHBOARD,
        min: Cells::new(6, 5),
        default: Cells::new(8, 8),
        max: Cells::new(12, 16),
        resize: Resize::Both,
    }],
    settings: &[
        WidgetSetting {
            key: "heading",
            kind: ValueKind::Text {
                max: 100,
                default: "",
            },
        },
        WidgetSetting {
            key: "view",
            kind: ValueKind::Choice {
                options: &["grid", "list"],
                default: "grid",
            },
        },
        WidgetSetting {
            key: "size",
            kind: ValueKind::Integer {
                min: 4,
                max: 48,
                default: 8,
            },
        },
        WidgetSetting {
            key: "sizes",
            kind: ValueKind::Text { max: 60, default: "" },
        },
    ],
}];
