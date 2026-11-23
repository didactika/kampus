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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The kinds of menu, the page kind of the home and the widgets customisation declares.

use kampus_module::{
    Cells, Grid, MenuKindDeclaration, PageKindDeclaration, Resize, ValueKind, WidgetDeclaration,
    WidgetSetting, WidgetSize,
};

/// The page kind of the home of a site: what every person sees first, arranged with widgets per scope.
pub const DASHBOARD: &str = "customise.dashboard";

/// The page of a course, which the courses module declares.
const COURSE_PAGE: &str = "courses.course";

/// The page kinds of this module.
pub const PAGE_KINDS: &[PageKindDeclaration] = &[PageKindDeclaration {
    code: DASHBOARD,
    summary: "The home of the site",
    grid: Grid::STANDARD,
    personal: true,
}];

/// The kinds of menu of every site.
pub const MENU_KINDS: &[MenuKindDeclaration] = &[
    MenuKindDeclaration {
        code: "customise.main",
        summary: "The main navigation of the site",
    },
    MenuKindDeclaration {
        code: "customise.footer",
        summary: "The links at the foot of every page",
    },
];

/// The widgets of this module.
pub const WIDGETS: &[WidgetDeclaration] = &[
    WidgetDeclaration {
        code: "customise.welcome",
        summary: "A greeting in the colours of the brand, with the figures of the reader",
        sizes: &[WidgetSize {
            page_kind: DASHBOARD,
            min: Cells::new(6, 3),
            default: Cells::new(12, 4),
            max: Cells::new(12, 6),
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
                key: "figures",
                kind: ValueKind::Boolean { default: true },
            },
        ],
    },
    WidgetDeclaration {
        code: "customise.text",
        summary: "A heading and a text, on any page",
        sizes: &[
            WidgetSize {
                page_kind: DASHBOARD,
                min: Cells::new(2, 2),
                default: Cells::new(4, 3),
                max: Cells::new(12, 12),
                resize: Resize::Both,
            },
            WidgetSize {
                page_kind: COURSE_PAGE,
                min: Cells::new(2, 2),
                default: Cells::new(4, 3),
                max: Cells::new(12, 12),
                resize: Resize::Both,
            },
        ],
        settings: &[
            WidgetSetting {
                key: "heading",
                kind: ValueKind::Text {
                    max: 100,
                    default: "",
                },
            },
            WidgetSetting {
                key: "text",
                kind: ValueKind::Text {
                    max: 2000,
                    default: "",
                },
            },
        ],
    },
];
