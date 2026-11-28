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
 * A section of a page in a card: its title, what it is for, its actions, and its content, with the same spacing on
 * every page. Its title names it for assistive technology.
 */

import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useId, type ReactNode } from "react";

import { IconTile } from "../../components/IconTile/index";

/** The props of `SectionCard`. */
export interface SectionCardProps {
  /** Its title. */
  readonly title: ReactNode;
  /** What it is for, under its title. */
  readonly lead?: ReactNode;
  /** An icon by its title. */
  readonly icon?: string;
  /** Its actions, by its title. */
  readonly actions?: ReactNode;
  /** Whether it is dense, as a card at the side of a page. */
  readonly dense?: boolean;
  readonly children?: ReactNode;
}

/** `<SectionCard title="Sessions" lead="…">{list}</SectionCard>` */
export function SectionCard({ title, lead, icon, actions, dense = false, children }: SectionCardProps) {
  const id = useId();
  return (
    <Card component="section" aria-labelledby={id}>
      <CardContent sx={{ p: dense ? 2.5 : { xs: 2.5, md: 4 }, "&:last-child": { pb: dense ? 2.5 : { xs: 2.5, md: 4 } } }}>
        <Stack direction="row" spacing={1.5} sx={{ alignItems: "center", mb: lead ? 0.5 : dense ? 1.5 : 2.5 }}>
          {icon ? <IconTile name={icon} size="small" /> : null}
          <Typography id={id} variant={dense ? "h4" : "h3"} component="h2" sx={{ flex: 1, minInlineSize: 0 }}>
            {title}
          </Typography>
          {actions ? <Stack direction="row" spacing={1}>{actions}</Stack> : null}
        </Stack>
        {lead ? (
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2.5, maxInlineSize: "70ch" }}>
            {lead}
          </Typography>
        ) : null}
        {children}
      </CardContent>
    </Card>
  );
}
