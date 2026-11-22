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
 * The MUI theme of a scope: Didactika's look made from the tokens and colour roles the server grows for the scope, in
 * a light and a dark scheme, as CSS variables (`--k-palette-primary-main`…) switched by `data-theme` on the root.
 */

import { createTheme, type PaletteOptions, type Theme } from "@mui/material/styles";

import "./augment";
import { componentStyles } from "./components";
import { DIDACTIKA, DIDACTIKA_TOKENS, type Roles, type ScopeTheme, type ThemeTokens } from "./scope-theme";

/** The colours with a role of their own, as the server names them, and as MUI's palette names them. */
const COLOURS = [
  ["primary", "primary"],
  ["secondary", "secondary"],
  ["accent", "accent"],
  ["danger", "error"],
  ["success", "success"],
  ["warning", "warning"],
  ["info", "info"],
] as const;

/** A role, or a visible mistake when a theme lacks it. */
function role(roles: Roles, name: string): string {
  return roles[name] ?? "#ff00ff";
}

/** The MUI palette of one scheme; papers sit one tone above the page, white in light and lighter in dark. */
export function paletteOf(roles: Roles, mode: "light" | "dark" = "light"): PaletteOptions {
  const colours = Object.fromEntries(
    COLOURS.map(([kampus, mui]) => [
      mui,
      {
        main: role(roles, kampus),
        contrastText: role(roles, `on-${kampus}`),
        container: role(roles, `${kampus}-container`),
        onContainer: role(roles, `on-${kampus}-container`),
      },
    ]),
  );
  return {
    ...colours,
    brand: { main: role(roles, "brand"), contrastText: role(roles, "on-brand") },
    background: {
      default: role(roles, "surface"),
      paper: role(roles, mode === "light" ? "surface-container-lowest" : "surface-container-low"),
    },
    text: { primary: role(roles, "on-surface"), secondary: role(roles, "on-surface-variant") },
    divider: role(roles, "outline-variant"),
    surface: {
      main: role(roles, "surface"),
      containerLowest: role(roles, "surface-container-lowest"),
      containerLow: role(roles, "surface-container-low"),
      container: role(roles, "surface-container"),
      containerHigh: role(roles, "surface-container-high"),
      containerHighest: role(roles, "surface-container-highest"),
      on: role(roles, "on-surface"),
      onVariant: role(roles, "on-surface-variant"),
      outline: role(roles, "outline"),
      outlineVariant: role(roles, "outline-variant"),
      inverse: role(roles, "inverse-surface"),
      onInverse: role(roles, "inverse-on-surface"),
      inversePrimary: role(roles, "inverse-primary"),
      scrim: role(roles, "scrim"),
      shadow: role(roles, "shadow"),
    },
  } as PaletteOptions;
}

/** The shadows of the five levels of elevation, tinted with the shadow role of the scheme. */
function shadows(): Theme["shadows"] {
  const tint = (alpha: number) => `color-mix(in srgb, var(--k-palette-surface-shadow) ${alpha}%, transparent)`;
  const levels = [
    "none",
    `0 1px 2px 0 ${tint(30)}, 0 1px 3px 1px ${tint(15)}`,
    `0 1px 2px 0 ${tint(30)}, 0 2px 6px 2px ${tint(15)}`,
    `0 1px 3px 0 ${tint(30)}, 0 4px 8px 3px ${tint(15)}`,
    `0 2px 3px 0 ${tint(30)}, 0 6px 10px 4px ${tint(15)}`,
    `0 4px 4px 0 ${tint(30)}, 0 8px 12px 6px ${tint(15)}`,
  ];
  return Array.from({ length: 25 }, (_, at) => levels[Math.min(at, levels.length - 1)]!) as Theme["shadows"];
}

/** The type scale: brand fonts for headings, text fonts for the rest, sizes in rem. */
function typography(tokens: ThemeTokens) {
  const brand = tokens.font_family_brand;
  const style = (fontFamily: string, size: number, line: number, fontWeight: number, tracking = 0) => ({
    fontFamily,
    fontSize: `${size}rem`,
    lineHeight: `${line}rem`,
    fontWeight,
    letterSpacing: `${tracking}rem`,
  });
  return {
    fontFamily: tokens.font_family,
    htmlFontSize: 16,
    h1: style(brand, 2, 2.5, 500),
    h2: style(brand, 1.75, 2.25, 500),
    h3: style(brand, 1.5, 2, 500),
    h4: style(brand, 1.375, 1.75, 500),
    h5: style(tokens.font_family, 1, 1.5, 500, 0.009375),
    h6: style(tokens.font_family, 0.875, 1.25, 500, 0.00625),
    subtitle1: style(tokens.font_family, 1, 1.5, 500, 0.009375),
    subtitle2: style(tokens.font_family, 0.875, 1.25, 500, 0.00625),
    body1: style(tokens.font_family, 1, 1.5, 400, 0.03125),
    body2: style(tokens.font_family, 0.875, 1.25, 400, 0.015625),
    caption: style(tokens.font_family, 0.75, 1, 400, 0.025),
    overline: { ...style(tokens.font_family, 0.6875, 1, 500, 0.03125), textTransform: "none" as const },
    button: { ...style(tokens.font_family, 0.875, 1.25, 500, 0.00625), textTransform: "none" as const },
  };
}

/** The MUI theme of a scope; Didactika's when no theme is given. */
export function createKampusTheme(scope: ScopeTheme = DIDACTIKA): Theme {
  const tokens: ThemeTokens = { ...DIDACTIKA_TOKENS, ...scope.tokens };
  const radius = tokens.radius;
  const still = tokens.motion === "reduced";
  const theme = createTheme({
    cssVariables: { colorSchemeSelector: "data-theme", cssVarPrefix: "k" },
    defaultColorScheme: tokens.scheme === "dark" ? "dark" : "light",
    colorSchemes: { light: { palette: paletteOf(scope.light, "light") }, dark: { palette: paletteOf(scope.dark, "dark") } },
    shape: {
      borderRadius: radius,
      small: Math.round((radius * 2) / 3),
      large: Math.round((radius * 4) / 3),
      extraLarge: Math.round((radius * 7) / 3),
      full: radius === 0 ? 0 : 999,
    },
    typography: typography(tokens),
    shadows: shadows(),
    transitions: still ? { create: () => "none" } : {},
    components: componentStyles({ compact: tokens.density === "compact", still, fontSize: tokens.font_size }),
  });
  return theme;
}
