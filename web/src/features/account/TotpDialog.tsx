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
 * Adding an authenticator app: the server gives a new secret, shown as a QR code and as a key to type; the app is
 * added once the person enters a code it shows, which proves it holds the secret.
 */

import Box from "@mui/material/Box";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import QRCode from "qrcode";
import { useEffect, useState, type FormEvent } from "react";

import { Button, Loading, TextField } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";

/** A new secret for an authenticator app. */
interface Setup {
  readonly secret: string;
  readonly uri: string;
}

/** What adding a second factor answers. */
export interface Added {
  readonly id: string;
  readonly recovery_codes: readonly string[] | null;
}

/** The props of `TotpDialog`. */
interface TotpDialogProps {
  readonly open: boolean;
  readonly onClose: () => void;
  /** Asks the server for a new secret, raising the session if needed. */
  readonly start: () => Promise<Setup>;
  /** Called with what the server answered once the app is added. */
  readonly onAdded: (added: Added, label: string) => void;
}

/** The key in groups of four, easier to read and type. */
function grouped(secret: string): string {
  return secret.replace(/(.{4})/g, "$1 ").trim();
}

/** `<TotpDialog open start={…} onAdded={…} onClose={…} />`; a new secret is asked each time it opens. */
export function TotpDialog({ open, onClose, start, onAdded }: TotpDialogProps) {
  const { t } = useI18n();
  const [setup, setSetup] = useState<Setup | null>(null);
  const [image, setImage] = useState("");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState("");

  useEffect(() => {
    if (!open) {
      setSetup(null);
      setImage("");
      setProblem("");
      return;
    }
    let live = true;
    start()
      .then(async (next) => {
        const data = await QRCode.toDataURL(next.uri, { errorCorrectionLevel: "M", margin: 2, width: 200, color: { dark: "#000000", light: "#ffffff" } });
        if (live) {
          setSetup(next);
          setImage(data);
        }
      })
      .catch(() => {
        if (live) onClose();
      });
    return () => {
      live = false;
    };
  }, [open]);

  async function confirm(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!setup) return;
    const form = new FormData(event.currentTarget);
    const label = String(form.get("label") ?? "").trim() || t("factor.method.totp");
    const code = String(form.get("code") ?? "").replace(/\s/g, "");
    setBusy(true);
    setProblem("");
    try {
      const added = await browserApi<Added>("/identity/me/second-factors/totp/confirm", { method: "POST", body: { label, secret: setup.secret, code } });
      onAdded(added, label);
    } catch (failure) {
      setProblem(failure instanceof ApiError && failure.status < 500 ? t("totp.failed") : t("error.generic"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onClose={onClose} aria-labelledby="totp-title" fullWidth maxWidth="xs">
      <form onSubmit={(event) => void confirm(event)}>
        <DialogTitle id="totp-title">{t("factors.add.totp")}</DialogTitle>
        <DialogContent>
          {setup && image ? (
            <Stack spacing={2.5}>
              <Typography color="text.secondary">{t("totp.scan")}</Typography>
              <Box sx={{ alignSelf: "center", p: 1, borderRadius: 3, bgcolor: "#ffffff", lineHeight: 0, boxShadow: 1 }}>
                <img src={image} width={200} height={200} alt={t("totp.qr")} />
              </Box>
              <Box>
                <Typography variant="caption" color="text.secondary" component="p">
                  {t("totp.secret")}
                </Typography>
                <Typography sx={{ fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace", wordBreak: "break-all", userSelect: "all" }}>{grouped(setup.secret)}</Typography>
              </Box>
              <TextField name="label" label={t("totp.label")} defaultValue={t("factor.method.totp")} inputProps={{ maxLength: 64 }} />
              <TextField name="code" label={t("totp.code")} autoComplete="one-time-code" inputMode="numeric" required error={problem || undefined} />
            </Stack>
          ) : (
            <Loading label={t("loading")} />
          )}
        </DialogContent>
        <DialogActions>
          <Button variant="text" onClick={onClose}>
            {t("action.cancel")}
          </Button>
          <Button type="submit" variant="filled" busy={busy} disabled={!setup}>
            {t("totp.confirm")}
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  );
}
