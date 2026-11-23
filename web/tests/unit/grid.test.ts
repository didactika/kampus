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

import { describe, expect, it } from "vitest";

import { collides, place, reading, room, settle, step, within, type Item, type Sizes } from "../../src/features/layouts/grid/engine";

/** A rectangle of the grid. */
const at = (key: number, x: number, y: number, w: number, h: number): Item => ({ key, x, y, w, h });

/** No two rectangles of a page share a cell. */
const apart = (items: readonly Item[]) => items.every((a) => items.every((b) => !collides(a, b)));

describe("the grid of a page", () => {
  const page = [at(1, 0, 0, 12, 4), at(2, 0, 4, 8, 8), at(3, 8, 4, 4, 5)];

  it("puts a widget where it is moved and pushes down the ones in its way, keeping the others", () => {
    const moved = place(page, 3, { x: 0, y: 0, w: 4, h: 5 }, 12);
    expect(moved.find((i) => i.key === 3)).toEqual(at(3, 0, 0, 4, 5));
    expect(moved.find((i) => i.key === 1)).toEqual(at(1, 0, 5, 12, 4));
    expect(moved.find((i) => i.key === 2)).toEqual(at(2, 0, 9, 8, 8));
    expect(apart(moved)).toBe(true);
  });

  it("never leaves an empty row above a widget: what moves down rises to the ones above it", () => {
    const moved = place(page, 3, { x: 8, y: 20, w: 4, h: 5 }, 12);
    expect(moved.find((i) => i.key === 3)).toEqual(at(3, 8, 4, 4, 5));
    expect(settle([at(1, 0, 3, 4, 2), at(2, 6, 9, 4, 2)])).toEqual([at(1, 0, 0, 4, 2), at(2, 6, 0, 4, 2)]);
  });

  it("keeps a pinned widget where it is, and the ones under it rest on it", () => {
    const pinned = { ...at(2, 0, 6, 4, 2), pinned: true };
    expect(settle([at(1, 0, 0, 4, 2), pinned, at(3, 0, 9, 4, 2)])).toEqual([at(1, 0, 0, 4, 2), pinned, at(3, 0, 8, 4, 2)]);
  });

  it("steps a widget under the next one and back, as the arrows do", () => {
    const down = step(page, 1, true, 12);
    expect(down.find((i) => i.key === 1)!.y).toBeGreaterThan(0);
    expect(step(down, 1, false, 12).find((i) => i.key === 1)!.y).toBe(0);
  });

  it("keeps a widget inside the grid", () => {
    expect(place(page, 3, { x: 11, y: -2, w: 4, h: 5 }, 12).find((i) => i.key === 3)).toEqual(at(3, 8, 0, 4, 5));
  });

  it("resizes within the sizes a widget declares and only in the directions it allows", () => {
    const sizes: Sizes = { min: { w: 3, h: 3 }, default: { w: 4, h: 5 }, max: { w: 6, h: 10 }, resize: "both" };
    expect(within(sizes, { w: 9, h: 1 }, 12)).toEqual({ w: 6, h: 3 });
    expect(within({ ...sizes, resize: "vertical" }, { w: 6, h: 7 }, 12)).toEqual({ w: 4, h: 7 });
    const fixed: Sizes = { min: { w: 8, h: 10 }, default: { w: 8, h: 10 }, max: { w: 8, h: 10 }, resize: "none" };
    expect(within(fixed, { w: 2, h: 2 }, 12)).toEqual({ w: 8, h: 10 });
  });

  it("finds the first room that holds a new widget, in reading order", () => {
    expect(room(page, { w: 4, h: 3 }, 12)).toEqual({ x: 8, y: 9 });
    expect(room(page, { w: 12, h: 2 }, 12)).toEqual({ x: 0, y: 12 });
  });

  it("reads widgets by row, then by column", () => {
    expect(reading([at(1, 8, 4, 4, 5), at(2, 0, 4, 8, 8), at(3, 0, 0, 12, 4)]).map((i) => i.key)).toEqual([3, 2, 1]);
  });
});
