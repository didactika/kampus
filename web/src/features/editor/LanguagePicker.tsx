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
 * The language a text is being written in, chosen from every language installed: a campus may install a hundred, so
 * the choice is a list that searches by name or code and marks the languages that already have text, never a row of
 * buttons. Compact, it is a small pill that floats over the text it belongs to.
 */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import InputBase from "@mui/material/InputBase";
import MenuItem from "@mui/material/MenuItem";
import MenuList from "@mui/material/MenuList";
import Popover from "@mui/material/Popover";
import Typography from "@mui/material/Typography";
import { useMemo, useRef, useState } from "react";

import { Icon } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** How many languages a list shows before it offers to search. */
const SEARCH_FROM = 6;

/** The props of `LanguagePicker`. */
export interface LanguagePickerProps {
  /** The languages to choose from, by code. */
  readonly languages: readonly string[];
  /** The language chosen. */
  readonly value: string;
  readonly onChange: (language: string) => void;
  /** The languages that already have text. */
  readonly filled?: ReadonlySet<string>;
  /** Whether it is the small pill of a text edited in place. */
  readonly compact?: boolean;
  /** Called when its list opens or closes, so what holds it may keep it visible. */
  readonly onOpen?: (open: boolean) => void;
  /** What it is called, given the name of the language chosen; the language of a text when not given. */
  readonly label?: (name: string) => string;
}

/** `<LanguagePicker languages={LANGUAGES} value="es" onChange={setLanguage} filled={new Set(["en"])} />` */
export function LanguagePicker({ languages, value, onChange, filled = new Set(), compact = false, onOpen, label }: LanguagePickerProps) {
  const { t, language } = useI18n();
  const anchor = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const names = useMemo(() => {
    const display = new Intl.DisplayNames([language], { type: "language" });
    // Names as a list shows them, with a capital, whatever the casing of the language that names them.
    const named = (code: string) => {
      const name = display.of(code) ?? code;
      return name.charAt(0).toLocaleUpperCase(language) + name.slice(1);
    };
    return new Map(languages.map((code) => [code, named(code)]));
  }, [languages, language]);
  const toggle = (next: boolean) => {
    setOpen(next);
    onOpen?.(next);
    if (!next) setQuery("");
  };
  const wanted = query.trim().toLowerCase();
  const shown = languages.filter((code) => !wanted || code.toLowerCase().includes(wanted) || (names.get(code) ?? "").toLowerCase().includes(wanted));
  return (
    <>
      <ButtonBase
        ref={anchor}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={label ? label(names.get(value) ?? value) : t("editor.language", { language: names.get(value) ?? value })}
        onMouseDown={(event) => event.preventDefault()}
        onClick={() => toggle(true)}
        sx={{
          gap: 0.5,
          px: compact ? 1 : 1.5,
          py: compact ? 0.25 : 0.75,
          borderRadius: 999,
          border: 1,
          borderColor: "divider",
          bgcolor: "background.paper",
          color: "text.primary",
          boxShadow: compact ? 3 : 0,
          typography: compact ? "caption" : "body2",
          fontWeight: 600,
        }}
      >
        <Icon name="translate" size="small" />
        {compact ? value.toUpperCase() : (names.get(value) ?? value)}
        <Icon name="keyboard_arrow_down" size="small" />
      </ButtonBase>
      <Popover
        open={open}
        anchorEl={anchor.current}
        onClose={() => toggle(false)}
        anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
        transformOrigin={{ vertical: "top", horizontal: "right" }}
        slotProps={{ paper: { sx: { inlineSize: 280, maxBlockSize: 360, display: "flex", flexDirection: "column" } } }}
      >
        {languages.length > SEARCH_FROM ? (
          <Box sx={{ display: "flex", alignItems: "center", gap: 1, px: 1.5, py: 1, borderBottom: 1, borderColor: "divider" }}>
            <Icon name="search" size="small" />
            <InputBase autoFocus fullWidth placeholder={t("editor.language.search")} value={query} onChange={(event) => setQuery(event.target.value)} inputProps={{ "aria-label": t("editor.language.search") }} />
          </Box>
        ) : null}
        <MenuList role="listbox" aria-label={t("editor.languages")} dense sx={{ overflowY: "auto" }} autoFocusItem={languages.length <= SEARCH_FROM}>
          {shown.map((code) => (
            <MenuItem
              key={code}
              role="option"
              selected={code === value}
              aria-selected={code === value}
              onClick={() => {
                onChange(code);
                toggle(false);
              }}
              sx={{ gap: 1.5 }}
            >
              <Typography variant="caption" sx={{ inlineSize: 28, fontWeight: 700, color: "text.secondary", textTransform: "uppercase" }}>
                {code}
              </Typography>
              <Typography variant="body2" sx={{ flex: 1 }}>
                {names.get(code)}
              </Typography>
              {filled.has(code) ? (
                <Box sx={{ color: "success.main", display: "flex" }} role="img" aria-label={t("editor.language.written")}>
                  <Icon name="check" size="small" />
                </Box>
              ) : null}
            </MenuItem>
          ))}
          {shown.length === 0 ? (
            <Typography variant="body2" color="text.secondary" sx={{ px: 2, py: 1 }}>
              {t("editor.language.none")}
            </Typography>
          ) : null}
        </MenuList>
      </Popover>
    </>
  );
}

/** The languages a value has text in. */
export function filledIn(value: unknown): ReadonlySet<string> {
  if (!value || typeof value !== "object") return new Set();
  return new Set(Object.entries(value as Record<string, unknown>).filter(([, text]) => typeof text === "string" && text.trim() && text !== "<p></p>").map(([code]) => code));
}
