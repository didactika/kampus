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


/** The language of the request a server component renders (the profile of the person first), and its messages and formatters. */

import { headers } from "next/headers";
import { cache } from "react";

import { optional } from "../api/server";
import type { Profile } from "../api/types";

import { LANGUAGES, directionOf, formatDate, formatDay, formatNumber, formatRelative, languageOf, translator } from "./translator";

/** The profile of the person of the request, once per request; none when nobody is signed in. */
const profile = cache(() => optional<Profile | null>("/users/me/profile", null));

/** The language of the request: the one of the profile of the person when the client speaks it, the browser's otherwise. */
const requestLanguage = cache(async () => {
  const chosen = (await profile())?.language_code?.toLowerCase();
  const spoken = chosen ? LANGUAGES.find((l) => l === chosen || chosen.startsWith(`${l}-`)) : undefined;
  return spoken ?? languageOf((await headers()).get("accept-language"));
});

/** The language of the request, its direction and its messages. */
export async function requestI18n() {
  const language = await requestLanguage();
  return {
    language,
    direction: directionOf(language),
    t: translator(language),
    date: (iso: string) => formatDate(language, iso),
    day: (iso: string) => formatDay(language, iso),
    relative: (iso: string) => formatRelative(language, iso),
    number: (value: string | number) => formatNumber(language, value),
  };
}
