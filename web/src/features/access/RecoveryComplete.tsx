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
 * Completing a recovery from its link: a code of the second factor when the account has one (a code of the
 * authenticator app or a recovery code; never one by email, since the link came by email), then the new password.
 */

import Stack from "@mui/material/Stack";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";
import { useEffect, useState } from "react";

import { Loading, TextField } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";
import { NewPasswordForm } from "../session/NewPasswordForm";
import { useLinkToken } from "./link-token";
import { BackToSignIn, Outcome, StepHeading } from "./Outcome";

/** A way of proving the second factor while recovering. */
type Method = "totp" | "recovery_code";

/** Where the recovery stands. */
type Stage = { readonly kind: "reading" } | { readonly kind: "invalid" } | { readonly kind: "ready"; readonly methods: readonly Method[] } | { readonly kind: "done" };

/** The page that completes a recovery. */
export function RecoveryComplete() {
  const { t } = useI18n();
  const token = useLinkToken();
  const [stage, setStage] = useState<Stage>({ kind: "reading" });
  const [method, setMethod] = useState<Method>("totp");

  useEffect(() => {
    if (token === undefined) return;
    if (!token) {
      setStage({ kind: "invalid" });
      return;
    }
    browserApi<{ methods: string[] }>("/identity/auth/recovery/needs", { method: "POST", body: { token } })
      .then((needs) => {
        const methods: Method[] = needs.methods.length === 0 ? [] : [...(needs.methods.includes("totp") ? (["totp"] as const) : []), "recovery_code"];
        setMethod(methods[0] ?? "totp");
        setStage({ kind: "ready", methods });
      })
      .catch(() => setStage({ kind: "invalid" }));
  }, [token]);

  if (stage.kind === "reading") return <Loading label={t("loading")} />;
  if (stage.kind === "invalid") {
    return (
      <Outcome title={t("recovery.complete.title")} severity="error">
        {t("link.invalid")}
      </Outcome>
    );
  }
  if (stage.kind === "done") {
    return (
      <Outcome title={t("recovery.complete.title")} severity="success">
        {t("recovery.done")}
      </Outcome>
    );
  }

  const needsProof = stage.methods.length > 0;
  return (
    <>
      <StepHeading title={t("recovery.complete.title")} lead={t("recovery.complete.lead")} />
      <NewPasswordForm
        wide
        label={t("recovery.complete.title")}
        submit={t("password.store")}
        describe={(failure) => {
          if (failure instanceof ApiError && failure.code === "identity.token_invalid") return t("link.invalid");
          if (failure instanceof ApiError && failure.status < 500) return t("factor.failed");
          return undefined;
        }}
        onSubmit={async (password, form) => {
          const code = String(form.get("code") ?? "").trim();
          await browserApi("/identity/auth/recovery/complete", {
            method: "POST",
            body: { token, password, proof: needsProof ? { kind: method, code } : null },
          });
          setStage({ kind: "done" });
        }}
      >
        {needsProof ? (
          <Stack spacing={1.5}>
            <Typography variant="body2" color="text.secondary">
              {t("recovery.proof")}
            </Typography>
            {stage.methods.length > 1 ? (
              <ToggleButtonGroup exclusive fullWidth size="small" color="primary" value={method} aria-label={t("factor.method")} onChange={(_, value: Method | null) => value && setMethod(value)}>
                {stage.methods.map((m) => (
                  <ToggleButton key={m} value={m} sx={{ textTransform: "none", fontWeight: 600 }}>
                    {t(`factor.method.${m}`)}
                  </ToggleButton>
                ))}
              </ToggleButtonGroup>
            ) : null}
            <TextField
              key={method}
              name="code"
              label={method === "recovery_code" ? t("factor.method.recovery_code") : t("factor.code")}
              autoComplete="one-time-code"
              inputMode={method === "recovery_code" ? "text" : "numeric"}
              required
            />
          </Stack>
        ) : null}
      </NewPasswordForm>
      <BackToSignIn />
    </>
  );
}
