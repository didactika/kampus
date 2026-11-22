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
// @package   web
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


"use client";

/**
 * Raising the session for a sensitive change. When the API answers that the sign-in is too old, a dialog asks for
 * the password once, raises the session and runs the change again; closing it gives the change up.
 */

import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Typography from "@mui/material/Typography";
import { createContext, useCallback, useContext, useRef, useState, type FormEvent, type ReactNode } from "react";

import { Button } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";
import { PasswordField } from "../session/PasswordField";

/** Thrown when the person closes the dialog instead of confirming. */
export class StepUpCancelled extends Error {
  constructor() {
    super("step-up cancelled");
    this.name = "StepUpCancelled";
  }
}

/** Runs a sensitive change, raising the session first when the API asks for it. */
type Guard = <T>(change: () => Promise<T>) => Promise<T>;

const StepUpContext = createContext<Guard | null>(null);

/** Whether a failure is the API asking for a recent sign-in. */
function needsStepUp(failure: unknown): boolean {
  return failure instanceof ApiError && failure.code === "identity.step_up_required";
}

/** Gives the pages under it `useStepUp`. */
export function StepUpProvider({ children }: { readonly children: ReactNode }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [wrong, setWrong] = useState("");
  const waiting = useRef<{ resolve: () => void; reject: (reason: unknown) => void } | null>(null);

  const guard = useCallback<Guard>(async (change) => {
    try {
      return await change();
    } catch (failure) {
      if (!needsStepUp(failure)) throw failure;
      await new Promise<void>((resolve, reject) => {
        waiting.current = { resolve, reject };
        setWrong("");
        setOpen(true);
      });
      return change();
    }
  }, []);

  function cancel() {
    setOpen(false);
    waiting.current?.reject(new StepUpCancelled());
    waiting.current = null;
  }

  async function confirm(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const password = String(new FormData(event.currentTarget).get("password") ?? "");
    setBusy(true);
    try {
      await browserApi("/identity/auth/step-up", { method: "POST", body: { password } });
      setOpen(false);
      waiting.current?.resolve();
      waiting.current = null;
    } catch (failure) {
      setWrong(failure instanceof ApiError && failure.status < 500 ? t("stepup.failed") : t("error.generic"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <StepUpContext.Provider value={guard}>
      {children}
      <Dialog open={open} onClose={cancel} aria-labelledby="stepup-title" fullWidth maxWidth="xs">
        <form onSubmit={(event) => void confirm(event)}>
          <DialogTitle id="stepup-title">{t("stepup.title")}</DialogTitle>
          <DialogContent>
            <Typography color="text.secondary" sx={{ mb: 2.5 }}>
              {t("stepup.lead")}
            </Typography>
            <PasswordField name="password" label={t("signin.password")} autoComplete="current-password" required autoFocus error={wrong || undefined} />
          </DialogContent>
          <DialogActions>
            <Button variant="text" onClick={cancel}>
              {t("action.cancel")}
            </Button>
            <Button type="submit" variant="filled" busy={busy}>
              {t("stepup.submit")}
            </Button>
          </DialogActions>
        </form>
      </Dialog>
    </StepUpContext.Provider>
  );
}

/** `const guard = useStepUp(); await guard(() => browserApi(...))` */
export function useStepUp(): Guard {
  const guard = useContext(StepUpContext);
  if (!guard) throw new Error("useStepUp needs a StepUpProvider");
  return guard;
}
