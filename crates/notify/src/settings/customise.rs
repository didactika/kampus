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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What notifications offer to customisation: the inbox as a widget of the home.

use kampus_module::{Cells, Resize, ValueKind, WidgetDeclaration, WidgetSetting, WidgetSize};

/// The widgets of this module.
pub const WIDGETS: &[WidgetDeclaration] = &[WidgetDeclaration {
    code: "notify.inbox",
    summary: "The latest notifications of the reader",
    sizes: &[WidgetSize {
        page_kind: "customise.dashboard",
        min: Cells::new(3, 3),
        default: Cells::new(4, 5),
        max: Cells::new(6, 10),
        resize: Resize::Both,
    }],
    settings: &[WidgetSetting {
        key: "limit",
        kind: ValueKind::Integer {
            min: 3,
            max: 20,
            default: 5,
        },
    }],
}];
