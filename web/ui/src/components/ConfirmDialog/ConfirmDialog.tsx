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

"use client";

/**
 * The question asked before an action that removes or stops something. Its title asks, its body says what stays and
 * what is lost, and its confirm button takes the tone of the action. Escape and the cancel button close it without
 * acting.
 */

import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import { useId, type ReactNode } from "react";

import { Button, type ButtonTone } from "../Button/index";

/** The props of `ConfirmDialog`. */
export interface ConfirmDialogProps {
  /** Whether it is open. */
  readonly open: boolean;
  /** The question, such as "Delete the course?". */
  readonly title: string;
  /** What stays and what is lost. */
  readonly children: ReactNode;
  /** The text of the button that acts, such as "Delete". */
  readonly confirmLabel: string;
  /** The text of the button that does not. */
  readonly cancelLabel: string;
  /** What the action does; `danger` unless said otherwise. */
  readonly tone?: ButtonTone;
  /** Whether the action is running. */
  readonly busy?: boolean;
  /** Acts. */
  readonly onConfirm: () => void;
  /** Closes it without acting. */
  readonly onClose: () => void;
}

/** `<ConfirmDialog open title="Delete the course?" …>Its activities and grades are deleted too.</ConfirmDialog>` */
export function ConfirmDialog({ open, title, children, confirmLabel, cancelLabel, tone = "danger", busy, onConfirm, onClose }: ConfirmDialogProps) {
  const heading = useId();
  return (
    <Dialog open={open} onClose={busy ? undefined : onClose} aria-labelledby={heading} maxWidth="xs" fullWidth>
      <DialogTitle id={heading}>{title}</DialogTitle>
      <DialogContent>{children}</DialogContent>
      <DialogActions>
        <Button variant="text" onClick={onClose} disabled={busy}>
          {cancelLabel}
        </Button>
        <Button variant="filled" tone={tone} onClick={onConfirm} busy={busy}>
          {confirmLabel}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
