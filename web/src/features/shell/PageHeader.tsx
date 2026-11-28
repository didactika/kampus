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

/** The head of a page: an optional way back, its title, a line that says what it is for, and its actions. */

import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import type { ReactNode } from "react";

import { Icon } from "@kampus/ui";

/** The props of `PageHeader`. */
export interface PageHeaderProps {
  /** The title. */
  readonly title: string;
  /** What the page is for. */
  readonly lead?: string | null;
  /** Where the way back goes, and what it says. */
  readonly back?: { readonly href: string; readonly label: string };
  /** The actions of the page. */
  readonly actions?: ReactNode;
  /** Something before the title, such as the picture of a person. */
  readonly leading?: ReactNode;
  /** A short fact beside the title, such as how many there are. */
  readonly meta?: ReactNode;
}

/** `<PageHeader title="Courses" lead="…" />` */
export function PageHeader({ title, lead, back, actions, leading, meta }: PageHeaderProps) {
  return (
    <Box component="header" sx={{ mb: 4 }}>
      {back ? (
        <Button component={Link} href={back.href} variant="text" startIcon={<Icon name="arrow_back" size="small" />} sx={{ mb: 1, ml: -1.5 }}>
          {back.label}
        </Button>
      ) : null}
      <Stack direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "flex-end" }, justifyContent: "space-between" }}>
        <Stack direction="row" spacing={2.5} sx={{ alignItems: "center", minInlineSize: 0 }}>
          {leading}
          <Box sx={{ minInlineSize: 0 }}>
          <Stack direction="row" spacing={1.5} sx={{ alignItems: "baseline", flexWrap: "wrap" }}>
            <Typography variant="h1">{title}</Typography>
            {meta ? (
              <Typography variant="body2" color="text.secondary">
                {meta}
              </Typography>
            ) : null}
          </Stack>
          {lead ? (
            <Typography variant="body1" color="text.secondary" sx={{ mt: 1, maxInlineSize: "70ch" }}>
              {lead}
            </Typography>
          ) : null}
          </Box>
        </Stack>
        {actions ? <Stack direction="row" spacing={1.5}>{actions}</Stack> : null}
      </Stack>
    </Box>
  );
}
