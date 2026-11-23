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

//! Layouts and menus: the widgets of a page kind placed on its grid, and navigation menus as data, each resolved
//! from the nearest scope that sets one unless a scope above locks its own.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Origin;

/// A widget on the grid of a layout: the rectangle of cells it takes, and its settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Placement {
    /// The widget, by code.
    pub widget: String,
    /// The first column it takes, from 0.
    pub x: u16,
    /// The first row it takes, from 0.
    pub y: u16,
    /// How many columns it takes, within the sizes the widget declares for the page kind.
    pub w: u16,
    /// How many rows it takes, within the sizes the widget declares for the page kind.
    pub h: u16,
    /// Whether the person pinned it where it is; otherwise it rises to the widgets above it, so no row is left empty.
    #[serde(default)]
    pub pinned: bool,
    /// Its settings, each of the kind the widget declares; the defaults fill what is left out.
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
    /// The permission a person needs to see it; everyone who sees the page when absent.
    #[serde(default)]
    pub permission: Option<String>,
}

impl Placement {
    /// Whether it shares a cell with another.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }
}

/// The grid of a page kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PageGrid {
    /// How many columns it has.
    pub columns: u16,
    /// The height of a row, by name: `compact`, `regular` or `tall`.
    pub row: RowHeight,
    /// The height of a row, in CSS pixels.
    pub row_pixels: u16,
}

/// The height of a row of a grid, by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RowHeight {
    /// 48 pixels.
    Compact,
    /// 72 pixels.
    Regular,
    /// 96 pixels.
    Tall,
}

/// A size on a grid, in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Size {
    /// Columns.
    pub w: u16,
    /// Rows.
    pub h: u16,
}

/// How a widget may be resized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Resize {
    /// It keeps its size.
    None,
    /// Only its width changes.
    Horizontal,
    /// Only its height changes.
    Vertical,
    /// Both change.
    Both,
}

/// The sizes a widget takes on a page kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WidgetSizes {
    /// The smallest.
    pub min: Size,
    /// The one it takes when it is placed.
    pub default: Size,
    /// The largest.
    pub max: Size,
    /// How it may be resized between them.
    pub resize: Resize,
    /// The heights it takes at some widths; every height between `min` and `max` at the widths no band holds.
    #[serde(default)]
    pub bands: Vec<SizeBand>,
}

/// The heights a widget takes between two widths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SizeBand {
    /// The narrowest and the widest it applies to, in columns.
    pub widths: [u16; 2],
    /// The lowest and the highest it takes there, in rows.
    pub heights: [u16; 2],
}

impl WidgetSizes {
    /// Whether a width and a height are within them.
    #[must_use]
    pub fn hold(&self, w: u16, h: u16) -> bool {
        let (low, high) = self.heights(w);
        self.min.w <= w && w <= self.max.w && low <= h && h <= high
    }

    /// The lowest and highest heights it takes at a width.
    #[must_use]
    pub fn heights(&self, w: u16) -> (u16, u16) {
        self.bands
            .iter()
            .find(|b| b.widths[0] <= w && w <= b.widths[1])
            .map_or((self.min.h, self.max.h), |b| (b.heights[0], b.heights[1]))
    }
}

/// The layout of a page kind in a scope.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct Layout {
    /// The page kind.
    pub page_kind: String,
    /// The grid its widgets are placed on.
    pub grid: PageGrid,
    /// The widgets the reader sees, in reading order (by row, then by column); the page shows its own arrangement
    /// when the layout comes from its declaration.
    pub placements: Vec<Placement>,
    /// Where the layout came from.
    pub origin: Origin,
    /// Whether the reader may change it here: they may arrange the page and no scope above locks it.
    pub editable: bool,
    /// The widgets the reader may place on the page kind, with their sizes and settings; only when it is editable.
    pub widgets: Vec<AvailableWidget>,
}

/// A widget that may be placed on a page kind.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct AvailableWidget {
    /// Its code.
    pub code: String,
    /// What it shows.
    pub summary: String,
    /// Its icon, by the name the design system gives it.
    pub icon: String,
    /// Whether a page holds it once at most.
    pub single: bool,
    /// Whether every page holds it, at one size and never removed.
    pub required: bool,
    /// The sizes it takes on the page kind.
    pub sizes: WidgetSizes,
    /// Its settings as declared: key, kind and default.
    pub settings: Vec<Value>,
}

/// An item of a navigation menu, with the items under it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MenuItem {
    /// What it says.
    pub label: String,
    /// A path of the site (`/…`) or an `http`/`https` address.
    pub target: String,
    /// The permission a person needs to see it, and the items under it; everyone when absent.
    #[serde(default)]
    pub permission: Option<String>,
    /// The items under it, in order.
    #[serde(default)]
    pub children: Vec<MenuItem>,
}

/// A navigation menu in a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Menu {
    /// The kind of menu.
    pub kind: String,
    /// The items the reader sees, in order.
    pub items: Vec<MenuItem>,
    /// Where the menu came from.
    pub origin: Origin,
    /// Whether the reader may change it here: they hold `customise.menu.manage` and no scope above locks it.
    pub editable: bool,
}
