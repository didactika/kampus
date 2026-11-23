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
 * The settings of a widget, in a panel at the side of the page so the widget stays in sight, as a form made from what
 * its module declares: a text field for text (several lines for a
 * long one), a switch for on or off, a list for a choice, a number within its bounds.
 */

import Box from "@mui/material/Box";
import Drawer from "@mui/material/Drawer";
import IconButton from "@mui/material/IconButton";
import Typography from "@mui/material/Typography";
import FormControlLabel from "@mui/material/FormControlLabel";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import { useId, useState } from "react";

import { Button, Icon, SelectField, TextField } from "@kampus/ui";

import type { DeclaredSetting } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";

/** The props of `WidgetSettings`. */
export interface WidgetSettingsProps {
  readonly title: string;
  /** The icon of the widget. */
  readonly icon?: string;
  readonly declared: readonly DeclaredSetting[];
  readonly values: Readonly<Record<string, unknown>>;
  /** The text of the button that applies them: "Apply" by default, "Add" before a widget goes in. */
  readonly applyLabel?: string;
  readonly onApply: (values: Record<string, unknown>) => void;
  readonly onClose: () => void;
}

/** The largest page size a list of sizes may offer. */
const LARGEST_SIZE = 500;

/** Settings whose text has a form of its own: a list of page sizes, such as "5, 10, 200". */
const FORMS: Readonly<Record<string, (text: string) => boolean>> = {
  sizes: (text) =>
    text.trim() === "" ||
    text.split(",").every((part) => /^\s*\d+\s*$/.test(part) && Number(part) >= 1 && Number(part) <= LARGEST_SIZE),
};

/** What is wrong with the value of a setting, in the language of the page; nothing when it is right. */
function problemOf(setting: DeclaredSetting, value: unknown, t: (key: string, values?: Record<string, string | number>) => string): string | undefined {
  switch (setting.kind.type) {
    case "integer": {
      const { min, max } = setting.kind;
      const text = String(value ?? "").trim();
      const number = Number(text);
      if (text === "" || !Number.isInteger(number) || number < min || number > max) return t("setting.error.range", { min, max });
      return undefined;
    }
    case "text": {
      const text = String(value ?? "");
      if (text.length > setting.kind.max) return t("setting.error.long", { max: setting.kind.max });
      const form = FORMS[setting.key];
      if (form && !form(text)) return t(`setting.error.${setting.key}`, { max: LARGEST_SIZE });
      return undefined;
    }
    default:
      return undefined;
  }
}

/** `<WidgetSettings title="…" declared={…} values={…} onApply={…} onClose={…} />` */
export function WidgetSettings({ title, icon = "tune", declared, values, applyLabel, onApply, onClose }: WidgetSettingsProps) {
  const { t } = useI18n();
  const heading = useId();
  const [draft, setDraft] = useState<Record<string, unknown>>({ ...values });
  const label = (key: string) => (t(`setting.${key}`) === `setting.${key}` ? key : t(`setting.${key}`));
  const set = (key: string, value: unknown) => setDraft((d) => ({ ...d, [key]: value }));
  const problems = Object.fromEntries(declared.map((s) => [s.key, problemOf(s, draft[s.key] ?? s.default, t)]));
  const valid = Object.values(problems).every((p) => p === undefined);
  // Numbers are typed as text and become numbers only when applied, so the person can type freely.
  const applied = () =>
    Object.fromEntries(declared.map((s) => [s.key, s.kind.type === "integer" ? Number(draft[s.key] ?? s.default) : (draft[s.key] ?? s.default)]));
  return (
    <Drawer
      open
      anchor="right"
      onClose={onClose}
      slotProps={{
        paper: {
          role: "dialog",
          "aria-labelledby": heading,
          sx: { inlineSize: 420, maxInlineSize: "100vw", display: "flex", flexDirection: "column", borderStartStartRadius: 20, borderEndStartRadius: 20 },
        },
      }}
    >
      <Stack direction="row" sx={{ alignItems: "center", gap: 1.5, px: 3, pt: 3, pb: 2 }}>
        <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, flex: "none", borderRadius: 2.5, backgroundImage: "var(--k-palette-gradient-soft)", color: "primary.main" }}>
          <Icon name={icon} />
        </Box>
        <Typography id={heading} variant="h4" component="h2" sx={{ flex: 1 }}>
          {title}
        </Typography>
        <IconButton aria-label={t("dashboard.cancel")} onClick={onClose}>
          <Icon name="close" />
        </IconButton>
      </Stack>
      <Box
        component="form"
        id={`${heading}-form`}
        onSubmit={(event) => {
          event.preventDefault();
          if (valid) onApply(applied());
        }}
        sx={{ flex: 1, overflowY: "auto", px: 3, py: 1 }}
      >
        <Stack spacing={2.5}>
          {declared.map((setting) => {
            const value = draft[setting.key] ?? setting.default;
            switch (setting.kind.type) {
              case "boolean":
                return (
                  <FormControlLabel
                    key={setting.key}
                    labelPlacement="start"
                    control={<Switch checked={Boolean(value)} onChange={(e) => set(setting.key, e.target.checked)} />}
                    label={label(setting.key)}
                    sx={{ mx: 0, justifyContent: "space-between", px: 2, py: 1, borderRadius: 3, border: 1, borderColor: "divider" }}
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
                    inputProps={{ min, max, inputMode: "numeric" }}
                    hint={problems[setting.key] ? undefined : t("setting.range", { min, max })}
                    error={problems[setting.key]}
                    onChange={(e) => set(setting.key, e.target.value)}
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
                    error={problems[setting.key]}
                    onChange={(e) => set(setting.key, e.target.value)}
                  />
                );
              }
            }
          })}
        </Stack>
      </Box>
      <Stack direction="row" spacing={1} sx={{ justifyContent: "flex-end", px: 3, py: 2, borderTop: 1, borderColor: "divider" }}>
        <Button variant="text" onClick={onClose}>
          {t("dashboard.cancel")}
        </Button>
        <Button variant="filled" type="submit" form={`${heading}-form`} disabled={!valid}>
          {applyLabel ?? t("dashboard.apply")}
        </Button>
      </Stack>
    </Drawer>
  );
}
