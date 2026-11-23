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
 * The bar of a page being changed: it floats at the foot of the page, over the content, while there is something
 * to keep or to give up, so the page itself keeps its layout. It says what is pending and offers to discard it or
 * save it; other actions, such as going back to the default, sit before them. Its message is read out when it
 * changes, and room is kept under the content so the bar never covers its end.
 */

import Box from "@mui/material/Box";
import Paper from "@mui/material/Paper";
import Slide from "@mui/material/Slide";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import type { ReactNode } from "react";

import { Button } from "../../components/Button";
import { Icon } from "../../components/Icon";

/** The props of `ChangesBar`. */
export interface ChangesBarProps {
  /** Whether it shows. */
  readonly open: boolean;
  /** What is pending, such as "3 unsaved changes". */
  readonly message: string;
  /** The name of the bar, for assistive technology. */
  readonly label: string;
  /** The text of the button that saves. */
  readonly saveLabel: string;
  /** The text of the button that gives the changes up. */
  readonly discardLabel: string;
  /** Saves. */
  readonly onSave: () => void;
  /** Gives the changes up. */
  readonly onDiscard: () => void;
  /** Whether saving is running. */
  readonly busy?: boolean;
  /** Whether there is anything to save; without changes only the discard button, as "Done", is offered. */
  readonly dirty?: boolean;
  /** Other actions, before discard and save. */
  readonly actions?: ReactNode;
}

/** `<ChangesBar open message="3 unsaved changes" … />` */
export function ChangesBar({ open, message, label, saveLabel, discardLabel, onSave, onDiscard, busy = false, dirty = true, actions }: ChangesBarProps) {
  return (
    <>
      {open ? <Box aria-hidden sx={{ blockSize: 96 }} /> : null}
      <Box sx={{ position: "fixed", insetBlockEnd: 24, insetInline: 16, zIndex: "appBar", display: "flex", justifyContent: "center", pointerEvents: "none" }}>
      <Slide direction="up" in={open} mountOnEnter unmountOnExit>
        <Paper
          role="region"
          aria-label={label}
          elevation={6}
          sx={{
            pointerEvents: "auto",
            maxInlineSize: "100%",
            display: "flex",
            alignItems: "center",
            gap: 2,
            pl: 2.5,
            pr: 1,
            py: 1,
            borderRadius: 999,
            bgcolor: "surface.inverse",
            color: "surface.onInverse",
          }}
        >
          <Icon name={dirty ? "edit_note" : "edit_square"} size="small" />
          <Typography variant="body2" aria-live="polite" sx={{ fontWeight: 600, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
            {message}
          </Typography>
          <Stack direction="row" spacing={1} sx={{ alignItems: "center", flex: "none" }}>
            {actions}
            <Button variant="text" onClick={onDiscard} disabled={busy} sx={{ color: "inherit" }}>
              {discardLabel}
            </Button>
            {dirty ? (
              <Button variant="filled" icon="save" busy={busy} onClick={onSave} sx={{ borderRadius: 999 }}>
                {saveLabel}
              </Button>
            ) : null}
          </Stack>
        </Paper>
      </Slide>
      </Box>
    </>
  );
}
