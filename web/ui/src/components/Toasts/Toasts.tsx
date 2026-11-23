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
 * The results of actions, as short messages stacked at the top right of the page, under its bar: each with the
 * icon and colour of what it tells, a close button, and a line that shows the time it has left. A message leaves
 * by itself (an error later than the rest), and waits while the pointer or the focus is on it. Messages are read
 * out without moving focus: an error at once, the rest politely. At most three show at a time; the next wait.
 */

import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import Paper from "@mui/material/Paper";
import Typography from "@mui/material/Typography";
import { keyframes } from "@mui/material/styles";
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";

import { Icon } from "../Icon/index";

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

/** How many messages show at a time. */
const SHOWN = 3;

/** The icon and the colour of each tone. */
const LOOK: Readonly<Record<ToastTone, { readonly icon: string; readonly colour: "info" | "success" | "warning" | "error" }>> = {
  info: { icon: "info", colour: "info" },
  success: { icon: "check_circle", colour: "success" },
  warning: { icon: "warning", colour: "warning" },
  danger: { icon: "error", colour: "error" },
};

const enter = keyframes`from { opacity: 0; transform: translateX(24px); } to { opacity: 1; transform: none; }`;
const drain = keyframes`from { transform: scaleX(1); } to { transform: scaleX(0); }`;

/** The props of `ToastProvider`. */
export interface ToastProviderProps {
  /** The name of the close button, in the language of the page. */
  readonly closeLabel: string;
  /** How long a message stays, in milliseconds; an error stays half as long again. */
  readonly lifetime?: number;
  /** The page. */
  readonly children: ReactNode;
}

let next = 0;

/** One message, which leaves when its time is over unless the pointer or the focus is on it. */
function ToastCard({ toast, lifetime, closeLabel, onClose }: { readonly toast: Toast; readonly lifetime: number; readonly closeLabel: string; readonly onClose: () => void }) {
  const [paused, setPaused] = useState(false);
  const left = useRef(lifetime);
  const started = useRef(Date.now());
  const { icon, colour } = LOOK[toast.tone];

  useEffect(() => {
    if (paused) return undefined;
    started.current = Date.now();
    const timer = setTimeout(onClose, left.current);
    return () => {
      clearTimeout(timer);
      left.current -= Date.now() - started.current;
    };
  }, [paused, onClose]);

  return (
    <Paper
      role={toast.tone === "danger" ? "alert" : "status"}
      aria-live={toast.tone === "danger" ? "assertive" : "polite"}
      elevation={3}
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      onFocus={() => setPaused(true)}
      onBlur={() => setPaused(false)}
      sx={{
        position: "relative",
        overflow: "hidden",
        display: "flex",
        alignItems: "flex-start",
        gap: 1.5,
        pl: 2,
        pr: 1,
        py: 1.5,
        inlineSize: 360,
        maxInlineSize: "calc(100vw - 32px)",
        borderRadius: 3,
        bgcolor: "surface.inverse",
        color: "surface.onInverse",
        pointerEvents: "auto",
        animation: `${enter} 220ms cubic-bezier(0.2, 0, 0, 1)`,
        "@media (prefers-reduced-motion: reduce)": { animation: "none" },
      }}
    >
      <Box sx={{ display: "flex", color: `${colour}.main`, filter: "brightness(1.35)", mt: 0.25 }}>
        <Icon name={icon} size="small" />
      </Box>
      <Typography variant="body2" sx={{ flex: 1, minInlineSize: 0, py: 0.25, fontWeight: 500 }}>
        {toast.message}
      </Typography>
      <IconButton aria-label={closeLabel} size="small" onClick={onClose} sx={{ color: "inherit", mt: -0.5 }}>
        <Icon name="close" size="small" />
      </IconButton>
      <Box
        aria-hidden
        sx={{
          position: "absolute",
          insetInline: 0,
          insetBlockEnd: 0,
          blockSize: 3,
          bgcolor: `${colour}.main`,
          transformOrigin: "left",
          animation: `${drain} ${lifetime}ms linear forwards`,
          animationPlayState: paused ? "paused" : "running",
          "@media (prefers-reduced-motion: reduce)": { animation: "none", opacity: 0.6 },
        }}
      />
    </Paper>
  );
}

/** Holds the messages of a page and shows them stacked at its top right. */
export function ToastProvider({ closeLabel, lifetime = 5000, children }: ToastProviderProps) {
  const [queue, setQueue] = useState<Toast[]>([]);
  const show = useCallback<ShowToast>((message, tone = "info") => {
    next += 1;
    setQueue((current) => [...current, { id: next, message, tone }]);
  }, []);
  const close = useCallback((id: number) => setQueue((current) => current.filter((t) => t.id !== id)), []);
  const value = useMemo(() => show, [show]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <Box
        sx={{
          position: "fixed",
          insetBlockStart: { xs: 72, md: 80 },
          insetInlineEnd: 16,
          zIndex: "snackbar",
          display: "flex",
          flexDirection: "column",
          alignItems: "flex-end",
          gap: 1,
          pointerEvents: "none",
        }}
      >
        {queue.slice(0, SHOWN).map((toast) => (
          <ToastCard key={toast.id} toast={toast} closeLabel={closeLabel} lifetime={toast.tone === "danger" ? lifetime * 1.5 : lifetime} onClose={() => close(toast.id)} />
        ))}
      </Box>
    </ToastContext.Provider>
  );
}

/** The function that shows a message, inside a `ToastProvider`. */
export function useToast(): ShowToast {
  const show = useContext(ToastContext);
  if (!show) throw new Error("useToast needs a ToastProvider around it");
  return show;
}
