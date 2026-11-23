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

//! The conversion of layouts from regions to the grid.

use kampus_customise::db::grid::arrange;

#[test]
fn the_top_goes_across_the_main_region_left_and_the_side_right_each_in_its_order() {
    let id = |n: u128| uuid::Uuid::from_u128(n);
    let placed = |n: u128, region: &str, widget: &str| (id(n), region.to_owned(), widget.to_owned());
    let rectangles = arrange(&[
        placed(1, "main", "courses.catalogue"),
        placed(2, "top", "customise.welcome"),
        placed(3, "side", "notify.inbox"),
        placed(4, "side", "customise.text"),
        placed(5, "main", "customise.text"),
    ]);
    let at = |n: u128| {
        rectangles
            .iter()
            .find(|r| r.0 == id(n))
            .map(|r| (r.1, r.2, r.3, r.4))
            .unwrap()
    };
    assert_eq!(at(2), (0, 0, 12, 4), "the welcome across the top");
    assert_eq!(at(1), (0, 4, 8, 8), "the catalogue on the left, under it");
    assert_eq!(at(5), (0, 12, 8, 3), "then the text");
    assert_eq!(at(3), (8, 4, 4, 5), "the notifications on the right");
    assert_eq!(at(4), (8, 9, 4, 3), "then the other text");
}
