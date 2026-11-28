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
 * The settings of a widget, in a dialog over the page: each setting by its kind, checked as it is typed, with a text
 * that takes one value per language written in each installed language and rich text in the rich editor. A setting
 * that may not be empty says so, and nothing applies while one is wrong. Before a widget goes in, the same dialog asks
 * for its settings.
 */

import Box from "@mui/material/Box";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import FormControlLabel from "@mui/material/FormControlLabel";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import Typography from "@mui/material/Typography";

import { Button, Icon, IconTile, SelectField, TextField } from "@kampus/ui";
import { useId, useState } from "react";

import type { DeclaredSetting } from "../../api/types";
import { LanguagePicker, filledIn } from "../editor/LanguagePicker";
import { RichTextEditor } from "../editor/RichTextEditor";
import { textIn, translated, withText } from "../editor/translated";
import { useI18n } from "../../i18n/I18nProvider";
import { LANGUAGES } from "../../i18n/translator";

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

/** The language every text starts in when it has none of its own. */
const FALLBACK = "en";

/** Settings whose text has a form of its own: a list of page sizes, such as "5, 10, 200". */
const FORMS: Readonly<Record<string, (text: string) => boolean>> = {
  sizes: (text) =>
    text.trim() === "" ||
    text.split(",").every((part) => /^\s*\d+\s*$/.test(part) && Number(part) >= 1 && Number(part) <= LARGEST_SIZE),
};

/** The texts of a value: itself, or one per language. */
const textsOf = (value: unknown): string[] =>
  typeof value === "string" ? [value] : value && typeof value === "object" ? Object.values(value).filter((v): v is string => typeof v === "string") : [];

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
    case "text":
    case "rich_text": {
      const { max } = setting.kind;
      const texts = textsOf(value);
      if (setting.required && texts.every((text) => !text.trim())) return t("setting.error.required");
      if (texts.some((text) => text.length > max)) return t("setting.error.long", { max });
      const form = FORMS[setting.key];
      if (form && texts.some((text) => !form(text))) return t(`setting.error.${setting.key}`, { max: LARGEST_SIZE });
      return undefined;
    }
    default:
      return undefined;
  }
}

/** `<WidgetSettings title="…" declared={…} values={…} onApply={…} onClose={…} />` */
export function WidgetSettings({ title, icon = "tune", declared, values, applyLabel, onApply, onClose }: WidgetSettingsProps) {
  const { t, language } = useI18n();
  const heading = useId();
  const [draft, setDraft] = useState<Record<string, unknown>>({ ...values });
  const [writing, setWriting] = useState(language);
  // A field says what is wrong once it was touched, or when the person tries to apply.
  const [touched, setTouched] = useState<ReadonlySet<string>>(new Set());
  const [tried, setTried] = useState(false);
  const shows = (key: string) => (tried || touched.has(key) ? problems[key] : undefined);
  const label = (key: string) => (t(`setting.${key}`) === `setting.${key}` ? key : t(`setting.${key}`));
  const set = (key: string, value: unknown) => {
    setDraft((d) => ({ ...d, [key]: value }));
    setTouched((t) => new Set([...t, key]));
  };
  const problems = Object.fromEntries(declared.map((s) => [s.key, problemOf(s, draft[s.key] ?? s.default, t)]));
  const valid = Object.values(problems).every((p) => p === undefined);
  const many = LANGUAGES.length > 1;
  // Numbers are typed as text and become numbers only when applied, so the person can type freely.
  const applied = () =>
    Object.fromEntries(declared.map((s) => [s.key, s.kind.type === "integer" ? Number(draft[s.key] ?? s.default) : (draft[s.key] ?? s.default)]));
  /** The text of a setting in the language being written, and how to write it. */
  const textOf = (setting: DeclaredSetting) => {
    const value = draft[setting.key] ?? setting.default;
    const per = Boolean(setting.translatable) && many;
    return {
      // A language with no text of its own starts from the text it reads now.
      text: (per ? textIn(value, writing, FALLBACK) : "") || translated(value, per ? writing : language),
      write: (text: string) => set(setting.key, per ? withText(value, writing, text, FALLBACK) : text),
      per,
    };
  };
  return (
    <Dialog open onClose={onClose} maxWidth="sm" fullWidth aria-labelledby={heading}>
      <DialogTitle component="div" sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
        <IconTile name={icon} />
        <Typography id={heading} variant="h4" component="h2" sx={{ flex: 1 }}>
          {title}
        </Typography>
        {many && declared.some((s) => s.translatable) ? (
          <LanguagePicker
            languages={LANGUAGES}
            value={writing}
            onChange={setWriting}
            filled={new Set(declared.filter((s) => s.translatable).flatMap((s) => [...filledIn(draft[s.key])]))}
          />
        ) : null}
        <IconButton aria-label={t("dashboard.cancel")} onClick={onClose}>
          <Icon name="close" />
        </IconButton>
      </DialogTitle>
      <DialogContent>
        <Box
          component="form"
          id={`${heading}-form`}
          onSubmit={(event) => {
            event.preventDefault();
            setTried(true);
            if (valid) onApply(applied());
          }}
          sx={{ pt: 1 }}
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
                      hint={shows(setting.key) ? undefined : t("setting.range", { min, max })}
                      error={shows(setting.key)}
                      onChange={(e) => set(setting.key, e.target.value)}
                    />
                  );
                }
                case "rich_text": {
                  const { text, write } = textOf(setting);
                  return (
                    <Box key={setting.key}>
                      <Typography variant="subtitle2" component="p" sx={{ mb: 0.75 }}>
                        {label(setting.key)}
                        {setting.required ? " *" : ""}
                      </Typography>
                                            <RichTextEditor label={label(setting.key)} value={text} onChange={write} />
                      {shows(setting.key) ? (
                        <Typography variant="caption" color="error" role="alert">
                          {shows(setting.key)}
                        </Typography>
                      ) : null}
                    </Box>
                  );
                }
                default: {
                  const { max } = setting.kind;
                  const { text, write } = textOf(setting);
                  return (
                    <Box key={setting.key}>
                                            <TextField
                        label={label(setting.key)}
                        required={Boolean(setting.required)}
                        value={text}
                        multiline={max > 200}
                        minRows={max > 200 ? 4 : undefined}
                        inputProps={{ maxLength: max }}
                        error={shows(setting.key)}
                        onChange={(e) => write(e.target.value)}
                      />
                    </Box>
                  );
                }
              }
            })}
          </Stack>
        </Box>
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 2.5 }}>
        <Button variant="text" onClick={onClose}>
          {t("dashboard.cancel")}
        </Button>
        <Button variant="filled" type="submit" form={`${heading}-form`}>
          {applyLabel ?? t("dashboard.apply")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
