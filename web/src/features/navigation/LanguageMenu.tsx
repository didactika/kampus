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
 * The language of the person, in the menu of their account: an item that says the language in use and opens the list
 * of the installed languages, searchable when there are many. The choice is kept in their profile, so the campus speaks
 * it on every device, and the page is drawn again in it at once.
 */

import Box from "@mui/material/Box";
import InputBase from "@mui/material/InputBase";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { Icon, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Profile } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { LANGUAGES } from "../../i18n/translator";

/** How many languages the list shows before it offers to search. */
const SEARCH_FROM = 6;

/** The props of `LanguageMenu`. */
export interface LanguageMenuProps {
  readonly profile: Profile | null;
  /** Called once a language is chosen, so the menu around it closes. */
  readonly onDone: () => void;
}

/** `<LanguageMenu profile={profile} onDone={close} />` */
export function LanguageMenu({ profile, onDone }: LanguageMenuProps) {
  const { t, language } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  const [query, setQuery] = useState("");
  const names = useMemo(() => {
    const display = new Intl.DisplayNames([language], { type: "language" });
    return new Map(
      LANGUAGES.map((code) => {
        const name = display.of(code) ?? code;
        return [code, name.charAt(0).toLocaleUpperCase(language) + name.slice(1)];
      }),
    );
  }, [language]);
  if (!profile || LANGUAGES.length < 2) return null;
  const wanted = query.trim().toLowerCase();
  const shown = LANGUAGES.filter((code) => !wanted || code.includes(wanted) || (names.get(code) ?? "").toLowerCase().includes(wanted));

  async function choose(code: string) {
    setAnchor(null);
    onDone();
    if (!profile || code === language) return;
    try {
      await browserApi<Profile>("/users/me/profile", {
        method: "PUT",
        headers: { "If-Match": `"${profile.version}"` },
        body: {
          given_name: profile.given_name,
          family_name: profile.family_name,
          display_name: profile.display_name,
          language_code: code,
          time_zone: profile.time_zone,
        },
      });
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    }
  }

  return (
    <>
      <MenuItem aria-haspopup="menu" aria-expanded={Boolean(anchor)} onClick={(event) => setAnchor(event.currentTarget)}>
        <ListItemIcon>
          <Icon name="translate" size="small" />
        </ListItemIcon>
        <ListItemText primary={t("nav.language.title")} secondary={names.get(language) ?? language} />
        <Icon name="chevron_right" size="small" />
      </MenuItem>
      <Menu
        anchorEl={anchor}
        open={Boolean(anchor)}
        onClose={() => {
          setAnchor(null);
          setQuery("");
        }}
        anchorOrigin={{ vertical: "top", horizontal: "left" }}
        transformOrigin={{ vertical: "top", horizontal: "right" }}
        slotProps={{ paper: { sx: { minInlineSize: 220, maxBlockSize: 360 } } }}
      >
        {LANGUAGES.length > SEARCH_FROM ? (
          <Box sx={{ display: "flex", alignItems: "center", gap: 1, px: 1.5, pb: 1, borderBottom: 1, borderColor: "divider" }} onKeyDown={(event) => event.stopPropagation()}>
            <Icon name="search" size="small" />
            <InputBase autoFocus fullWidth placeholder={t("editor.language.search")} value={query} onChange={(event) => setQuery(event.target.value)} inputProps={{ "aria-label": t("editor.language.search") }} />
          </Box>
        ) : null}
        {shown.map((code) => (
          <MenuItem key={code} selected={code === language} onClick={() => void choose(code)} lang={code}>
            <ListItemText primary={names.get(code) ?? code} />
            {code === language ? <Icon name="check" size="small" /> : null}
          </MenuItem>
        ))}
      </Menu>
    </>
  );
}
