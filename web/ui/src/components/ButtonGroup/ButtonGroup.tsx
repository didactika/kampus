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
 * Buttons that belong together. Connected, they touch as one control, with full corners at the ends; standard, they
 * sit apart with a gap.
 */

import MuiButtonGroup from "@mui/material/ButtonGroup";
import Stack from "@mui/material/Stack";
import type { ReactNode } from "react";

/** The props of `ButtonGroup`. */
export interface ButtonGroupProps {
  /** What the buttons are about, for assistive technology. */
  readonly label: string;
  /** Whether the buttons touch as one control. */
  readonly connected?: boolean;
  /** The buttons. */
  readonly children: ReactNode;
}

/** `<ButtonGroup label="View" connected>…</ButtonGroup>` */
export function ButtonGroup({ label, connected = false, children }: ButtonGroupProps) {
  if (connected) {
    return (
      <MuiButtonGroup role="group" aria-label={label} variant="outlined">
        {children}
      </MuiButtonGroup>
    );
  }
  return (
    <Stack direction="row" role="group" aria-label={label} useFlexGap sx={{ flexWrap: "wrap", gap: 1 }}>
      {children}
    </Stack>
  );
}
