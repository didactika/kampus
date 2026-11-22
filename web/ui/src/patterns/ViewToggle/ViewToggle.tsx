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

/** A choice among a few ways of showing the same thing (cards, a list), as one connected control of icons. */

import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Tooltip from "@mui/material/Tooltip";

import { Icon } from "../../components/Icon";

/** One way of showing. */
export interface ViewOption {
  /** Its value. */
  readonly value: string;
  /** Its name. */
  readonly label: string;
  /** Its icon. */
  readonly icon: string;
}

/** The props of `ViewToggle`. */
export interface ViewToggleProps {
  /** What the choice is about. */
  readonly label: string;
  /** The ways of showing. */
  readonly options: readonly ViewOption[];
  /** The one chosen. */
  readonly value: string;
  /** Chooses one. */
  readonly onChange: (value: string) => void;
}

/** `<ViewToggle label="View" options={…} value="grid" onChange={…} />` */
export function ViewToggle({ label, options, value, onChange }: ViewToggleProps) {
  return (
    <ToggleButtonGroup
      exclusive
      size="small"
      aria-label={label}
      value={value}
      onChange={(_, next: string | null) => next && onChange(next)}
      sx={{
        "& .MuiToggleButton-root": { px: 1.5, borderColor: "divider" },
        "& .MuiToggleButton-root:first-of-type": { borderStartStartRadius: 999, borderEndStartRadius: 999 },
        "& .MuiToggleButton-root:last-of-type": { borderStartEndRadius: 999, borderEndEndRadius: 999 },
        "& .Mui-selected": { bgcolor: "secondary.container", color: "secondary.onContainer" },
      }}
    >
      {options.map((option) => (
        <Tooltip key={option.value} title={option.label}>
          <ToggleButton value={option.value} aria-label={option.label}>
            <Icon name={option.icon} size="small" />
          </ToggleButton>
        </Tooltip>
      ))}
    </ToggleButtonGroup>
  );
}
