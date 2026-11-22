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
  const mix = (a: string, b: string, at: number) => `color-mix(in srgb, ${a} ${at}%, ${b})`;
  const primary = role(roles, "primary");
  const info = role(roles, "info");
  const brand = role(roles, "brand");
  return {
    ...colours,
    brand: { main: brand, contrastText: role(roles, "on-brand") },
    gradient: {
      primary: `linear-gradient(135deg, ${primary} 0%, ${mix(primary, info, 62)} 100%)`,
      bar:
        mode === "light"
          ? `linear-gradient(135deg, ${primary} 0%, ${mix(primary, info, 62)} 100%)`
          : `linear-gradient(135deg, ${brand} 0%, ${mix(brand, role(roles, "info-container"), 62)} 100%)`,
      soft: `linear-gradient(135deg, ${role(roles, "primary-container")} 0%, ${mix(info, role(roles, "surface-container-lowest"), mode === "light" ? 12 : 22)} 100%)`,
    },
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

/**
 * The shadows of the levels of elevation, tinted with the primary colour as Didactika's theme draws them: a fine
 * contact shadow and a wide, soft one of the brand.
 */
function shadows(): Theme["shadows"] {
  const ink = (alpha: number) => `color-mix(in srgb, var(--k-palette-surface-scrim) ${alpha}%, transparent)`;
  const tint = (alpha: number) => `color-mix(in srgb, var(--k-palette-primary-main) ${alpha}%, transparent)`;
  const levels = [
    "none",
    `0 1px 2px ${ink(6)}, 0 4px 12px -4px ${tint(14)}`,
    `0 2px 6px ${ink(6)}, 0 10px 28px ${tint(16)}`,
    `0 4px 12px ${ink(8)}, 0 18px 40px -10px ${tint(24)}`,
    `0 4px 12px ${ink(8)}, 0 24px 56px -12px ${tint(30)}`,
    `0 6px 16px ${ink(10)}, 0 32px 72px -16px ${tint(36)}`,
  ];
  return Array.from({ length: 25 }, (_, at) => levels[Math.min(at, levels.length - 1)]!) as Theme["shadows"];
}

/**
 * The type scale of Didactika's theme: Inter, headings in bold with tight tracking, buttons in semibold, text with an
 * open line height; sizes in rem, so they follow the size of the theme.
 */
function typography(tokens: ThemeTokens) {
  const heading = (size: number, line: number) => ({
    fontFamily: tokens.font_family_brand,
    fontSize: `${size}rem`,
    lineHeight: line,
    fontWeight: 700,
    letterSpacing: "-0.02em",
  });
  const text = (size: number, line: number, fontWeight = 400) => ({ fontSize: `${size}rem`, lineHeight: line, fontWeight });
  return {
    fontFamily: tokens.font_family,
    htmlFontSize: 16,
    h1: heading(2.25, 1.15),
    h2: heading(1.625, 1.25),
    h3: heading(1.375, 1.3),
    h4: heading(1.125, 1.35),
    h5: heading(1, 1.4),
    h6: heading(0.875, 1.4),
    subtitle1: text(1, 1.5, 600),
    subtitle2: text(0.875, 1.45, 600),
    body1: text(1, 1.6),
    body2: text(0.875, 1.55),
    caption: text(0.75, 1.45),
    overline: { ...text(0.6875, 1.4, 600), letterSpacing: "0.06em", textTransform: "uppercase" as const },
    button: { ...text(0.875, 1.4, 600), letterSpacing: "-0.005em", textTransform: "none" as const },
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
