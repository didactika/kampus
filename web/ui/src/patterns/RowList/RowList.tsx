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
 * A list of rows, the same everywhere: each row an icon, what it is, a line about it and its actions, rounded and lit
 * under the pointer, inside the card of its section. A row may be marked, such as something not read yet.
 */

import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import type { ReactNode } from "react";

import { IconTile, type IconTileProps } from "../../components/IconTile/index";

/** The props of `RowList`. */
export interface RowListProps {
  /** What the list is, for assistive technology. */
  readonly label?: string;
  readonly children: ReactNode;
}

/** `<RowList label="Sessions"><Row …/></RowList>` */
export function RowList({ label, children }: RowListProps) {
  return (
    <Box component="ul" aria-label={label} sx={{ m: 0, mx: -1, p: 0, listStyle: "none", display: "flex", flexDirection: "column", gap: 0.5 }}>
      {children}
    </Box>
  );
}

/** The props of `Row`. */
export interface RowProps {
  /** Its icon. */
  readonly icon?: string;
  readonly tone?: IconTileProps["tone"];
  /** What it is. */
  readonly title: ReactNode;
  /** A line about it. */
  readonly meta?: ReactNode;
  /** Its actions, at its end. */
  readonly actions?: ReactNode;
  /** Whether it is marked, such as something not read yet. */
  readonly marked?: boolean;
}

/** `<Row icon="devices" title="Chrome on Linux" meta="…" actions={…} />` */
export function Row({ icon, tone, title, meta, actions, marked = false }: RowProps) {
  return (
    <Stack
      component="li"
      direction={{ xs: "column", sm: "row" }}
      spacing={{ xs: 1, sm: 2 }}
      sx={{
        alignItems: { sm: "center" },
        px: 1.5,
        py: 1.25,
        borderRadius: 3,
        bgcolor: marked ? "color-mix(in srgb, var(--k-palette-primary-main) 6%, transparent)" : "transparent",
        transition: "background-color 120ms",
        "&:hover": { bgcolor: "action.hover" },
      }}
    >
      <Stack direction="row" spacing={2} sx={{ alignItems: "center", flex: 1, minInlineSize: 0 }}>
        {icon ? <IconTile name={icon} tone={tone} /> : null}
        <Box sx={{ flex: 1, minInlineSize: 0 }}>
          <Typography variant="subtitle2" noWrap sx={{ fontWeight: marked ? 700 : 600 }}>
            {title}
          </Typography>
          {meta ? (
            <Typography variant="caption" color="text.secondary" component="p" noWrap>
              {meta}
            </Typography>
          ) : null}
        </Box>
      </Stack>
      {actions ? <Stack direction="row" spacing={1} sx={{ flex: "none", alignItems: "center" }}>{actions}</Stack> : null}
    </Stack>
  );
}
