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

"use client";

/**
 * A page of widgets as people read it. Each widget takes the width of its rectangle and the height of its content, at
 * most the height of its rows, scrolling inside beyond that; the widgets under it rise until they touch it, so a
 * widget whose content is shorter than its rows never leaves dead room below it. A pinned widget keeps its row. Until
 * the widths and heights are measured, and on a narrow screen, where the grid is one column in reading order, the
 * widgets sit in the cells of a CSS grid.
 */

import Box from "@mui/material/Box";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useTheme } from "@mui/material/styles";
import { useEffect, useRef, useState, type ReactNode } from "react";

import { reading, type Item } from "./grid/engine";
import { rows, spots as place, type Spot } from "./grid/spots";

/** The room between widgets, in units of the spacing of the theme. */
export const GAP = 3;

/** The pixels of one unit of spacing of the theme. */
const UNIT = 8;

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
  const theme = useTheme();
  const wide = useMediaQuery(theme.breakpoints.up("md"));
  const canvas = useRef<HTMLDivElement>(null);
  const contents = useRef(new Map<number, HTMLElement>());
  const [width, setWidth] = useState(0);
  const [heights, setHeights] = useState<Readonly<Record<number, number>>>({});
  const gap = GAP * UNIT;
  const cell = (h: number) => rows({ columns, row, gap, width }, h);

  useEffect(() => {
    const element = canvas.current;
    if (!element) return undefined;
    const observer = new ResizeObserver(([entry]) => setWidth(entry?.contentRect.width ?? 0));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const observer = new ResizeObserver((entries) =>
      setHeights((known) => {
        const next = { ...known };
        for (const entry of entries) next[Number((entry.target as HTMLElement).dataset["key"])] = (entry.target as HTMLElement).offsetHeight;
        return next;
      }),
    );
    for (const element of contents.current.values()) observer.observe(element);
    return () => observer.disconnect();
  }, [items]);

  const order = reading(items);
  const measured = wide && width > 0 && order.every((i) => heights[i.key] !== undefined);
  const { at: spots, bottom } = measured
    ? place(items, { columns, row, gap, width }, (item) => heights[item.key] ?? 0)
    : { at: new Map<number, Spot>(), bottom: 0 };

  return (
    <Box
      ref={canvas}
      sx={
        measured
          ? { position: "relative", blockSize: bottom }
          : {
              display: "grid",
              gap: GAP,
              gridTemplateColumns: { xs: "minmax(0, 1fr)", md: `repeat(${columns}, minmax(0, 1fr))` },
              gridAutoRows: { md: `${row}px` },
            }
      }
    >
      {order.map((item) => {
        const spot = spots.get(item.key);
        return (
          <Box
            key={item.key}
            sx={
              spot
                ? { position: "absolute", left: spot.left, top: spot.top, inlineSize: spot.width, transition: "top 200ms cubic-bezier(0.2, 0, 0, 1)" }
                : { minInlineSize: 0, gridColumn: { md: `${item.x + 1} / span ${item.w}` }, gridRow: { md: `${item.y + 1} / span ${item.h}` } }
            }
          >
            <Box
              data-key={item.key}
              ref={(element: HTMLElement | null) => {
                if (element) contents.current.set(item.key, element);
                else contents.current.delete(item.key);
              }}
              sx={{ maxBlockSize: { md: cell(item.h) }, overflow: { md: "clip" }, overflowClipMargin: 40, display: "flex", flexDirection: "column", "& > *": { minBlockSize: 0 } }}
            >
              {item.node}
            </Box>
          </Box>
        );
      })}
    </Box>
  );
}
