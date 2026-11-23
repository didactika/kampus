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
 * A list to choose one from, with its label above it like a text field. It opens a list of the design system, never
 * the one of the browser: the option chosen carries a mark, the keyboard moves through it and types ahead, and a
 * long list gets a search at its top. A hidden input carries the value, so it takes part in the form around it.
 */

import Autocomplete from "@mui/material/Autocomplete";
import FormControl from "@mui/material/FormControl";
import FormHelperText from "@mui/material/FormHelperText";
import FormLabel from "@mui/material/FormLabel";
import ListItemIcon from "@mui/material/ListItemIcon";
import MenuItem from "@mui/material/MenuItem";
import OutlinedInput from "@mui/material/OutlinedInput";
import Select from "@mui/material/Select";
import { useId, useState } from "react";

import { Icon } from "../Icon/index";

/** One option of a `SelectField`. */
export interface SelectOption {
  /** What it submits. */
  readonly value: string;
  /** What it shows. */
  readonly label: string;
  /** Whether it cannot be chosen. */
  readonly disabled?: boolean;
}

/** From how many options a list gets a search. */
export const SEARCH_FROM = 11;

/** The props of `SelectField`. */
export interface SelectFieldProps {
  /** The visible label. */
  readonly label: string;
  /** The options. */
  readonly options: readonly SelectOption[];
  /** The name it submits with, in a form. */
  readonly name?: string;
  /** The value chosen, when the field is controlled. */
  readonly value?: string;
  /** The value it starts with, when it is not; the first option otherwise. */
  readonly defaultValue?: string;
  /** Chooses a value. */
  readonly onChange?: (value: string) => void;
  /** Help under the list. */
  readonly hint?: string;
  /** An error to show. */
  readonly error?: string;
  /** What the search says while empty, in a long list. */
  readonly searchPlaceholder?: string;
  /** What a search that finds nothing says. */
  readonly noMatch?: string;
  readonly id?: string;
  readonly required?: boolean;
  readonly disabled?: boolean;
}

/** The mark of the option chosen, or the room it takes. */
function Mark({ chosen }: { readonly chosen: boolean }) {
  return (
    <ListItemIcon sx={{ minWidth: "28px !important", color: "inherit" }}>
      {chosen ? <Icon name="check" size="small" /> : null}
    </ListItemIcon>
  );
}

/** `<SelectField label="Language" name="language" options={[…]} />` */
export function SelectField(props: SelectFieldProps) {
  const { label, options, name, hint, error, id, required, disabled, onChange, searchPlaceholder, noMatch } = props;
  const own = useId();
  const field = id ?? own;
  const [inner, setInner] = useState(props.defaultValue ?? options[0]?.value ?? "");
  const value = props.value ?? inner;
  const choose = (next: string) => {
    if (props.value === undefined) setInner(next);
    onChange?.(next);
  };
  const described = [hint ? `${field}-hint` : "", error ? `${field}-error` : ""].filter(Boolean).join(" ") || undefined;
  const searchable = options.length >= SEARCH_FROM;

  return (
    <FormControl fullWidth required={required} disabled={disabled} error={Boolean(error)}>
      <FormLabel htmlFor={field} id={`${field}-label`}>
        {label}
      </FormLabel>
      {searchable ? (
        <>
          <Autocomplete
            id={field}
            options={options}
            value={options.find((o) => o.value === value) ?? null}
            onChange={(_, option) => option && choose(option.value)}
            getOptionLabel={(option) => option.label}
            getOptionDisabled={(option) => Boolean(option.disabled)}
            isOptionEqualToValue={(option, chosen) => option.value === chosen.value}
            disableClearable={required || options.some((o) => o.value === "")}
            disabled={disabled}
            noOptionsText={noMatch}
            popupIcon={<Icon name="keyboard_arrow_down" />}
            autoHighlight
            renderOption={({ key, ...item }, option, { selected }) => (
              <li key={key} {...item}>
                <Mark chosen={selected} />
                {option.label}
              </li>
            )}
            renderInput={(params) => (
              <OutlinedInput
                ref={params.slotProps.input.ref}
                className={params.slotProps.input.className}
                onMouseDown={params.slotProps.input.onMouseDown}
                endAdornment={params.slotProps.input.endAdornment}
                startAdornment={<Icon name="search" size="small" sx={{ color: "text.secondary", ml: 0.5 }} />}
                fullWidth
                inputProps={{
                  ...params.slotProps.htmlInput,
                  placeholder: searchPlaceholder,
                  "aria-describedby": described,
                  "aria-invalid": error ? true : undefined,
                }}
              />
            )}
          />
          {name ? <input type="hidden" name={name} value={value} /> : null}
        </>
      ) : (
        <Select
          value={options.some((o) => o.value === value) ? value : ""}
          onChange={(event) => choose(String(event.target.value))}
          name={name}
          displayEmpty
          input={<OutlinedInput />}
          IconComponent={(iconProps: { className?: string }) => (
            <Icon name="keyboard_arrow_down" className={iconProps.className} sx={{ insetInlineEnd: 12, position: "absolute", pointerEvents: "none" }} />
          )}
          inputProps={{ id: field, "aria-describedby": described, "aria-invalid": error ? true : undefined }}
          labelId={`${field}-label`}
          renderValue={(chosen) => options.find((o) => o.value === chosen)?.label ?? ""}
          MenuProps={{ slotProps: { paper: { sx: { mt: 0.5 } } } }}
        >
          {options.map((option) => (
            <MenuItem key={option.value} value={option.value} disabled={option.disabled}>
              <Mark chosen={option.value === value} />
              {option.label}
            </MenuItem>
          ))}
        </Select>
      )}
      {hint ? (
        <FormHelperText id={`${field}-hint`} error={false}>
          {hint}
        </FormHelperText>
      ) : null}
      {error ? <FormHelperText id={`${field}-error`}>{error}</FormHelperText> : null}
    </FormControl>
  );
}
