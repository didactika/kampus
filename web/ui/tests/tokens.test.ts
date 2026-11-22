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
// @package   web/ui
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/**
 * The default theme resolves, and every pair of colours that carries text or an edge meets WCAG 2.2 AA in both schemes.
 */

import { describe, expect, it } from "vitest";

import dark from "../tokens/dark.tokens.json";
import light from "../tokens/light.tokens.json";
import reference from "../tokens/reference.tokens.json";
import { contrast, flatten, roles, type TokenNode } from "../src/index.js";

const documents = {
  reference: reference as TokenNode,
  light: light as TokenNode,
  dark: dark as TokenNode,
};

/** The colour roles of a scheme, by name. */
function scheme(document: TokenNode): Record<string, string> {
  return Object.fromEntries(roles(documents.reference, document).map((t) => [t.path.slice("color.".length), String(t.value)]));
}

const COLOURS = ["primary", "secondary", "accent", "danger", "success", "warning", "info"];
const SURFACES = ["surface", ...["lowest", "low", "", "high", "highest"].map((s) => (s ? `surface-container-${s}` : "surface-container"))];

/** Text on its background: 4.5 to 1; the edge of a control: 3 to 1. */
const PAIRS: [string, string, number][] = [
  ...COLOURS.flatMap((c): [string, string, number][] => [
    [`on-${c}`, c, 4.5],
    [`on-${c}-container`, `${c}-container`, 4.5],
    [c, "surface", 4.5],
  ]),
  ["on-brand", "brand", 4.5],
  ...SURFACES.flatMap((s): [string, string, number][] => [
    ["on-surface", s, 4.5],
    ["on-surface-variant", s, 4.5],
    ["outline", s, 3],
  ]),
  ["inverse-on-surface", "inverse-surface", 4.5],
  ["inverse-primary", "inverse-surface", 4.5],
];

describe("the default theme", () => {
  for (const [mode, roles] of [
    ["light", scheme(documents.light)],
    ["dark", scheme(documents.dark)],
  ] as const) {
    it(`${mode}: has the same roles as the other scheme`, () => {
      expect(Object.keys(roles).sort()).toEqual(Object.keys(scheme(documents.light)).sort());
    });
    for (const [fore, back, minimum] of PAIRS) {
      it(`${mode}: ${fore} on ${back} reaches ${minimum}:1`, () => {
        const ratio = contrast(roles[fore]!, roles[back]!);
        expect(ratio, `${roles[fore]} on ${roles[back]} is ${ratio.toFixed(2)}:1`).toBeGreaterThanOrEqual(minimum);
      });
    }
  }

  it("keeps Didactika's brand and primary colours", () => {
    const roles = scheme(documents.light);
    expect(roles["brand"]).toBe("#00555a");
    expect(roles["primary"]).toBe("#0f766e");
    expect(roles["surface"]).toBe("#f8fafc");
    expect(scheme(documents.dark)["surface"]).toBe("#0f172a");
  });

  it("refuses a token that refers to itself or to nothing", () => {
    expect(() => flatten({ color: { a: { $value: "{color.b}" }, b: { $value: "{color.a}" } } })).toThrow(/itself/);
    expect(() => flatten({ color: { a: { $value: "{color.none}" } } })).toThrow(/unknown/);
  });
});
