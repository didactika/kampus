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

/** The language of the page for the components of the browser, chosen once by the server. */

import { createContext, useContext, useMemo, type ReactNode } from "react";

import { formatDate, formatDay, formatNumber, formatRelative, translator, type Translate } from "./translator";

/** What a component needs to speak the language of the page. */
export interface I18n {
  /** The language. */
  readonly language: string;
  /** Its messages. */
  readonly t: Translate;
  /** A date and time in it. */
  readonly date: (iso: string) => string;
  /** A day in it. */
  readonly day: (iso: string) => string;
  /** How long ago a moment was, in it. */
  readonly relative: (iso: string) => string;
  /** A number in it. */
  readonly number: (value: string | number) => string;
}

const I18nContext = createContext<I18n | null>(null);

/** Gives the language of the page to everything inside. */
export function I18nProvider({ language, children }: { readonly language: string; readonly children: ReactNode }) {
  const value = useMemo<I18n>(
    () => ({
      language,
      t: translator(language),
      date: (iso) => formatDate(language, iso),
      day: (iso) => formatDay(language, iso),
      relative: (iso) => formatRelative(language, iso),
      number: (n) => formatNumber(language, n),
    }),
    [language],
  );
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

/** The language of the page. */
export function useI18n(): I18n {
  const i18n = useContext(I18nContext);
  if (!i18n) throw new Error("useI18n needs an I18nProvider around it");
  return i18n;
}
