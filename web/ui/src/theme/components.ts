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
 * Didactika's look of MUI's components, so they look the same whether a module uses them directly or through the
 * components of the design system: pill buttons that narrow while pressed, a tonal button, labels above fields,
 * tonal alerts and badges, cards and dialogs with the corners of the theme, and the scrim of the theme.
 */

import type { Components, Theme } from "@mui/material/styles";

/** What changes the components of a theme. */
export interface ComponentOptions {
  /** Smaller controls. */
  readonly compact: boolean;
  /** No motion. */
  readonly still: boolean;
  /** The size of normal text, in pixels. */
  readonly fontSize: number;
}

/** The palette of a theme as CSS variables. */
const vars = (theme: Theme) => theme.vars!.palette;

/** The colours a tonal or tinted element takes from the colour of a component. */
type Tone = "primary" | "secondary" | "accent" | "error" | "success" | "warning" | "info";

/** Every tone. */
const TONES: readonly Tone[] = ["primary", "secondary", "accent", "error", "success", "warning", "info"];

/** The colour of a component as a tone, `primary` when it has none. */
function toneOf(color: unknown): Tone {
  return ["secondary", "accent", "error", "success", "warning", "info"].includes(String(color)) ? (color as Tone) : "primary";
}

/** The style of the components of a theme. */
export function componentStyles({ compact, still, fontSize }: ComponentOptions): Components<Omit<Theme, "components">> {
  const size = compact ? "small" : "medium";
  return {
    MuiCssBaseline: {
      styleOverrides: (theme) => ({
        html: { fontSize: `${(fontSize * 100) / 16}%` },
        body: { backgroundColor: vars(theme).surface.main, color: vars(theme).surface.on },
        ...(still ? { "*, *::before, *::after": { transition: "none !important", animation: "none !important" } } : {}),
        "@media (prefers-reduced-motion: reduce)": {
          "*, *::before, *::after": { transitionDuration: "0s !important", animationDuration: "0s !important" },
        },
      }),
    },
    MuiButtonBase: { defaultProps: { disableRipple: still } },
    MuiButton: {
      defaultProps: { disableElevation: true, size, variant: "outlined" },
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.full,
          paddingInline: theme.spacing(3),
          minHeight: compact ? 32 : 40,
          transition: theme.transitions.create(["border-radius", "background-color", "box-shadow"], {
            duration: 350,
            easing: "cubic-bezier(0.42, 1.67, 0.21, 0.9)",
          }),
          "&:active:not(.Mui-disabled)": { borderRadius: theme.shape.small },
        }),
        sizeLarge: { minHeight: compact ? 48 : 56, fontSize: "1rem", paddingInline: 32 },
        sizeSmall: ({ theme }) => ({ minHeight: 32, paddingInline: theme.spacing(1.5) }),
        outlined: ({ theme }) => ({ borderColor: vars(theme).surface.outline }),
        text: ({ theme }) => ({ paddingInline: theme.spacing(1.5) }),
      },
      variants: TONES.map((tone) => ({
        props: { variant: "tonal" as const, color: tone },
        style: ({ theme }: { theme: Omit<Theme, "components"> }) => ({
          backgroundColor: vars(theme)[tone].container,
          color: vars(theme)[tone].onContainer,
          "&:hover": { backgroundColor: `color-mix(in srgb, ${vars(theme)[tone].onContainer} 8%, ${vars(theme)[tone].container})` },
          "&.Mui-disabled": { backgroundColor: `color-mix(in srgb, ${vars(theme).surface.on} 12%, transparent)` },
        }),
      })),
    },
    MuiIconButton: { defaultProps: { size } },
    MuiButtonGroup: { defaultProps: { disableElevation: true } },
    MuiTextField: { defaultProps: { size, fullWidth: true } },
    MuiOutlinedInput: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.small,
          "& .MuiOutlinedInput-notchedOutline": { borderColor: vars(theme).surface.outline },
        }),
      },
    },
    MuiFormLabel: {
      styleOverrides: {
        root: ({ theme }) => ({ ...theme.typography.subtitle2, color: vars(theme).surface.on, marginBottom: theme.spacing(0.5) }),
        asterisk: ({ theme }) => ({ color: vars(theme).error.main }),
      },
    },
    MuiFormHelperText: { styleOverrides: { root: ({ theme }) => ({ marginInline: theme.spacing(2) }) } },
    MuiCard: {
      defaultProps: { variant: "outlined" },
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.borderRadius,
          backgroundColor: vars(theme).background.paper,
        }),
      },
    },
    MuiPaper: {
      styleOverrides: {
        outlined: ({ theme }) => ({ borderColor: vars(theme).surface.outlineVariant }),
        rounded: ({ theme }) => ({ borderRadius: theme.shape.borderRadius }),
      },
    },
    MuiAlert: {
      defaultProps: { variant: "standard" },
      styleOverrides: {
        root: ({ theme, ownerState }) => {
          const tone = toneOf(ownerState.severity === "error" ? "error" : ownerState.severity);
          return {
            borderRadius: theme.shape.borderRadius,
            backgroundColor: vars(theme)[tone].container,
            color: vars(theme)[tone].onContainer,
            "& .MuiAlert-icon": { color: vars(theme)[tone].onContainer },
          };
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: ({ theme }) => ({ borderRadius: theme.shape.full, fontWeight: 500 }),
        outlined: ({ theme }) => ({ borderColor: vars(theme).surface.outline }),
      },
    },
    MuiDialog: {
      styleOverrides: {
        paper: ({ theme }) => ({
          borderRadius: theme.shape.extraLarge,
          backgroundColor: vars(theme).surface.containerHigh,
          backgroundImage: "none",
          padding: theme.spacing(1),
        }),
      },
    },
    MuiDialogTitle: { styleOverrides: { root: ({ theme }) => ({ ...theme.typography.h3 }) } },
    MuiDialogContentText: { styleOverrides: { root: ({ theme }) => ({ color: vars(theme).surface.onVariant }) } },
    MuiBackdrop: {
      styleOverrides: {
        root: ({ theme }) => ({
          "&:not(.MuiBackdrop-invisible)": { backgroundColor: `color-mix(in srgb, ${vars(theme).surface.scrim} 32%, transparent)` },
        }),
      },
    },
    MuiMenu: { styleOverrides: { paper: ({ theme }) => ({ borderRadius: theme.shape.small, backgroundColor: vars(theme).surface.container }) } },
    MuiTooltip: {
      styleOverrides: {
        tooltip: ({ theme }) => ({
          borderRadius: theme.shape.small,
          backgroundColor: vars(theme).surface.inverse,
          color: vars(theme).surface.onInverse,
          ...theme.typography.caption,
        }),
      },
    },
    MuiSnackbarContent: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.small,
          backgroundColor: vars(theme).surface.inverse,
          color: vars(theme).surface.onInverse,
          ...theme.typography.body2,
        }),
      },
    },
    MuiTabs: {
      styleOverrides: {
        root: ({ theme }) => ({ borderBottom: `1px solid ${vars(theme).surface.outlineVariant}` }),
        indicator: { height: 3, borderTopLeftRadius: 3, borderTopRightRadius: 3 },
      },
    },
    MuiTab: { styleOverrides: { root: ({ theme }) => ({ ...theme.typography.subtitle2, minHeight: compact ? 40 : 48 }) } },
    MuiTableCell: {
      styleOverrides: {
        root: ({ theme }) => ({ borderBottomColor: vars(theme).surface.outlineVariant }),
        head: ({ theme }) => ({ ...theme.typography.subtitle2, color: vars(theme).surface.onVariant }),
      },
    },
    MuiAppBar: {
      defaultProps: { elevation: 0, color: "inherit" },
      styleOverrides: { root: ({ theme }) => ({ backgroundColor: vars(theme).surface.container }) },
    },
    MuiLink: { defaultProps: { underline: "hover" } },
  };
}
