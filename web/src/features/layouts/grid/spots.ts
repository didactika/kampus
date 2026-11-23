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
 * Where the widgets of a page go on the screen, in pixels: each takes the width of its columns and the height it
 * shows (its content, at most its rows), and rests on the lowest widget above it that shares a column, or on its own
 * row when it is pinned. The page and its editor place widgets with the same rule, so arranging a page shows it as it
 * will be read.
 */

import { reading, type Item } from "./engine";

/** The measures of a grid on the screen. */
export interface Metrics {
  readonly columns: number;
  /** The height of a row, in pixels. */
  readonly row: number;
  /** The room between widgets, in pixels. */
  readonly gap: number;
  /** The width of the page, in pixels. */
  readonly width: number;
}

/** Where a widget goes, in pixels. */
export interface Spot {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
}

/** The height of a number of rows, with the room between them. */
export const rows = (metrics: Metrics, h: number): number => h * metrics.row + (h - 1) * metrics.gap;

/** The spots of the items, and how tall the page they make is. */
export function spots(items: readonly Item[], metrics: Metrics, heightOf: (item: Item) => number): { readonly at: ReadonlyMap<number, Spot>; readonly bottom: number } {
  const { columns, row, gap, width } = metrics;
  const column = (width - gap * (columns - 1)) / columns;
  const at = new Map<number, Spot>();
  const placed: { readonly x: number; readonly w: number; readonly end: number }[] = [];
  let bottom = 0;
  for (const item of reading(items)) {
    const floor = placed.filter((p) => p.x < item.x + item.w && item.x < p.x + p.w).reduce((low, p) => Math.max(low, p.end + gap), 0);
    const top = item.pinned ? Math.max(floor, item.y * (row + gap)) : floor;
    // A widget that grows is as tall as its content; any other at most as tall as its rows.
    const height = item.grows ? heightOf(item) : Math.min(heightOf(item), rows(metrics, item.h));
    at.set(item.key, { left: item.x * (column + gap), top, width: item.w * column + (item.w - 1) * gap, height });
    placed.push({ x: item.x, w: item.w, end: top + height });
    bottom = Math.max(bottom, top + height);
  }
  return { at, bottom };
}
