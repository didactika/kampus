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
 * A short list in the chrome of a list (its order, the size of its pages): a pill as tall as the other controls of
 * the toolbar, with an optional icon before it and a name for assistive technology. It opens a list of the design
 * system, never the one of the browser, with a mark on the option chosen; `compact` makes it smaller, for a pager.
 */

import Box from "@mui/material/Box";
import ListItemIcon from "@mui/material/ListItemIcon";
import MenuItem from "@mui/material/MenuItem";
import OutlinedInput from "@mui/material/OutlinedInput";
import Select from "@mui/material/Select";
import { useId } from "react";

import { Icon } from "../../components/Icon";

/** The props of `PillSelect`. */
export interface PillSelectProps {
  /** What it chooses, for assistive technology and as the tooltip of its icon. */
  readonly label: string;
  /** The options: value and text. */
  readonly options: readonly { readonly value: string; readonly label: string }[];
  /** The value chosen. */
  readonly value: string;
  /** Chooses a value. */
  readonly onChange: (value: string) => void;
  /** An icon before the list. */
  readonly icon?: string;
  /** Smaller, for a pager. */
  readonly compact?: boolean;
  /** Where its list opens: below it, or above it when it sits at the foot of a page, such as a pager. */
  readonly opens?: "below" | "above";
}

/** `<PillSelect label="Order" icon="sort" options={…} value="name" onChange={…} />` */
export function PillSelect({ label, options, value, onChange, icon, compact = false, opens = "below" }: PillSelectProps) {
  const id = useId();
  // As wide as its longest option, so choosing another never changes its size nor moves what follows it.
  const longest = Math.max(1, ...options.map((option) => option.label.length));
  return (
    <Select
      value={value}
      onChange={(event) => onChange(String(event.target.value))}
      inputProps={{ id, "aria-label": label, title: label }}
      IconComponent={(iconProps: { className?: string }) => (
        <Icon name="keyboard_arrow_down" size="small" className={iconProps.className} sx={{ position: "absolute", insetInlineEnd: compact ? 6 : 10, pointerEvents: "none" }} />
      )}
      renderValue={(chosen) => options.find((o) => o.value === chosen)?.label ?? ""}
      MenuProps={{
        anchorOrigin: { vertical: opens === "above" ? "top" : "bottom", horizontal: "left" },
        transformOrigin: { vertical: opens === "above" ? "bottom" : "top", horizontal: "left" },
        slotProps: { paper: { sx: { mt: opens === "above" ? -0.5 : 0.5, minWidth: 160 } } },
      }}
      input={
        <OutlinedInput
          startAdornment={
            icon ? (
              <Box component="span" sx={{ display: "flex", color: "text.secondary", mr: 0.5 }}>
                <Icon name={icon} size="small" />
              </Box>
            ) : undefined
          }
          sx={{
            borderRadius: (theme) => `${theme.shape.full}px`,
            fontSize: compact ? "0.8125rem" : "0.875rem",
            fontWeight: 600,
            minHeight: compact ? 30 : undefined,
            pl: icon ? 1.5 : 0,
            "& .MuiSelect-select": { minWidth: `calc(${longest + 1}ch + ${compact ? 40 : icon ? 40 : 52}px) !important`, pl: icon ? 0.5 : compact ? 1.5 : 2, pr: compact ? "28px !important" : "36px !important", py: compact ? 0.5 : undefined, minHeight: compact ? "22px !important" : "24px" },
          }}
        />
      }
    >
      {options.map((option) => (
        <MenuItem key={option.value} value={option.value} dense={compact}>
          <ListItemIcon sx={{ minWidth: "28px !important", color: "inherit" }}>{option.value === value ? <Icon name="check" size="small" /> : null}</ListItemIcon>
          {option.label}
        </MenuItem>
      ))}
    </Select>
  );
}
