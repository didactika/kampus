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

//! Layouts and menus: the widgets of a page kind arranged in its regions, and navigation menus as data,
//! each resolved from the nearest scope that sets one unless a scope above locks its own.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Origin;

/// A widget in a region of a layout, with its settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Placement {
    /// The region of the page kind.
    pub region: String,
    /// The widget, by code.
    pub widget: String,
    /// Its settings, each of the kind the widget declares; the defaults fill what is left out.
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
    /// The permission a person needs to see it; everyone who sees the page when absent.
    #[serde(default)]
    pub permission: Option<String>,
}

/// The layout of a page kind in a scope.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct Layout {
    /// The page kind.
    pub page_kind: String,
    /// Its regions, in order.
    pub regions: Vec<String>,
    /// The widgets the reader sees, by region and in order; the page shows its own arrangement when the
    /// layout comes from its declaration.
    pub placements: Vec<Placement>,
    /// Where the layout came from.
    pub origin: Origin,
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
}
