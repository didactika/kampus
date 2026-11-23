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
 * A region of a page being arranged: its widgets in order, as a list they can be dragged within, and a place to drop
 * the widgets of other regions, which lights up while one is over it. An empty region still takes a widget.
 */

import { useDroppable } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import type { ReactNode } from "react";

/** The id a region takes among the places a widget can be dropped. */
export const regionId = (region: string) => `region:${region}`;

/** The props of `DropRegion`. */
export interface DropRegionProps {
  /** The code of the region. */
  readonly region: string;
  /** Its name, in the language of the page. */
  readonly label: string;
  /** The keys of its widgets, in order. */
  readonly items: readonly number[];
  /** Its widgets, then the way to add one. */
  readonly children: ReactNode;
}

/** `<DropRegion region="main" label="Main" items={[1, 2]}>…</DropRegion>` */
export function DropRegion({ region, label, items, children }: DropRegionProps) {
  const { setNodeRef, isOver } = useDroppable({ id: regionId(region) });
  return (
    <SortableContext id={regionId(region)} items={[...items]} strategy={verticalListSortingStrategy}>
      <Stack
        ref={setNodeRef}
        role="group"
        aria-label={label}
        spacing={2.5}
        sx={{
          p: 2,
          minBlockSize: 160,
          borderRadius: 4,
          border: "2px dashed",
          borderColor: isOver ? "primary.main" : "color-mix(in srgb, var(--k-palette-primary-main) 45%, transparent)",
          bgcolor: isOver ? "color-mix(in srgb, var(--k-palette-primary-main) 9%, transparent)" : "color-mix(in srgb, var(--k-palette-primary-main) 3%, transparent)",
          transition: "background-color 120ms, border-color 120ms",
        }}
      >
        <Typography variant="overline" color="primary" aria-hidden>
          {label}
        </Typography>
        {children}
      </Stack>
    </SortableContext>
  );
}
