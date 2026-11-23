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
 * Didactika's look of MUI's components, as Didactika's theme draws them, so they look the same whether a module uses
 * them directly or through the components of the design system. Controls are 36 pixels tall with small corners;
 * the main action carries the gradient of the brand and rises with a tinted shadow; one focus ring for everything,
 * 3 pixels of the primary colour; fields tint on hover; cards have a hairline and a soft tinted shadow; dialogs,
 * menus and toasts float with deeper shadows; progress fills with the gradient.
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
  return TONES.includes(color as Tone) ? (color as Tone) : "primary";
}

/** One colour mixed into another. */
const mix = (a: string, b: string, at: number) => `color-mix(in srgb, ${a} ${at}%, ${b})`;

/** The focus ring of everything that takes focus. */
const ring = (theme: Theme) => `0 0 0 3px ${mix(vars(theme).primary.main, "transparent", 30)}`;

/** The easing of every change of state. */
const EASE = "140ms cubic-bezier(0.2, 0, 0, 1)";

/** The style of the components of a theme. */
export function componentStyles({ compact, still, fontSize }: ComponentOptions): Components<Omit<Theme, "components">> {
  const size = compact ? "small" : "medium";
  const control = compact ? 32 : 36;
  return {
    MuiCssBaseline: {
      styleOverrides: (theme) => ({
        html: { fontSize: `${(fontSize * 100) / 16}%` },
        body: {
          backgroundColor: vars(theme).surface.main,
          color: vars(theme).surface.on,
          fontFeatureSettings: '"calt", "cv11", "ss03"',
          fontOpticalSizing: "auto",
          WebkitFontSmoothing: "antialiased",
        },
        "::selection": { backgroundColor: mix(vars(theme).primary.main, "transparent", 22) },
        ...(still ? { "*, *::before, *::after": { transition: "none !important", animation: "none !important" } } : {}),
        "@media (prefers-reduced-motion: reduce)": {
          "*, *::before, *::after": { transitionDuration: "0s !important", animationDuration: "0s !important" },
        },
      }),
    },
    MuiButtonBase: {
      defaultProps: { disableRipple: true },
      styleOverrides: {
        root: ({ theme }) => ({
          "&.Mui-focusVisible": { boxShadow: ring(theme), outline: "none" },
        }),
      },
    },
    MuiButton: {
      defaultProps: { disableElevation: true, size, variant: "outlined" },
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.small,
          minHeight: control,
          paddingBlock: 0,
          paddingInline: theme.spacing(2),
          whiteSpace: "nowrap",
          // Every variant has the same border, transparent but for outlined, so a button changing variant keeps its size.
          border: "1px solid transparent",
          transition: `background-color ${EASE}, border-color ${EASE}, color ${EASE}, box-shadow 200ms cubic-bezier(0.2, 0, 0, 1), transform 120ms cubic-bezier(0.2, 0, 0, 1)`,
          "&:active:not(.Mui-disabled)": { transform: "translateY(0)" },
        }),
        contained: ({ theme, ownerState }) => {
          const tone = toneOf(ownerState.color);
          const main = vars(theme)[tone].main;
          return {
            ...(tone === "primary" ? { backgroundImage: vars(theme).gradient.primary } : {}),
            "&:hover:not(.Mui-disabled)": {
              transform: still ? "none" : "translateY(-1px)",
              boxShadow: `0 6px 16px -4px ${mix(main, "transparent", 55)}`,
              backgroundColor: main,
            },
            "&.Mui-disabled": { backgroundImage: "none" },
          };
        },
        outlined: ({ theme }) => ({
          borderColor: vars(theme).surface.outline,
          "&:hover:not(.Mui-disabled)": { backgroundColor: mix(vars(theme).primary.main, "transparent", 8) },
        }),
        text: ({ theme }) => ({
          paddingInline: theme.spacing(1.5),
          "&:hover:not(.Mui-disabled)": { backgroundColor: mix(vars(theme).primary.main, "transparent", 10) },
        }),
        sizeLarge: { minHeight: compact ? 40 : 44, fontSize: "0.9375rem", paddingInline: 22 },
        sizeSmall: ({ theme }) => ({ minHeight: 28, paddingInline: theme.spacing(1.25), fontSize: "0.8125rem" }),
      },
      variants: TONES.map((tone) => ({
        props: { variant: "tonal" as const, color: tone },
        style: ({ theme }: { theme: Omit<Theme, "components"> }) => ({
          backgroundColor: vars(theme)[tone].container,
          color: vars(theme)[tone].onContainer,
          "&:hover:not(.Mui-disabled)": { backgroundColor: mix(vars(theme)[tone].onContainer, vars(theme)[tone].container, 8) },
          "&.Mui-disabled": { backgroundColor: mix(vars(theme).surface.on, "transparent", 12) },
        }),
      })),
    },
    MuiIconButton: {
      defaultProps: { size },
      styleOverrides: {
        root: ({ theme }) => ({
          transition: `background-color ${EASE}, color ${EASE}`,
          "&:hover": { backgroundColor: mix(vars(theme).primary.main, "transparent", 12) },
        }),
      },
    },
    MuiButtonGroup: { defaultProps: { disableElevation: true } },
    MuiToggleButton: {
      styleOverrides: {
        root: ({ theme }) => ({
          minHeight: control,
          textTransform: "none",
          borderColor: vars(theme).surface.outline,
          "&.Mui-selected": { backgroundColor: vars(theme).secondary.container, color: vars(theme).secondary.onContainer },
        }),
      },
    },
    MuiTextField: { defaultProps: { size, fullWidth: true } },
    MuiOutlinedInput: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.small,
          backgroundColor: vars(theme).background.paper,
          transition: `background-color ${EASE}, box-shadow ${EASE}`,
          "& .MuiOutlinedInput-notchedOutline": { borderColor: vars(theme).surface.outline, transition: `border-color ${EASE}` },
          "&:hover:not(.Mui-disabled):not(.Mui-focused)": {
            backgroundColor: mix(vars(theme).primary.main, vars(theme).background.paper, 3),
            "& .MuiOutlinedInput-notchedOutline": { borderColor: vars(theme).surface.onVariant },
          },
          "&.Mui-focused": { boxShadow: ring(theme), "& .MuiOutlinedInput-notchedOutline": { borderWidth: 1, borderColor: vars(theme).primary.main } },
          "&.Mui-error.Mui-focused": { boxShadow: `0 0 0 3px ${mix(vars(theme).error.main, "transparent", 28)}` },
        }),
        input: { minHeight: control - 2, paddingBlock: 0, boxSizing: "border-box" },
      },
    },
    MuiSelect: {
      styleOverrides: {
        select: { minHeight: `${control - 2}px !important`, display: "flex", alignItems: "center", "&:focus": { backgroundColor: "transparent" } },
      },
    },
    MuiAutocomplete: {
      styleOverrides: {
        inputRoot: { paddingBlock: "0 !important", minHeight: control },
        paper: ({ theme }) => ({ borderRadius: theme.shape.borderRadius, boxShadow: theme.shadows[3], marginTop: theme.spacing(0.5) }),
        listbox: ({ theme }) => ({ padding: theme.spacing(0.75) }),
        option: ({ theme }) => ({
          borderRadius: theme.shape.small,
          minHeight: control,
          "&.Mui-focused": { backgroundColor: `${mix(vars(theme).primary.main, "transparent", 8)} !important` },
          "&[aria-selected=\"true\"]": { backgroundColor: `${vars(theme).primary.container} !important`, color: vars(theme).primary.onContainer },
        }),
      },
    },
    MuiNativeSelect: { styleOverrides: { select: { "&:focus": { backgroundColor: "transparent" } } } },
    MuiFormLabel: {
      styleOverrides: {
        root: ({ theme }) => ({ fontSize: "0.875rem", fontWeight: 600, color: vars(theme).surface.on, marginBottom: theme.spacing(0.75), "&.Mui-focused": { color: vars(theme).surface.on } }),
        asterisk: ({ theme }) => ({ color: vars(theme).error.main }),
      },
    },
    MuiFormHelperText: { styleOverrides: { root: ({ theme }) => ({ marginInline: 0, marginTop: theme.spacing(0.75) }) } },
    MuiCard: {
      defaultProps: { variant: "outlined" },
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.large,
          backgroundColor: vars(theme).background.paper,
          borderColor: mix(vars(theme).surface.outlineVariant, "transparent", 70),
          boxShadow: theme.shadows[1],
        }),
      },
    },
    MuiCardActionArea: {
      styleOverrides: {
        root: ({ theme }) => ({
          "&:hover": { backgroundImage: vars(theme).gradient.soft },
          "& .MuiCardActionArea-focusHighlight": { display: "none" },
        }),
      },
    },
    MuiPaper: {
      styleOverrides: {
        outlined: ({ theme }) => ({ borderColor: mix(vars(theme).surface.outlineVariant, "transparent", 70) }),
        rounded: ({ theme }) => ({ borderRadius: theme.shape.borderRadius }),
      },
    },
    MuiAlert: {
      defaultProps: { variant: "standard" },
      styleOverrides: {
        root: ({ theme, ownerState }) => {
          const tone = toneOf(ownerState.severity);
          return {
            borderRadius: theme.shape.borderRadius,
            backgroundColor: vars(theme)[tone].container,
            color: vars(theme)[tone].onContainer,
            boxShadow: `inset 0 0 0 1px ${mix(vars(theme)[tone].main, "transparent", 22)}`,
            "& .MuiAlert-icon": { color: vars(theme)[tone].main },
          };
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: ({ theme }) => ({ borderRadius: theme.shape.full, fontWeight: 600 }),
        outlined: ({ theme }) => ({ borderColor: vars(theme).surface.outlineVariant }),
      },
    },
    MuiDialog: {
      styleOverrides: {
        paper: ({ theme }) => ({
          borderRadius: theme.shape.extraLarge,
          backgroundColor: vars(theme).background.paper,
          backgroundImage: "none",
          border: 0,
          boxShadow: theme.shadows[4],
          padding: theme.spacing(1),
        }),
      },
    },
    MuiDialogTitle: { styleOverrides: { root: ({ theme }) => ({ ...theme.typography.h3 }) } },
    MuiDialogContentText: { styleOverrides: { root: ({ theme }) => ({ color: vars(theme).surface.onVariant }) } },
    MuiBackdrop: {
      styleOverrides: {
        root: ({ theme }) => ({
          "&:not(.MuiBackdrop-invisible)": { backgroundColor: mix(vars(theme).surface.scrim, "transparent", 40), backdropFilter: "blur(2px)" },
        }),
      },
    },
    MuiPopover: {
      styleOverrides: {
        paper: ({ theme }) => ({ borderRadius: theme.shape.large, boxShadow: theme.shadows[3], border: 0 }),
      },
    },
    MuiMenu: {
      styleOverrides: {
        paper: ({ theme }) => ({ borderRadius: theme.shape.borderRadius, boxShadow: theme.shadows[3], border: 0 }),
        list: ({ theme }) => ({ padding: theme.spacing(0.75) }),
      },
    },
    MuiMenuItem: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.small,
          minHeight: control,
          "&:hover": { backgroundColor: mix(vars(theme).primary.main, "transparent", 8) },
          "&.Mui-selected": { backgroundColor: vars(theme).primary.container, color: vars(theme).primary.onContainer },
        }),
      },
    },
    MuiListItemButton: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.small,
          "&.Mui-selected": { backgroundColor: vars(theme).primary.container, color: vars(theme).primary.onContainer },
        }),
      },
    },
    MuiTooltip: {
      defaultProps: { arrow: false },
      styleOverrides: {
        tooltip: ({ theme }) => ({
          borderRadius: theme.shape.borderRadius,
          backgroundColor: vars(theme).surface.inverse,
          color: vars(theme).surface.onInverse,
          fontSize: "0.75rem",
          fontWeight: 500,
          padding: "7px 12px",
          boxShadow: theme.shadows[2],
        }),
      },
    },
    MuiSnackbarContent: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.borderRadius,
          backgroundColor: vars(theme).surface.inverse,
          color: vars(theme).surface.onInverse,
          boxShadow: theme.shadows[4],
          ...theme.typography.body2,
        }),
      },
    },
    MuiTabs: {
      styleOverrides: {
        root: ({ theme }) => ({ borderBottom: `1px solid ${vars(theme).divider}`, minHeight: 44 }),
        indicator: ({ theme }) => ({ height: 3, borderTopLeftRadius: 3, borderTopRightRadius: 3, backgroundImage: vars(theme).gradient.primary }),
      },
    },
    MuiTab: {
      styleOverrides: {
        root: ({ theme }) => ({
          ...theme.typography.subtitle2,
          minHeight: compact ? 40 : 44,
          textTransform: "none",
          transition: `color ${EASE}, background-color ${EASE}`,
          "&:hover": { color: vars(theme).primary.main },
        }),
      },
    },
    MuiLinearProgress: {
      styleOverrides: {
        root: ({ theme }) => ({ borderRadius: theme.shape.full, backgroundColor: vars(theme).surface.containerHighest }),
        bar: ({ theme }) => ({ borderRadius: theme.shape.full, backgroundImage: vars(theme).gradient.primary }),
      },
    },
    MuiTableCell: {
      styleOverrides: {
        root: ({ theme }) => ({ borderBottomColor: vars(theme).divider }),
        head: ({ theme }) => ({ fontSize: "0.75rem", fontWeight: 600, letterSpacing: "0.04em", textTransform: "uppercase", color: vars(theme).surface.onVariant }),
      },
    },
    MuiTableRow: {
      styleOverrides: { root: ({ theme }) => ({ "&.MuiTableRow-hover:hover": { backgroundColor: mix(vars(theme).primary.main, "transparent", 5) } }) },
    },
    MuiAccordion: {
      styleOverrides: {
        root: ({ theme }) => ({
          borderRadius: theme.shape.large,
          border: `1px solid ${mix(vars(theme).surface.outlineVariant, "transparent", 70)}`,
          boxShadow: theme.shadows[1],
          "&::before": { display: "none" },
          "&:first-of-type, &:last-of-type": { borderRadius: theme.shape.large },
        }),
      },
    },
    MuiAppBar: {
      defaultProps: { elevation: 0, color: "inherit" },
      styleOverrides: { root: ({ theme }) => ({ backgroundColor: vars(theme).surface.container }) },
    },
    MuiLink: { defaultProps: { underline: "hover" } },
  };
}
