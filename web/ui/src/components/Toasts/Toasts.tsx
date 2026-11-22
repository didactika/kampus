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
 * The results of asynchronous actions, as short messages at the foot of the page, one at a time and in order. Each is
 * read out without moving focus (an error at once, the rest politely) and leaves after a while, or with its close
 * button.
 */

import IconButton from "@mui/material/IconButton";
import Snackbar from "@mui/material/Snackbar";
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";

import { Icon } from "../Icon/index.js";

/** What a message tells. */
export type ToastTone = "info" | "success" | "warning" | "danger";

/** One message. */
interface Toast {
  readonly id: number;
  readonly message: string;
  readonly tone: ToastTone;
}

/** Shows a message. */
export type ShowToast = (message: string, tone?: ToastTone) => void;

const ToastContext = createContext<ShowToast | null>(null);

/** The props of `ToastProvider`. */
export interface ToastProviderProps {
  /** The name of the close button, in the language of the page. */
  readonly closeLabel: string;
  /** How long a message stays, in milliseconds. */
  readonly lifetime?: number;
  /** The page. */
  readonly children: ReactNode;
}

let next = 0;

/** Holds the messages of a page and shows them one after another. */
export function ToastProvider({ closeLabel, lifetime = 6000, children }: ToastProviderProps) {
  const [queue, setQueue] = useState<Toast[]>([]);
  const show = useCallback<ShowToast>((message, tone = "info") => {
    next += 1;
    setQueue((current) => [...current, { id: next, message, tone }]);
  }, []);
  const close = useCallback(() => setQueue((current) => current.slice(1)), []);
  const current = queue[0];
  const value = useMemo(() => show, [show]);
  return (
    <ToastContext.Provider value={value}>
      {children}
      <Snackbar
        key={current?.id}
        open={Boolean(current)}
        message={current?.message}
        autoHideDuration={lifetime}
        onClose={(_, reason) => reason !== "clickaway" && close()}
        anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
        slotProps={{
          content: {
            role: current?.tone === "danger" ? "alert" : "status",
            "aria-live": current?.tone === "danger" ? "assertive" : "polite",
          },
        }}
        action={
          <IconButton aria-label={closeLabel} color="inherit" size="small" onClick={close}>
            <Icon name="close" size="small" />
          </IconButton>
        }
      />
    </ToastContext.Provider>
  );
}

/** The function that shows a message, inside a `ToastProvider`. */
export function useToast(): ShowToast {
  const show = useContext(ToastContext);
  if (!show) throw new Error("useToast needs a ToastProvider around it");
  return show;
}
