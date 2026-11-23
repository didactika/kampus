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
 * The edit mode of the campus: a switch in the bar, shown only on a page the person may arrange (the home, a course),
 * that turns every such page into its editor. The mode is kept for the session as the person moves between pages,
 * and changes are saved as they are made, so turning it off is all it takes to finish.
 */

import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";

import { useI18n } from "../../i18n/I18nProvider";

/** Where the mode is kept for the session. */
const STORAGE_KEY = "kampus-edit-mode";

interface EditMode {
  readonly on: boolean;
  readonly offered: boolean;
  readonly turn: (on: boolean) => void;
  readonly offer: (yes: boolean) => void;
}

const EditModeContext = createContext<EditMode | null>(null);

/** Holds the edit mode of the pages under it. */
export function EditModeProvider({ children }: { readonly children: ReactNode }) {
  const [on, setOn] = useState(false);
  const [offers, setOffers] = useState(0);
  useEffect(() => {
    try {
      setOn(window.sessionStorage.getItem(STORAGE_KEY) === "1");
    } catch {
      // Without storage the mode simply starts off.
    }
  }, []);
  const turn = useCallback((next: boolean) => {
    setOn(next);
    try {
      window.sessionStorage.setItem(STORAGE_KEY, next ? "1" : "0");
    } catch {
      // Without storage the mode lasts until the page is left.
    }
  }, []);
  const offer = useCallback((yes: boolean) => setOffers((n) => n + (yes ? 1 : -1)), []);
  const value = useMemo(() => ({ on, offered: offers > 0, turn, offer }), [on, offers, turn, offer]);
  return <EditModeContext.Provider value={value}>{children}</EditModeContext.Provider>;
}

/** On a page that may be arranged by this person: offers the switch while mounted, and says whether the mode is on. */
export function useOfferEditMode(editable: boolean): boolean {
  const mode = useContext(EditModeContext);
  const offer = mode?.offer;
  useEffect(() => {
    if (!editable || !offer) return undefined;
    offer(true);
    return () => offer(false);
  }, [editable, offer]);
  return Boolean(editable && mode?.on);
}

/** The switch of the bar; nothing on a page the person cannot arrange. */
export function EditModeSwitch() {
  const { t } = useI18n();
  const mode = useContext(EditModeContext);
  if (!mode?.offered) return null;
  return (
    <FormControlLabel
      control={<Switch checked={mode.on} onChange={(event) => mode.turn(event.target.checked)} color="default" size="small" />}
      label={t("editmode.label")}
      sx={{
        mr: 1,
        color: "inherit",
        "& .MuiFormControlLabel-label": { fontSize: "0.875rem", fontWeight: 600, display: { xs: "none", sm: "block" } },
        "& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track": { backgroundColor: "#fff", opacity: 0.6 },
      }}
    />
  );
}
