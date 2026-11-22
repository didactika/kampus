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
 * A button with the meaning Didactika gives it: `filled` is the main action of an area (one per area), `tonal` and
 * `elevated` important ones beside it, `outlined` an ordinary action and `text` the least. Its tone says what it
 * does: `danger` removes or stops and always asks first, `success` starts a process. While `busy` it cannot be
 * pressed again.
 */

import MuiButton, { type ButtonProps as MuiButtonProps } from "@mui/material/Button";

import { Icon } from "../Icon/index.js";

/** How much a button stands out, from the most to the least. */
export type ButtonVariant = "filled" | "tonal" | "elevated" | "outlined" | "text";

/** What a button does. */
export type ButtonTone = "primary" | "danger" | "success";

/** The props of `Button`. */
export interface ButtonProps extends Omit<MuiButtonProps, "variant" | "color" | "startIcon" | "loading"> {
  /** How much it stands out. */
  readonly variant?: ButtonVariant;
  /** What it does. */
  readonly tone?: ButtonTone;
  /** The name of an icon before its text. */
  readonly icon?: string;
  /** Whether the action it started is still running. */
  readonly busy?: boolean;
}

/** MUI's variant of each variant of Didactika. */
const VARIANTS = { filled: "contained", elevated: "contained", tonal: "tonal", outlined: "outlined", text: "text" } as const;

/** `<Button variant="filled">Save</Button>` */
export function Button({ variant = "outlined", tone = "primary", icon, busy = false, sx, ...rest }: ButtonProps) {
  const color = tone === "danger" ? "error" : tone;
  const elevated =
    variant === "elevated"
      ? {
          backgroundColor: "var(--k-palette-surface-containerLow)",
          color: `var(--k-palette-${color}-main)`,
          boxShadow: 1,
          "&:hover": { boxShadow: 2, backgroundColor: "var(--k-palette-surface-containerLow)" },
        }
      : {};
  return (
    <MuiButton
      variant={VARIANTS[variant]}
      color={color}
      loading={busy || undefined}
      startIcon={icon ? <Icon name={icon} size="small" /> : undefined}
      aria-busy={busy || undefined}
      sx={[elevated, ...(Array.isArray(sx) ? sx : [sx])]}
      {...rest}
    />
  );
}
