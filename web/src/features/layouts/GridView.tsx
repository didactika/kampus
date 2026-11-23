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
// @package   web
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/**
 * A page of widgets as people read it: each widget in its rectangle of the grid, at the height of its rows, its
 * content scrolling inside it rather than spilling out. On a narrow screen the grid is one column, in reading order.
 */

import Box from "@mui/material/Box";
import type { ReactNode } from "react";

import { reading, type Item } from "./grid/engine";

/** The room between widgets, in units of the spacing of the theme. */
export const GAP = 3;

/** A widget on the page, with what it shows. */
export interface ShownWidget extends Item {
  readonly node: ReactNode;
}

/** The props of `GridView`. */
export interface GridViewProps {
  readonly columns: number;
  /** The height of a row, in pixels. */
  readonly row: number;
  readonly items: readonly ShownWidget[];
}

/** `<GridView columns={12} row={72} items={…} />` */
export function GridView({ columns, row, items }: GridViewProps) {
  return (
    <Box
      sx={{
        display: "grid",
        gap: GAP,
        gridTemplateColumns: { xs: "minmax(0, 1fr)", md: `repeat(${columns}, minmax(0, 1fr))` },
        gridAutoRows: { md: `${row}px` },
      }}
    >
      {reading(items).map((item) => (
        <Box
          key={item.key}
          sx={{
            minInlineSize: 0,
            minBlockSize: 0,
            gridColumn: { md: `${item.x + 1} / span ${item.w}` },
            gridRow: { md: `${item.y + 1} / span ${item.h}` },
            overflow: { md: "auto" },
            display: "flex",
            flexDirection: "column",
            "& > *": { flex: { md: "1 1 auto" } },
          }}
        >
          {item.node}
        </Box>
      ))}
    </Box>
  );
}
