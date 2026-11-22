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

/** One filter of a list: a small title and its values as chips with their counts, any number of them on. */

import Chip from "@mui/material/Chip";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useId } from "react";

import { Icon } from "../../components/Icon";

/** One value of a filter. */
export interface FilterValue {
  /** What it filters by. */
  readonly value: string;
  /** What it shows. */
  readonly label: string;
  /** How many items have it. */
  readonly count?: number;
}

/** The props of `FilterGroup`. */
export interface FilterGroupProps {
  /** The title of the filter. */
  readonly title: string;
  /** Its values. */
  readonly values: readonly FilterValue[];
  /** The values on. */
  readonly selected: readonly string[];
  /** Turns the values on. */
  readonly onChange: (selected: string[]) => void;
}

/** `<FilterGroup title="Category" values={…} selected={…} onChange={…} />` */
export function FilterGroup({ title, values, selected, onChange }: FilterGroupProps) {
  const heading = useId();
  return (
    <div role="group" aria-labelledby={heading}>
      <Typography id={heading} variant="overline" color="text.secondary" sx={{ display: "block", mb: 1, textTransform: "uppercase", letterSpacing: "0.06em" }}>
        {title}
      </Typography>
      <Stack direction="row" useFlexGap sx={{ flexWrap: "wrap", gap: 1 }}>
        {values.map((item) => {
          const on = selected.includes(item.value);
          return (
            <Chip
              key={item.value}
              label={item.count === undefined ? item.label : `${item.label} · ${item.count}`}
              variant={on ? "filled" : "outlined"}
              color={on ? "primary" : "default"}
              icon={on ? <Icon name="check" size="small" /> : undefined}
              aria-pressed={on}
              onClick={() => onChange(on ? selected.filter((v) => v !== item.value) : [...selected, item.value])}
              sx={on ? { bgcolor: "primary.container", color: "primary.onContainer", "& .MuiChip-icon": { color: "inherit" } } : undefined}
            />
          );
        })}
      </Stack>
    </div>
  );
}
