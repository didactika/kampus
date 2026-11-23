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
 * The settings of a widget, as a form made from what its module declares: a text field for text (several lines for a
 * long one), a switch for on or off, a list for a choice, a number within its bounds.
 */

import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import FormControlLabel from "@mui/material/FormControlLabel";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import { useId, useState } from "react";

import { Button, SelectField, TextField } from "@kampus/ui";

import type { DeclaredSetting } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";

/** The props of `WidgetSettings`. */
export interface WidgetSettingsProps {
  readonly title: string;
  readonly declared: readonly DeclaredSetting[];
  readonly values: Readonly<Record<string, unknown>>;
  readonly onApply: (values: Record<string, unknown>) => void;
  readonly onClose: () => void;
}

/** `<WidgetSettings title="…" declared={…} values={…} onApply={…} onClose={…} />` */
export function WidgetSettings({ title, declared, values, onApply, onClose }: WidgetSettingsProps) {
  const { t } = useI18n();
  const heading = useId();
  const [draft, setDraft] = useState<Record<string, unknown>>({ ...values });
  const label = (key: string) => (t(`setting.${key}`) === `setting.${key}` ? key : t(`setting.${key}`));
  const set = (key: string, value: unknown) => setDraft((d) => ({ ...d, [key]: value }));
  return (
    <Dialog open onClose={onClose} maxWidth="xs" fullWidth aria-labelledby={heading}>
      <DialogTitle id={heading}>{title}</DialogTitle>
      <DialogContent>
        <Stack spacing={2.5} sx={{ pt: 1 }}>
          {declared.map((setting) => {
            const value = draft[setting.key] ?? setting.default;
            switch (setting.kind.type) {
              case "boolean":
                return (
                  <FormControlLabel
                    key={setting.key}
                    control={<Switch checked={Boolean(value)} onChange={(e) => set(setting.key, e.target.checked)} />}
                    label={label(setting.key)}
                  />
                );
              case "choice":
                return (
                  <SelectField
                    key={setting.key}
                    label={label(setting.key)}
                    value={String(value)}
                    onChange={(next) => set(setting.key, next)}
                    options={setting.kind.options.map((o) => ({ value: o, label: t(`choice.${o}`) === `choice.${o}` ? o : t(`choice.${o}`) }))}
                  />
                );
              case "integer": {
                const { min, max } = setting.kind;
                return (
                  <TextField
                    key={setting.key}
                    label={label(setting.key)}
                    type="number"
                    value={String(value)}
                    inputProps={{ min, max }}
                    hint={`${min}–${max}`}
                    onChange={(e) => set(setting.key, Math.min(max, Math.max(min, Number(e.target.value) || min)))}
                  />
                );
              }
              default: {
                const { max } = setting.kind;
                return (
                  <TextField
                    key={setting.key}
                    label={label(setting.key)}
                    value={String(value ?? "")}
                    multiline={max > 200}
                    minRows={max > 200 ? 4 : undefined}
                    inputProps={{ maxLength: max }}
                    onChange={(e) => set(setting.key, e.target.value)}
                  />
                );
              }
            }
          })}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="text" onClick={onClose}>
          {t("dashboard.cancel")}
        </Button>
        <Button variant="filled" onClick={() => onApply(draft)}>
          {t("dashboard.apply")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
