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
// @package   web
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


"use client";

/** A password field whose value can be shown, so a person can check what they typed. */

import IconButton from "@mui/material/IconButton";
import InputAdornment from "@mui/material/InputAdornment";
import { useState } from "react";

import { Icon, TextField, type TextFieldProps } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** `<PasswordField name="password" label="Password" autoComplete="current-password" required />` */
export function PasswordField({ withIcon = false, ...field }: Omit<TextFieldProps, "type"> & { readonly withIcon?: boolean }) {
  const { t } = useI18n();
  const [shown, setShown] = useState(false);
  return (
    <TextField
      {...field}
      type={shown ? "text" : "password"}
      startAdornment={
        withIcon ? (
          <InputAdornment position="start">
            <Icon name="lock" size="small" />
          </InputAdornment>
        ) : undefined
      }
      endAdornment={
        <InputAdornment position="end">
          <IconButton edge="end" size="small" aria-label={shown ? t("signin.hide") : t("signin.show")} aria-pressed={shown} onClick={() => setShown((s) => !s)}>
            <Icon name={shown ? "visibility_off" : "visibility"} size="small" />
          </IconButton>
        </InputAdornment>
      }
    />
  );
}
