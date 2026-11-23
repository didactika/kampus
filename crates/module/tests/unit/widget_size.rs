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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The sizes a widget declares, checked against the grid of a page.

use kampus_module::{Cells, Grid, Resize, WidgetSize};

/// A size on the standard grid, between two sizes, starting at the smaller.
const fn size(min: Cells, max: Cells, resize: Resize) -> WidgetSize {
    WidgetSize {
        page_kind: "testing.page",
        min,
        default: min,
        max,
        resize,
    }
}

#[test]
fn a_size_fits_its_grid_only_between_its_bounds_and_in_the_directions_it_resizes() {
    let grid = Grid::STANDARD;
    assert_eq!(
        size(Cells::new(2, 2), Cells::new(6, 4), Resize::Both).problem(grid),
        None
    );
    assert_eq!(
        WidgetSize::fixed("testing.page", Cells::new(8, 10)).problem(grid),
        None
    );
    for wrong in [
        size(Cells::new(0, 2), Cells::new(6, 4), Resize::Both),
        size(Cells::new(6, 4), Cells::new(2, 2), Resize::Both),
        size(Cells::new(2, 2), Cells::new(13, 4), Resize::Both),
        size(Cells::new(2, 2), Cells::new(6, 4), Resize::None),
        size(Cells::new(2, 2), Cells::new(6, 4), Resize::Horizontal),
        size(Cells::new(2, 2), Cells::new(6, 4), Resize::Vertical),
    ] {
        assert!(wrong.problem(grid).is_some(), "{wrong:?}");
    }
    assert_eq!(
        size(Cells::new(2, 2), Cells::new(6, 2), Resize::Horizontal).problem(grid),
        None
    );
}
