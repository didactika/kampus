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
 * A short native list in the chrome of a list (its order, the size of its pages): a pill as tall as the other
 * controls of the toolbar, with an optional icon before it and a name for assistive technology.
 */

import Box from "@mui/material/Box";
import NativeSelect from "@mui/material/NativeSelect";
import OutlinedInput from "@mui/material/OutlinedInput";
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
}

/** The arrow of the list. */
function Arrow(props: { className?: string }) {
  return <Icon name="keyboard_arrow_down" size="small" className={props.className} sx={{ position: "absolute", insetInlineEnd: 10, pointerEvents: "none" }} />;
}

/** `<PillSelect label="Order" icon="sort" options={…} value="name" onChange={…} />` */
export function PillSelect({ label, options, value, onChange, icon }: PillSelectProps) {
  const id = useId();
  return (
    <NativeSelect
      value={value}
      onChange={(event) => onChange(event.target.value)}
      IconComponent={Arrow}
      inputProps={{ id, "aria-label": label, title: label }}
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
            fontSize: "0.875rem",
            fontWeight: 600,
            pl: icon ? 1.5 : 0,
            "& select": { pl: icon ? 0.5 : 2, pr: "36px !important", py: 0 },
          }}
        />
      }
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </NativeSelect>
  );
}
