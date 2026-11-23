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
 * Editing the texts of a widget on the widget itself. While its page is arranged, the frame of a widget gives its
 * settings to what it renders; a text marked as editable then becomes a field in the same place and the same
 * typography, and a rich text an editor, in every language installed when the setting is translatable. Outside the
 * editor they only show their text, in the language of the reader.
 */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import InputBase from "@mui/material/InputBase";
import Stack from "@mui/material/Stack";
import Typography, { type TypographyProps } from "@mui/material/Typography";
import { createContext, useContext, useState, type ReactNode, type SyntheticEvent } from "react";

import type { DeclaredSetting } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { LANGUAGES } from "../../i18n/translator";
import { RichText } from "./RichText";
import { RichTextEditor } from "./RichTextEditor";
import { textIn, translated, withText } from "./translated";

/** The settings of a widget being arranged, and how to change one. */
export interface EditInPlaceValue {
  readonly settings: Readonly<Record<string, unknown>>;
  readonly declared: readonly DeclaredSetting[];
  readonly set: (key: string, value: unknown) => void;
}

const EditInPlaceContext = createContext<EditInPlaceValue | null>(null);

/** Gives the settings of a widget being arranged to what it renders. */
export function EditInPlaceProvider({ value, children }: { readonly value: EditInPlaceValue | null; readonly children: ReactNode }) {
  return <EditInPlaceContext.Provider value={value}>{children}</EditInPlaceContext.Provider>;
}

/** The language every text starts in when it has none of its own. */
const FALLBACK = "en";

/** Keeps a press or a key inside a field from moving or selecting the widget around it. */
const contained = {
  onPointerDown: (event: SyntheticEvent) => event.stopPropagation(),
  onKeyDown: (event: SyntheticEvent) => event.stopPropagation(),
};

/** What every editable text needs: its value now, the language being written, and how to write it. */
function useEditable(setting: string, given: unknown) {
  const edit = useContext(EditInPlaceContext);
  const { language } = useI18n();
  const [writing, setWriting] = useState(language);
  const declared = edit?.declared.find((d) => d.key === setting);
  const value = edit ? (edit.settings[setting] ?? given) : given;
  const translatable = Boolean(declared?.translatable) && LANGUAGES.length > 1;
  const write = (text: string) => edit?.set(setting, translatable ? withText(value, writing, text, FALLBACK) : text);
  return { edit, value, writing, setWriting, translatable, required: Boolean(declared?.required), write, shown: translated(value, language) };
}

/** The languages a text is written in, to switch between them without leaving the page. */
function Languages({ writing, onWrite }: { readonly writing: string; readonly onWrite: (language: string) => void }) {
  const { t } = useI18n();
  return (
    <Stack
      direction="row"
      role="group"
      aria-label={t("editor.languages")}
      className="edit-languages"
      sx={{ position: "absolute", insetBlockEnd: "100%", insetInlineEnd: 0, mb: 0.5, zIndex: 3, gap: 0.25, p: 0.25, borderRadius: 999, bgcolor: "background.paper", color: "text.primary", border: 1, borderColor: "divider", boxShadow: 3 }}
    >
      {LANGUAGES.map((code) => (
        <ButtonBase
          key={code}
          aria-pressed={code === writing}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => onWrite(code)}
          sx={{ px: 1, py: 0.25, borderRadius: 999, typography: "caption", fontWeight: 700, textTransform: "uppercase", ...(code === writing ? { bgcolor: "primary.main", color: "primary.contrastText" } : {}) }}
        >
          {code}
        </ButtonBase>
      ))}
    </Stack>
  );
}

/** The look of a text that can be edited: marked on hover, outlined while it has the focus, red when it may not be empty. */
const editable = (wrong: boolean) => ({
  position: "relative",
  pointerEvents: "auto",
  cursor: "text",
  borderRadius: 1,
  outline: "1px dashed transparent",
  outlineOffset: 4,
  transition: "outline-color 120ms",
  "&:hover": { outlineColor: "currentColor" },
  "&:focus-within": { outline: "2px solid", outlineColor: wrong ? "error.main" : "currentColor" },
  "& .edit-languages": { display: "none" },
  "&:focus-within .edit-languages": { display: "flex" },
}) as const;

/** The props of `EditableText`. */
export interface EditableTextProps {
  /** Its typography. */
  readonly variant?: TypographyProps["variant"];
  /** The element it is, such as `h2`. */
  readonly component?: "h1" | "h2" | "h3" | "h4" | "p" | "span";
  readonly id?: string;
  readonly sx?: TypographyProps["sx"];
  /** The key of the setting it edits. */
  readonly setting: string;
  /** Its value as the server gave it: a text, or one per language. */
  readonly value: unknown;
  /** What it shows when it is empty, such as a greeting; nothing when there is none. */
  readonly fallback?: string;
  /** What it says while it is written and empty; the fallback when there is none. */
  readonly placeholder?: string;
}

/** `<EditableText setting="heading" value={settings.heading} fallback={t("home.greeting")} variant="h1" />` */
export function EditableText({ setting, value: given, fallback = "", placeholder, variant, component = "p", id, sx }: EditableTextProps) {
  const { t } = useI18n();
  const { edit, value, writing, setWriting, translatable, required, write, shown } = useEditable(setting, given);
  const [wrong, setWrong] = useState(false);
  if (!edit) {
    if (!shown && !fallback) return null;
    return (
      <Typography variant={variant} component={component} id={id} sx={sx}>
        {shown || fallback}
      </Typography>
    );
  }
  // A language with no text of its own starts from the text it reads now; it keeps its own once it is written.
  const current = (translatable ? textIn(value, writing, FALLBACK) : "") || translated(value, writing);
  return (
    <Typography variant={variant} component="div" id={id} className="editable-text" sx={[editable(wrong), ...(Array.isArray(sx) ? sx : [sx])]} {...contained}>
      {translatable ? <Languages writing={writing} onWrite={setWriting} /> : null}
      <InputBase
        fullWidth
        multiline
        value={current}
        placeholder={placeholder ?? fallback}
        inputProps={{ "aria-label": t("editor.text", { setting }), "aria-invalid": wrong }}
        onChange={(event) => {
          const text = event.target.value.replace(/\n/g, " ");
          // A text that may not be empty keeps its last value until something is written.
          const empty = !text.trim();
          setWrong(required && empty);
          if (!(required && empty)) write(text);
        }}
        sx={{ font: "inherit", color: "inherit", letterSpacing: "inherit", p: 0, "& textarea": { font: "inherit", lineHeight: "inherit", p: 0, "&::placeholder": { color: "inherit", opacity: 0.6 } } }}
      />
      {wrong ? (
        <Box component="span" role="alert" sx={{ display: "block", typography: "caption", color: "error.main" }}>
          {t("setting.error.required")}
        </Box>
      ) : null}
    </Typography>
  );
}

/** The props of `EditableRichText`. */
export interface EditableRichTextProps {
  readonly setting: string;
  readonly value: unknown;
  readonly placeholder?: string;
}

/** `<EditableRichText setting="text" value={settings.text} />` */
export function EditableRichText({ setting, value: given, placeholder = "" }: EditableRichTextProps) {
  const { t } = useI18n();
  const { edit, value, writing, setWriting, translatable, required, write, shown } = useEditable(setting, given);
  const [wrong, setWrong] = useState(false);
  if (!edit) return shown ? <RichText html={shown} /> : null;
  // A language with no text of its own starts from the text it reads now; it keeps its own once it is written.
  const current = (translatable ? textIn(value, writing, FALLBACK) : "") || translated(value, writing);
  return (
    <Box className="editable-text" sx={editable(wrong)} {...contained}>
      {translatable ? <Languages writing={writing} onWrite={setWriting} /> : null}
      <RichTextEditor
        bare
        label={t("editor.text", { setting })}
        value={current}
        placeholder={placeholder || t("editor.write")}
        onChange={(html) => {
          setWrong(required && !html);
          if (!(required && !html)) write(html);
        }}
      />
      {wrong ? (
        <Box component="span" role="alert" sx={{ display: "block", typography: "caption", color: "error.main" }}>
          {t("setting.error.required")}
        </Box>
      ) : null}
    </Box>
  );
}
