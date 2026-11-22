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

/** A state or a count, as a small pill tinted with the colour of its state, never white on white. */

import Chip from "@mui/material/Chip";

/** What a badge tells: a state, or nothing more than a count. */
export type StatusTone = "neutral" | "primary" | "accent" | "info" | "success" | "warning" | "danger";

/** The palette colour of each tone. */
const PALETTE = {
  neutral: "secondary",
  primary: "primary",
  accent: "accent",
  info: "info",
  success: "success",
  warning: "warning",
  danger: "error",
} as const;

/** The props of `StatusBadge`. */
export interface StatusBadgeProps {
  /** What it says. */
  readonly label: string;
  /** What it tells. */
  readonly tone?: StatusTone;
}

/** `<StatusBadge tone="success" label="Done" />` */
export function StatusBadge({ label, tone = "neutral" }: StatusBadgeProps) {
  const colour = PALETTE[tone];
  return (
    <Chip
      component="span"
      label={label}
      size="small"
      sx={{
        backgroundColor: `var(--k-palette-${colour}-container)`,
        color: `var(--k-palette-${colour}-onContainer)`,
      }}
    />
  );
}
