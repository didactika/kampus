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
 * Asking for a recovery link with the email of the account. The answer is the same whether the email has an account,
 * so the page never tells who has one.
 */

import Alert from "@mui/material/Alert";
import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import { useState, type FormEvent } from "react";

import { Button, Icon, TextField } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";
import { BackToSignIn, Outcome, StepHeading } from "./Outcome";

/** The form that asks for a recovery link. */
export function RecoveryRequest() {
  const { t } = useI18n();
  const [busy, setBusy] = useState(false);
  const [sent, setSent] = useState(false);
  const [problem, setProblem] = useState("");

  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const email = String(new FormData(event.currentTarget).get("email") ?? "").trim();
    setBusy(true);
    setProblem("");
    try {
      await browserApi("/identity/auth/recovery", { method: "POST", body: { email } });
      setSent(true);
    } catch (failure) {
      setProblem(failure instanceof ApiError && failure.status === 429 ? t("recovery.throttled") : t("error.generic"));
      setBusy(false);
    }
  }

  if (sent) {
    return (
      <Outcome title={t("recovery.title")} severity="info">
        {t("recovery.sent")}
      </Outcome>
    );
  }

  return (
    <>
      <StepHeading title={t("recovery.title")} lead={t("recovery.lead")} />
      <Stack component="form" spacing={2.5} onSubmit={(event) => void send(event)} aria-label={t("recovery.title")}>
        <TextField
          name="email"
          type="email"
          label={t("recovery.email")}
          autoComplete="email"
          required
          autoFocus
          startAdornment={
            <InputAdornment position="start">
              <Icon name="mail" size="small" />
            </InputAdornment>
          }
        />
        {problem ? (
          <Alert severity="error" role="alert">
            {problem}
          </Alert>
        ) : null}
        <Button type="submit" variant="filled" size="large" fullWidth busy={busy}>
          {t("recovery.submit")}
        </Button>
      </Stack>
      <BackToSignIn />
    </>
  );
}
