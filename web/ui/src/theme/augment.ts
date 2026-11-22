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
 * What the theme of Kampus adds to MUI's types: the container roles of every colour, the accent and brand colours,
 * the surfaces, the corners after the medium one, and the tonal variant of buttons.
 */

import "@mui/material/styles";
import "@mui/material/Button";
import "@mui/material/Chip";

/** The surfaces of a scheme and the colours read on them. */
export interface SurfaceRoles {
  /** The page. */
  main: string;
  /** The surfaces from the lowest to the highest. */
  containerLowest: string;
  containerLow: string;
  container: string;
  containerHigh: string;
  containerHighest: string;
  /** Text on a surface, and quieter text. */
  on: string;
  onVariant: string;
  /** The edge of a control, and a quieter divider. */
  outline: string;
  outlineVariant: string;
  /** The inverse surface (toasts), its text and the primary colour on it. */
  inverse: string;
  onInverse: string;
  inversePrimary: string;
  /** What darkens the page behind a dialog. */
  scrim: string;
  /** The colour of shadows. */
  shadow: string;
}

/** The gradients of Didactika's theme. */
export interface GradientRoles {
  /** The main action and progress: the primary colour towards information. */
  primary: string;
  /** The bar of the brand at the top of every page. */
  bar: string;
  /** A soft wash behind something the person points at. */
  soft: string;
}

declare module "@mui/material/styles" {
  interface PaletteColor {
    /** A softer background of the colour. */
    container: string;
    /** Text on that background. */
    onContainer: string;
  }
  interface SimplePaletteColorOptions {
    container?: string;
    onContainer?: string;
  }
  interface Palette {
    accent: PaletteColor;
    brand: PaletteColor;
    surface: SurfaceRoles;
    gradient: GradientRoles;
  }
  interface PaletteOptions {
    accent?: SimplePaletteColorOptions;
    brand?: SimplePaletteColorOptions;
    surface?: Partial<SurfaceRoles>;
    gradient?: Partial<GradientRoles>;
  }
  interface Shape {
    /** Fields, menus and chips. */
    small: number;
    /** Sheets and large cards. */
    large: number;
    /** Dialogs. */
    extraLarge: number;
    /** Pills: buttons and badges; 0 when the theme wants square shapes. */
    full: number;
  }
  interface ShapeOptions {
    small?: number;
    large?: number;
    extraLarge?: number;
    full?: number;
  }
}

declare module "@mui/material/Button" {
  interface ButtonPropsVariantOverrides {
    tonal: true;
  }
  interface ButtonPropsColorOverrides {
    accent: true;
    brand: true;
  }
}

declare module "@mui/material/Chip" {
  interface ChipPropsColorOverrides {
    accent: true;
    brand: true;
  }
}
