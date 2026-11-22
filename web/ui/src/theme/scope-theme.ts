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


/**
 * The theme of a scope as the server gives it (`GET /api/v1/customise/scopes/{scope}/theme`): its tokens and the
 * colour roles of its light and dark schemes. Didactika's theme, the default, comes from the token files of the
 * design system, which a test of the server keeps equal to what it grows for the default tokens.
 */

import dark from "../../tokens/dark.tokens.json";
import light from "../../tokens/light.tokens.json";
import reference from "../../tokens/reference.tokens.json";
import { roles, type TokenNode } from "../tokens/tokens.js";

/** The colour roles of one scheme, by name (`primary`, `on-primary-container`, `surface-container-high`…). */
export type Roles = Readonly<Record<string, string>>;

/** The tokens of a theme other than colours. */
export interface ThemeTokens {
  /** The fonts of text. */
  readonly font_family: string;
  /** The fonts of display, headlines and large titles. */
  readonly font_family_brand: string;
  /** The size of normal text, in pixels. */
  readonly font_size: number;
  /** The medium corner, in pixels; every other corner follows it. */
  readonly radius: number;
  /** Comfortable or compact controls. */
  readonly density: "comfortable" | "compact";
  /** Whether things move. */
  readonly motion: "full" | "reduced";
  /** The scheme the scope prefers. */
  readonly scheme: "system" | "light" | "dark";
}

/** The theme of a scope. */
export interface ScopeTheme {
  /** Its tokens; those missing take Didactika's. */
  readonly tokens: Partial<ThemeTokens>;
  /** The roles of its light scheme. */
  readonly light: Roles;
  /** The roles of its dark scheme. */
  readonly dark: Roles;
}

/** The roles of a scheme document, by name. */
function rolesOf(scheme: unknown): Roles {
  return Object.fromEntries(
    roles(reference as TokenNode, scheme as TokenNode).map((token) => [token.path.slice("color.".length), String(token.value)]),
  );
}

/** Didactika's tokens, the defaults of every theme. */
export const DIDACTIKA_TOKENS: ThemeTokens = {
  font_family: '"IBM Plex Sans", system-ui, sans-serif',
  font_family_brand: '"Space Grotesk", "IBM Plex Sans", system-ui, sans-serif',
  font_size: 16,
  radius: 12,
  density: "comfortable",
  motion: "full",
  scheme: "system",
};

/** Didactika's theme. */
export const DIDACTIKA: ScopeTheme = { tokens: DIDACTIKA_TOKENS, light: rolesOf(light), dark: rolesOf(dark) };
