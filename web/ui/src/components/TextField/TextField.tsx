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
 * A text field with its label above it, as Didactika's forms have it, so the label is always read and never covers
 * the value. Its hint and its error sit under it and are tied to the field; an error from the server is shown until
 * the person types. It is a real input, so it takes part in the form around it.
 */

import FormControl from "@mui/material/FormControl";
import FormHelperText from "@mui/material/FormHelperText";
import FormLabel from "@mui/material/FormLabel";
import OutlinedInput, { type OutlinedInputProps } from "@mui/material/OutlinedInput";
import { useId } from "react";

/** The props of `TextField`. */
export interface TextFieldProps extends Omit<OutlinedInputProps, "error" | "label"> {
  /** The visible label; every field has one. */
  readonly label: string;
  /** Help under the field. */
  readonly hint?: string;
  /** An error to show, such as one the server returned. */
  readonly error?: string;
}

/** `<TextField label="Email" name="email" type="email" required />` */
export function TextField({ label, hint, error, id, required, disabled, fullWidth = true, ...input }: TextFieldProps) {
  const own = useId();
  const field = id ?? own;
  const described = [hint ? `${field}-hint` : "", error ? `${field}-error` : ""].filter(Boolean).join(" ") || undefined;
  return (
    <FormControl fullWidth={fullWidth} required={required} disabled={disabled} error={Boolean(error)}>
      <FormLabel htmlFor={field}>{label}</FormLabel>
      <OutlinedInput
        id={field}
        required={required}
        {...input}
        inputProps={{ ...input.inputProps, "aria-describedby": described, "aria-invalid": error ? true : undefined }}
      />
      {hint ? <FormHelperText id={`${field}-hint`} error={false}>{hint}</FormHelperText> : null}
      {error ? <FormHelperText id={`${field}-error`}>{error}</FormHelperText> : null}
    </FormControl>
  );
}
