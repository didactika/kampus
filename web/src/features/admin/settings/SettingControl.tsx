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

/**
 * The field of a setting, built from its kind: a switch for on or off, the options side by side for a short choice and
 * a list for a long one, a number with its unit and bounds, a text. Its name and help are written beside it, so the
 * field is named by them.
 */

import InputAdornment from "@mui/material/InputAdornment";
import OutlinedInput from "@mui/material/OutlinedInput";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";

import { SelectField } from "@kampus/ui";

import type { ScopeSetting } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { optionName, settingName } from "../names";
import { unitOf } from "./format";

/** The most options shown side by side; more are a list. */
const SIDE_BY_SIDE = 4;

/** The props of `SettingControl`. */
export interface SettingControlProps {
  readonly setting: ScopeSetting;
  readonly value: unknown;
  readonly onChange: (value: unknown) => void;
  readonly disabled: boolean;
  /** The ids of its name and of its help. */
  readonly labelledBy: string;
  readonly describedBy: string;
  readonly invalid: boolean;
}

/** `<SettingControl setting={s} value={v} onChange={…} disabled={false} labelledBy="…" describedBy="…" invalid={false} />` */
export function SettingControl({ setting, value, onChange, disabled, labelledBy, describedBy, invalid }: SettingControlProps) {
  const { t } = useI18n();
  const kind = setting.kind;
  const aria = { "aria-labelledby": labelledBy, "aria-describedby": describedBy };
  if (kind.type === "boolean") {
    return (
      <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
        <Switch checked={value === true} disabled={disabled} onChange={(event) => onChange(event.target.checked)} slotProps={{ input: aria }} />
        <Typography variant="body2" color={value === true ? "text.primary" : "text.secondary"} sx={{ fontWeight: 600 }} aria-hidden>
          {value === true ? t("admin.on") : t("admin.off")}
        </Typography>
      </Stack>
    );
  }
  if (kind.type === "choice") {
    if (kind.options.length > SIDE_BY_SIDE) {
      return (
        <SelectField
          label={settingName(t, setting)}
          value={String(value ?? "")}
          disabled={disabled}
          onChange={onChange}
          options={kind.options.map((option) => ({ value: option, label: optionName(t, setting.key, option) }))}
        />
      );
    }
    return (
      <ToggleButtonGroup
        exclusive
        size="small"
        value={String(value ?? "")}
        disabled={disabled}
        onChange={(_, next: string | null) => {
          if (next !== null) onChange(next);
        }}
        {...aria}
        sx={{ flexWrap: "wrap", "& .MuiToggleButton-root": { textTransform: "none", px: 1.5, fontWeight: 600 } }}
      >
        {kind.options.map((option) => (
          <ToggleButton key={option} value={option}>
            {optionName(t, setting.key, option)}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    );
  }
  if (kind.type === "integer") {
    const unit = unitOf(setting.key);
    return (
      <OutlinedInput
        type="number"
        size="small"
        value={typeof value === "number" ? String(value) : ""}
        disabled={disabled}
        error={invalid}
        onChange={(event) => onChange(event.target.value === "" ? null : Number(event.target.value))}
        inputProps={{ min: kind.min, max: kind.max, step: 1, inputMode: "numeric", ...aria, "aria-invalid": invalid || undefined }}
        endAdornment={unit ? <InputAdornment position="end">{t(`admin.unit.${unit}.word`)}</InputAdornment> : undefined}
        sx={{ inlineSize: 200 }}
      />
    );
  }
  return (
    <OutlinedInput
      size="small"
      fullWidth
      value={typeof value === "string" ? value : ""}
      disabled={disabled}
      error={invalid}
      multiline={kind.type === "rich_text"}
      minRows={kind.type === "rich_text" ? 3 : undefined}
      onChange={(event) => onChange(event.target.value)}
      inputProps={{ maxLength: kind.max, ...aria, "aria-invalid": invalid || undefined }}
    />
  );
}
