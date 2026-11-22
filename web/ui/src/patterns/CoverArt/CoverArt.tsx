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


"use client";

/**
 * The cover of an item without an image of its own: a composition of the colour roles of the theme, chosen from the
 * item's id so the same item always looks the same and items next to each other differ, with an icon of what it is.
 * It follows the theme of the scope and both schemes, and draws nothing but CSS.
 */

import Box from "@mui/material/Box";
import type { ReactNode } from "react";

import { Icon } from "../../components/Icon";

/** The colours a cover can take. */
const TONES = ["primary", "accent", "info", "success", "secondary", "warning"] as const;

/** The compositions a cover can take, as layers of gradients over the container colour. */
const COMPOSITIONS = [
  (a: string, b: string) => `radial-gradient(circle at 85% 20%, ${b} 0 18%, transparent 19%), radial-gradient(circle at 15% 110%, ${a} 0 35%, transparent 36%)`,
  (a: string, b: string) => `linear-gradient(135deg, transparent 0 55%, ${a} 55% 70%, transparent 70%), linear-gradient(135deg, transparent 0 75%, ${b} 75%)`,
  (a: string, b: string) => `radial-gradient(${a} 2px, transparent 2.5px) 0 0 / 14px 14px, radial-gradient(ellipse at 100% 100%, ${b} 0 40%, transparent 41%)`,
  (a: string, b: string) => `repeating-linear-gradient(-45deg, transparent 0 14px, ${a} 14px 16px), radial-gradient(circle at 0% 0%, ${b} 0 30%, transparent 31%)`,
] as const;

/** A number from a text, the same for the same text. */
function hash(text: string): number {
  let value = 2166136261;
  for (const char of text) value = Math.imul(value ^ char.charCodeAt(0), 16777619);
  return value >>> 0;
}

/** The props of `CoverArt`. */
export interface CoverArtProps {
  /** What chooses the colours and composition, such as the id of the item. */
  readonly seed: string;
  /** The icon of what the item is. */
  readonly icon?: string;
  /** Its height in pixels. */
  readonly height?: number;
  /** What sits over it, such as badges. */
  readonly children?: ReactNode;
}

/** `<CoverArt seed={course.id} icon="science" />` */
export function CoverArt({ seed, icon, height = 120, children }: CoverArtProps) {
  const value = hash(seed);
  const tone = TONES[value % TONES.length]!;
  const next = TONES[(value >>> 3) % TONES.length]!;
  const compose = COMPOSITIONS[(value >>> 6) % COMPOSITIONS.length]!;
  const a = `color-mix(in srgb, var(--k-palette-${tone === "secondary" ? "secondary" : tone}-main) 22%, transparent)`;
  const b = `color-mix(in srgb, var(--k-palette-${next}-main) 30%, transparent)`;
  return (
    <Box
      aria-hidden
      sx={{
        position: "relative",
        blockSize: height,
        overflow: "hidden",
        color: `${tone === "warning" ? "warning" : tone}.onContainer`,
        background: `${compose(a, b)}, linear-gradient(160deg, var(--k-palette-${tone}-container), var(--k-palette-${next}-container))`,
      }}
    >
      {icon ? (
        <Box
          sx={{
            position: "absolute",
            insetInlineStart: 16,
            insetBlockEnd: 16,
            display: "grid",
            placeItems: "center",
            inlineSize: 48,
            blockSize: 48,
            borderRadius: 3,
            bgcolor: "background.paper",
            boxShadow: 2,
          }}
        >
          <Icon name={icon} />
        </Box>
      ) : null}
      {children ? <Box sx={{ position: "absolute", insetInlineEnd: 12, insetBlockStart: 12, display: "flex", gap: 0.75 }}>{children}</Box> : null}
    </Box>
  );
}
