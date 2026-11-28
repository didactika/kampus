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
 * An icon on a soft tile of the brand: the mark of a section, a widget, a row of a list or a dialog. One size for a
 * heading or a row, a smaller one for dense rows, a larger one for the head of a page.
 */

import Box, { type BoxProps } from "@mui/material/Box";

import { Icon } from "../Icon/index";

/** The side of each size, in pixels, and the size of the icon inside. */
const SIZES = { small: [32, "small"], medium: [40, "medium"], large: [56, "large"] } as const;

/** The props of `IconTile`. */
export interface IconTileProps extends Omit<BoxProps, "children"> {
  /** The icon, by name; or `text` instead, such as the number of a section. */
  readonly name?: string;
  /** A short text instead of an icon. */
  readonly text?: string;
  readonly size?: keyof typeof SIZES;
  /** The colour of the icon and its tile: the brand by default, or the tone of what it says. */
  readonly tone?: "primary" | "success" | "warning" | "error" | "info";
}

/** `<IconTile name="devices" />` */
export function IconTile({ name, text, size = "medium", tone = "primary", sx, ...rest }: IconTileProps) {
  const [side, icon] = SIZES[size];
  return (
    <Box
      aria-hidden
      {...rest}
      sx={[
        {
          display: "grid",
          placeItems: "center",
          inlineSize: side,
          blockSize: side,
          flex: "none",
          borderRadius: size === "small" ? 2 : 2.5,
          color: `${tone}.main`,
          backgroundImage: tone === "primary" ? "var(--k-palette-gradient-soft)" : undefined,
          bgcolor: tone === "primary" ? undefined : `color-mix(in srgb, var(--k-palette-${tone}-main) 14%, transparent)`,
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
    >
      {name ? <Icon name={name} size={icon} /> : <Box component="span" sx={{ typography: "subtitle1", fontWeight: 700 }}>{text}</Box>}
    </Box>
  );
}
