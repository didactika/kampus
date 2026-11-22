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

/** How far something has gone, as a ring with its percentage in the middle and a name for assistive technology. */

import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Typography from "@mui/material/Typography";

/** The props of `ProgressRing`. */
export interface ProgressRingProps {
  /** From 0 to 100. */
  readonly value: number;
  /** What it measures. */
  readonly label: string;
  /** Its size in pixels. */
  readonly size?: number;
}

/** `<ProgressRing value={40} label="Course progress" />` */
export function ProgressRing({ value, label, size = 96 }: ProgressRingProps) {
  const shown = Math.round(Math.min(100, Math.max(0, value)));
  return (
    <Box sx={{ position: "relative", display: "inline-grid", placeItems: "center", inlineSize: size, blockSize: size }}>
      <CircularProgress variant="determinate" value={100} size={size} thickness={4} sx={{ position: "absolute", color: "surface.containerHighest" }} aria-hidden />
      <CircularProgress
        variant="determinate"
        value={shown}
        size={size}
        thickness={4}
        aria-label={label}
        sx={{ position: "absolute", color: "primary.main", "& .MuiCircularProgress-circle": { strokeLinecap: "round" } }}
      />
      <Typography variant="h4" component="span" aria-hidden>
        {shown}%
      </Typography>
    </Box>
  );
}
