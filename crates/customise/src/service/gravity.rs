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

//! The gravity of a grid of widgets: every widget not pinned rises as far as the ones above it let it, so a page never
//! shows an empty row above a widget unless a widget pinned below it holds it there. A placement keeps the place it
//! was given only when the person pinned it.

use std::collections::BTreeMap;

use crate::Placement;

/// The placements in reading order, with every widget that is not pinned risen as far as it goes. A pinned widget
/// stays where it is; the others go around it, and one that overlaps a widget before it goes down until it does not.
#[must_use]
pub fn settle(placements: Vec<Placement>) -> Vec<Placement> {
    let mut order = placements;
    order.sort_by_key(|p| (p.y, p.x));
    let mut settled: Vec<Placement> = order.iter().filter(|p| p.pinned).cloned().collect();
    for placement in order.into_iter().filter(|p| !p.pinned) {
        let mut moved = placement;
        while moved.y > 0
            && !settled.iter().any(|s| {
                s.overlaps(&Placement {
                    y: moved.y - 1,
                    ..moved.clone()
                })
            })
        {
            moved.y -= 1;
        }
        while settled.iter().any(|s| s.overlaps(&moved)) {
            moved.y += 1;
        }
        settled.push(moved);
    }
    settled.sort_by_key(|p| (p.y, p.x));
    settled
}

/// The first cell, in reading order, where a rectangle of a size fits among placements without moving any; under all
/// of them when none does.
#[must_use]
pub fn room(placements: &[Placement], w: u16, h: u16, columns: u16) -> (u16, u16) {
    let bottom = placements.iter().map(|p| p.y + p.h).max().unwrap_or(0);
    let probe = |x: u16, y: u16| Placement {
        widget: String::new(),
        x,
        y,
        w,
        h,
        pinned: false,
        locked: false,
        held: false,
        grows: false,
        settings: BTreeMap::new(),
        permission: None,
    };
    for y in 0..=bottom {
        for x in 0..=columns.saturating_sub(w) {
            if !placements.iter().any(|p| p.overlaps(&probe(x, y))) {
                return (x, y);
            }
        }
    }
    (0, bottom)
}
