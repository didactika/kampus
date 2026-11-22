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
 * The expressive loading indicator, for waits too short to measure: a shape in the primary colour that turns and
 * changes form. `contained` puts it on a circle of the primary container, for use over content. For a long wait, or
 * one whose progress is known, use MUI's `CircularProgress` or `LinearProgress`.
 */

import Box from "@mui/material/Box";
import { keyframes } from "@mui/material/styles";

const morph = keyframes`
  0%, 100% { border-radius: 50%; scale: 0.9; }
  25% { border-radius: 30% 70% 70% 30% / 30% 30% 70% 70%; scale: 1; }
  50% { border-radius: 18%; scale: 0.86; }
  75% { border-radius: 62% 38% 46% 54% / 40% 58% 42% 60%; scale: 1; }
`;

const turn = keyframes`
  to { rotate: 360deg; }
`;

/** The props of `Loading`. */
export interface LoadingProps {
  /** What is being done, for assistive technology. */
  readonly label: string;
  /** Whether it sits on a circle of its own. */
  readonly contained?: boolean;
  /** Its size in rem. */
  readonly size?: number;
}

/** `<Loading label="Loading courses" />` */
export function Loading({ label, contained = false, size = 3 }: LoadingProps) {
  return (
    <Box
      role="status"
      aria-label={label}
      sx={{
        display: "inline-grid",
        placeItems: "center",
        inlineSize: `${size}rem`,
        blockSize: `${size}rem`,
        borderRadius: "50%",
        backgroundColor: contained ? "var(--k-palette-primary-container)" : "transparent",
      }}
    >
      <Box
        aria-hidden
        sx={{
          inlineSize: contained ? "58%" : "79%",
          blockSize: contained ? "58%" : "79%",
          backgroundColor: "primary.main",
          animation: `${morph} 2.8s cubic-bezier(0.38, 1.21, 0.22, 1) infinite, ${turn} 4.2s linear infinite`,
          "@media (prefers-reduced-motion: reduce)": { animation: "none", borderRadius: "50%" },
        }}
      />
    </Box>
  );
}
