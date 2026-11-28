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
 * The second factors of the account: each with its kind, name and use, and a way to remove it; adding an
 * authenticator app or codes by email; and a new set of recovery codes. Every change raises the session when the
 * sign-in is old, and the recovery codes of a first factor are shown once.
 */

import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import { Button, ConfirmDialog, Row, RowList, SectionCard, TextField, useToast } from "@kampus/ui";
import { useState, type FormEvent } from "react";

import { browserApi } from "../../api/browser";
import type { SecondFactor } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { RecoveryCodesDialog } from "./RecoveryCodesDialog";
import { StepUpCancelled, useStepUp } from "./StepUp";
import { TotpDialog, type Added } from "./TotpDialog";

/** The icon of each kind of factor. */
const ICONS: Readonly<Record<string, string>> = { totp: "mobile_lock_portrait", email: "mail", passkey: "passkey" };

/** `<SecondFactors factors={factors} />` */
export function SecondFactors({ factors }: { readonly factors: readonly SecondFactor[] }) {
  const { t, date, relative } = useI18n();
  const toast = useToast();
  const guard = useStepUp();
  const [items, setItems] = useState(factors);
  const [codes, setCodes] = useState<readonly string[] | null>(null);
  const [adding, setAdding] = useState<"totp" | "email" | null>(null);
  const [removing, setRemoving] = useState<SecondFactor | null>(null);
  const [renewing, setRenewing] = useState(false);
  const [busy, setBusy] = useState(false);

  function failed(failure: unknown) {
    if (!(failure instanceof StepUpCancelled)) toast(t("error.generic"), "danger");
  }

  function added(result: Added, kind: SecondFactor["kind"], label: string) {
    setItems((list) => [...list, { id: result.id, kind, label, created_at: new Date().toISOString(), last_used_at: null }]);
    setAdding(null);
    toast(t("factors.added"), "success");
    if (result.recovery_codes) setCodes(result.recovery_codes);
  }

  async function addEmail(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const label = String(new FormData(event.currentTarget).get("label") ?? "").trim() || t("factor.method.email");
    setBusy(true);
    try {
      const result = await guard(() => browserApi<Added>("/identity/me/second-factors/email", { method: "POST", body: { label } }));
      added(result, "email", label);
    } catch (failure) {
      failed(failure);
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    if (!removing) return;
    setBusy(true);
    try {
      await guard(() => browserApi(`/identity/me/second-factors/${removing.id}`, { method: "DELETE" }));
      setItems((list) => list.filter((f) => f.id !== removing.id));
      toast(t("factors.removed"), "success");
      setRemoving(null);
    } catch (failure) {
      failed(failure);
    } finally {
      setBusy(false);
    }
  }

  async function renew() {
    setBusy(true);
    try {
      const result = await guard(() => browserApi<{ recovery_codes: string[] }>("/identity/me/recovery-codes", { method: "POST" }));
      setRenewing(false);
      setCodes(result.recovery_codes);
    } catch (failure) {
      failed(failure);
    } finally {
      setBusy(false);
    }
  }

  return (
    <SectionCard icon="shield" title={t("factors.title")} lead={t("factors.lead")}>
        {items.length === 0 ? (
          <EmptyState compact icon="shield" message={t("factors.empty")} />
        ) : (
          <RowList label={t("factors.title")}>
            {items.map((factor) => (
              <Row
                key={factor.id}
                icon={ICONS[factor.kind] ?? "shield"}
                title={factor.label}
                meta={[t(`factor.method.${factor.kind}`), t("factors.since", { date: date(factor.created_at) }), factor.last_used_at ? t("factors.used", { date: relative(factor.last_used_at) }) : t("factors.unused")].join(" · ")}
                actions={
                  <Button variant="text" tone="danger" icon="delete" size="small" onClick={() => setRemoving(factor)}>
                    {t("factors.remove")}
                  </Button>
                }
              />
            ))}
          </RowList>
        )}
        <Stack direction="row" sx={{ mt: 2.5, flexWrap: "wrap", gap: 1 }}>
          <Button variant="tonal" icon="mobile_lock_portrait" onClick={() => setAdding("totp")}>
            {t("factors.add.totp")}
          </Button>
          <Button variant="outlined" icon="mail" onClick={() => setAdding("email")}>
            {t("factors.add.email")}
          </Button>
          {items.length > 0 ? (
            <Button variant="text" icon="key" onClick={() => setRenewing(true)}>
              {t("factors.codes")}
            </Button>
          ) : null}
        </Stack>

      <TotpDialog
        open={adding === "totp"}
        onClose={() => setAdding(null)}
        start={() => guard(() => browserApi<{ secret: string; uri: string }>("/identity/me/second-factors/totp", { method: "POST" }))}
        onAdded={(result, label) => added(result, "totp", label)}
      />

      <Dialog open={adding === "email"} onClose={() => setAdding(null)} aria-labelledby="email-factor-title" fullWidth maxWidth="xs">
        <form onSubmit={(event) => void addEmail(event)}>
          <DialogTitle id="email-factor-title">{t("factors.add.email")}</DialogTitle>
          <DialogContent>
            <Typography color="text.secondary" sx={{ mb: 2.5 }}>
              {t("factors.email.lead")}
            </Typography>
            <TextField name="label" label={t("totp.label")} defaultValue={t("factor.method.email")} inputProps={{ maxLength: 64 }} />
          </DialogContent>
          <DialogActions>
            <Button variant="text" onClick={() => setAdding(null)}>
              {t("action.cancel")}
            </Button>
            <Button type="submit" variant="filled" busy={busy}>
              {t("totp.confirm")}
            </Button>
          </DialogActions>
        </form>
      </Dialog>

      <ConfirmDialog
        open={removing !== null}
        title={t("factors.remove.title")}
        confirmLabel={t("factors.remove")}
        cancelLabel={t("action.cancel")}
        busy={busy}
        onConfirm={() => void remove()}
        onClose={() => setRemoving(null)}
      >
        {t("factors.remove.body", { label: removing?.label ?? "" })}
      </ConfirmDialog>

      <ConfirmDialog
        open={renewing}
        title={t("factors.codes.title")}
        confirmLabel={t("factors.codes.confirm")}
        cancelLabel={t("action.cancel")}
        tone="primary"
        busy={busy}
        onConfirm={() => void renew()}
        onClose={() => setRenewing(false)}
      >
        {t("factors.codes.body")}
      </ConfirmDialog>

      <RecoveryCodesDialog codes={codes} onClose={() => setCodes(null)} />
    </SectionCard>
  );
}
