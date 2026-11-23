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
 * The rules of a grid of widgets, without the page: rectangles of cells that never overlap, moved and resized within
 * the sizes each widget declares, and the ones in the way pushed down. Every widget not pinned then rises as far as
 * the ones above it let it, so no row is left empty above a widget; the server settles a layout the same way.
 */

/** A size in cells. */
export interface Cells {
  readonly w: number;
  readonly h: number;
}

/** How a widget may be resized. */
export type Resize = "none" | "horizontal" | "vertical" | "both";

/** The sizes a widget takes on a page. */
export interface Sizes {
  readonly min: Cells;
  readonly default: Cells;
  readonly max: Cells;
  readonly resize: Resize;
}

/** A widget on the grid. */
export interface Item {
  readonly key: number;
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
  /** Whether the person pinned it where it is; otherwise it rises. */
  readonly pinned?: boolean;
}

/** Whether two rectangles share a cell. */
export function collides(a: Item, b: Item): boolean {
  return a.key !== b.key && a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

/** The items in reading order: by row, then by column. */
export function reading<T extends Item>(items: readonly T[]): T[] {
  return [...items].sort((a, b) => a.y - b.y || a.x - b.x || a.key - b.key);
}

/**
 * The items with every one not pinned risen as far as the ones above it let it, in reading order; a pinned one stays,
 * and one that overlaps an item before it goes down until it does not.
 */
export function settle<T extends Item>(items: readonly T[]): T[] {
  const settled: T[] = items.filter((i) => i.pinned);
  for (const item of reading(items.filter((i) => !i.pinned))) {
    let y = item.y;
    while (y > 0 && !settled.some((s) => collides({ ...item, y: y - 1 }, s))) y -= 1;
    while (settled.some((s) => collides({ ...item, y }, s))) y += 1;
    settled.push({ ...item, y });
  }
  return items.map((i) => settled.find((s) => s.key === i.key)!);
}

/** A number kept between two others. */
const clamp = (value: number, low: number, high: number) => Math.min(Math.max(value, low), high);

/**
 * Puts an item at a rectangle, kept within the grid; the items in its way go just under it, in reading order, each
 * pushing down those it then covers, and then everything settles. Placing from where the items were when a move began
 * brings back those it no longer covers.
 */
export function place<T extends Item>(items: readonly T[], key: number, to: Pick<Item, "x" | "y" | "w" | "h">, columns: number): T[] {
  const w = clamp(to.w, 1, columns);
  const moved = items.map((i) => (i.key === key ? { ...i, w, h: Math.max(1, to.h), x: clamp(to.x, 0, columns - w), y: Math.max(0, to.y) } : i));
  const target = moved.find((i) => i.key === key);
  if (!target) return [...items];
  const settled: T[] = [target];
  for (const item of reading(moved)) {
    if (item.key === key) continue;
    let y = item.y;
    while (settled.some((s) => collides({ ...item, y }, s))) y += 1;
    settled.push({ ...item, y });
  }
  return settle(moved.map((i) => settled.find((p) => p.key === i.key)!));
}

/** A size kept within the sizes of a widget and the directions it may change in. */
export function within(sizes: Sizes, size: Cells, columns: number): Cells {
  const horizontal = sizes.resize === "both" || sizes.resize === "horizontal";
  const vertical = sizes.resize === "both" || sizes.resize === "vertical";
  return {
    w: horizontal ? clamp(size.w, sizes.min.w, Math.min(sizes.max.w, columns)) : Math.min(sizes.default.w, columns),
    h: vertical ? clamp(size.h, sizes.min.h, sizes.max.h) : sizes.default.h,
  };
}

/**
 * Moves an item one step up or down in the order of the page: down, it goes under the next item in its way; up, over
 * the one above it. A plain move by one row would only settle back where it was.
 */
export function step<T extends Item>(items: readonly T[], key: number, down: boolean, columns: number): T[] {
  const start = items.find((i) => i.key === key);
  if (!start) return [...items];
  for (let y = start.y + (down ? 1 : -1); y >= 0 && y <= start.y + 200; y += down ? 1 : -1) {
    const placed = place(items, key, { ...start, y }, columns);
    const now = placed.find((i) => i.key === key)!;
    if (now.y !== start.y || now.x !== start.x) return placed;
  }
  return [...items];
}

/** The first place, in reading order, where a size fits without moving anything; under everything when none does. */
export function room(items: readonly Item[], size: Cells, columns: number): Pick<Item, "x" | "y"> {
  const bottom = items.reduce((low, i) => Math.max(low, i.y + i.h), 0);
  for (let y = 0; y <= bottom; y += 1) {
    for (let x = 0; x + size.w <= columns; x += 1) {
      if (!items.some((i) => collides({ key: -1, x, y, ...size }, i))) return { x, y };
    }
  }
  return { x: 0, y: bottom };
}
