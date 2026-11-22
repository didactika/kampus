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


/**
 * A list to choose one from, with its label above it like a text field. It is the native list of the browser, so it
 * works with every keyboard, screen reader and form.
 */

import FormControl from "@mui/material/FormControl";
import FormHelperText from "@mui/material/FormHelperText";
import FormLabel from "@mui/material/FormLabel";
import NativeSelect, { type NativeSelectProps } from "@mui/material/NativeSelect";
import OutlinedInput from "@mui/material/OutlinedInput";
import { useId } from "react";

import { Icon } from "../Icon/index.js";

/** One option of a `SelectField`. */
export interface SelectOption {
  /** What it submits. */
  readonly value: string;
  /** What it shows. */
  readonly label: string;
  /** Whether it cannot be chosen. */
  readonly disabled?: boolean;
}

/** The props of `SelectField`. */
export interface SelectFieldProps extends Omit<NativeSelectProps, "error" | "input" | "children"> {
  /** The visible label. */
  readonly label: string;
  /** The options. */
  readonly options: readonly SelectOption[];
  /** Help under the list. */
  readonly hint?: string;
  /** An error to show. */
  readonly error?: string;
}

/** The arrow of the list. */
function Arrow(props: { className?: string }) {
  return <Icon name="keyboard_arrow_down" className={props.className} sx={{ insetInlineEnd: 12, position: "absolute" }} />;
}

/** `<SelectField label="Language" name="language" options={[…]} />` */
export function SelectField({ label, options, hint, error, id, required, disabled, ...select }: SelectFieldProps) {
  const own = useId();
  const field = id ?? own;
  const described = [hint ? `${field}-hint` : "", error ? `${field}-error` : ""].filter(Boolean).join(" ") || undefined;
  return (
    <FormControl fullWidth required={required} disabled={disabled} error={Boolean(error)}>
      <FormLabel htmlFor={field}>{label}</FormLabel>
      <NativeSelect
        {...select}
        IconComponent={Arrow}
        input={<OutlinedInput />}
        inputProps={{ id: field, required, "aria-describedby": described, "aria-invalid": error ? true : undefined }}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value} disabled={option.disabled}>
            {option.label}
          </option>
        ))}
      </NativeSelect>
      {hint ? <FormHelperText id={`${field}-hint`} error={false}>{hint}</FormHelperText> : null}
      {error ? <FormHelperText id={`${field}-error`}>{error}</FormHelperText> : null}
    </FormControl>
  );
}
