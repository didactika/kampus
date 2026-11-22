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


/**
 * The theme of a scope, read on the server before the page renders, so the first paint already has it. A scope whose
 * theme cannot be read shows Didactika's.
 */

import { cache } from "react";

import { DIDACTIKA, type ScopeTheme } from "@kampus/ui";

import { optional } from "../api/server";

/** What the API answers for a theme, which an older server may give in part. */
type Answer = Partial<{ tokens: Record<string, unknown>; light: Record<string, string>; dark: Record<string, string> }>;

/** A theme read from the API, with whatever it lacks taken from Didactika's. */
export function completed(answer: Answer | null): ScopeTheme {
  return {
    tokens: { ...DIDACTIKA.tokens, ...(answer?.tokens ?? {}) },
    light: { ...DIDACTIKA.light, ...(answer?.light ?? {}) },
    dark: { ...DIDACTIKA.dark, ...(answer?.dark ?? {}) },
  };
}

/** The theme of a scope, once per request. */
export const scopeTheme = cache(async (scope: string | null | undefined): Promise<ScopeTheme> => {
  if (!scope) return DIDACTIKA;
  return completed(await optional<Answer | null>(`/customise/scopes/${encodeURIComponent(scope)}/theme`, null));
});
