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


/** The provider every page of Kampus is wrapped in: the MUI theme of its scope, its baseline and its scheme. */

import CssBaseline from "@mui/material/CssBaseline";
import { ThemeProvider } from "@mui/material/styles";
import { useMemo, type ReactNode } from "react";

import { createKampusTheme } from "./create-theme.js";
import { DIDACTIKA, type ScopeTheme } from "./scope-theme.js";

/** The props of `KampusThemeProvider`. */
export interface KampusThemeProviderProps {
  /** The theme of the scope of the page; Didactika's when none is given. */
  readonly theme?: ScopeTheme;
  /** The page. */
  readonly children: ReactNode;
}

/** Applies the theme of a scope: light, dark or the system's, as the scope prefers until the person chooses. */
export function KampusThemeProvider({ theme = DIDACTIKA, children }: KampusThemeProviderProps) {
  const mui = useMemo(() => createKampusTheme(theme), [theme]);
  const mode = theme.tokens.scheme ?? "system";
  return (
    <ThemeProvider theme={mui} defaultMode={mode} modeStorageKey="kampus-scheme">
      <CssBaseline enableColorScheme />
      {children}
    </ThemeProvider>
  );
}
