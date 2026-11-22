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
 * The sign-in: the name and the password first, then, when the account has one, a second factor (a code of the
 * authenticator app, a code sent by email or a recovery code). One clear message when something is not right.
 */

import Alert from "@mui/material/Alert";
import InputAdornment from "@mui/material/InputAdornment";
import Link from "@mui/material/Link";
import Stack from "@mui/material/Stack";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";
import NextLink from "next/link";
import { useRouter } from "next/navigation";
import { useState, type FormEvent } from "react";

import { Button, Icon, TextField } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";
import { PasswordField } from "./PasswordField";

/** What a sign-in answers. */
type Answer =
  | { readonly status: "signed_in" }
  | { readonly status: "second_factor_required"; readonly challenge: string; readonly methods: readonly string[] }
  | { readonly status: "confirm_link" };

/** A way of proving the second factor this form offers. */
type Method = "totp" | "email" | "recovery_code";

/** A pending sign-in that waits for its second factor. */
interface Pending {
  readonly challenge: string;
  readonly methods: readonly Method[];
}

/** The ways of proving a second factor the account has, plus its recovery codes. */
function methodsOf(kinds: readonly string[]): Method[] {
  const offered = kinds.filter((kind): kind is Method => kind === "totp" || kind === "email");
  return offered.length > 0 ? [...offered, "recovery_code"] : [];
}

/** The form that signs a person in; `notice` tells why they are asked to sign in again. */
export function SignInForm({ notice }: { readonly notice?: string }) {
  const { t } = useI18n();
  const router = useRouter();
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState("");
  const [pending, setPending] = useState<Pending | null>(null);

  function failed(failure: unknown) {
    if (failure instanceof ApiError && failure.code === "identity.sign_in_throttled") setProblem(t("signin.throttled"));
    else setProblem(failure instanceof ApiError && failure.status < 500 ? t("signin.failed") : t("error.generic"));
  }

  function enter() {
    router.replace("/");
    router.refresh();
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setBusy(true);
    setProblem("");
    try {
      const answer = await browserApi<Answer>("/identity/auth/sign-in", { method: "POST", body: { name: form.get("name"), password: form.get("password") } });
      if (answer.status === "second_factor_required") {
        setPending({ challenge: answer.challenge, methods: methodsOf(answer.methods) });
        setBusy(false);
      } else if (answer.status === "signed_in") {
        enter();
      } else {
        setProblem(t("error.generic"));
        setBusy(false);
      }
    } catch (failure) {
      failed(failure);
      setBusy(false);
    }
  }

  if (pending) return <SecondFactor pending={pending} onSignedIn={enter} onBack={() => setPending(null)} />;

  return (
    <>
      <Typography variant="h1" sx={{ fontSize: { xs: "1.75rem", md: "2rem" } }}>
        {t("signin.welcome")}
      </Typography>
      <Typography color="text.secondary" sx={{ mt: 1, mb: 4 }}>
        {t("signin.lead")}
      </Typography>
      <Stack spacing={2.5} component="form" onSubmit={(event) => void submit(event)} aria-label={t("signin.title")}>
        {notice ? (
          <Alert severity="success" role="status">
            {notice}
          </Alert>
        ) : null}
        <TextField
          name="name"
          label={t("signin.name")}
          autoComplete="username"
          required
          autoFocus
          startAdornment={
            <InputAdornment position="start">
              <Icon name="person" size="small" />
            </InputAdornment>
          }
        />
        <PasswordField name="password" label={t("signin.password")} autoComplete="current-password" required withIcon />
        <Link component={NextLink} href="/recovery" variant="body2" sx={{ alignSelf: "flex-end", fontWeight: 600 }}>
          {t("signin.forgot")}
        </Link>
        {problem ? (
          <Alert severity="error" role="alert">
            {problem}
          </Alert>
        ) : null}
        <Button type="submit" variant="filled" size="large" busy={busy} fullWidth>
          {t("signin.submit")}
        </Button>
      </Stack>
      <Stack direction="row" spacing={1} sx={{ alignItems: "center", justifyContent: "center", mt: 4, color: "text.secondary" }}>
        <Icon name="shield_person" size="small" />
        <Typography variant="caption">{t("signin.secure")}</Typography>
      </Stack>
    </>
  );
}

/** The second step of a sign-in: the code of a second factor. */
function SecondFactor({ pending, onSignedIn, onBack }: { readonly pending: Pending; readonly onSignedIn: () => void; readonly onBack: () => void }) {
  const { t } = useI18n();
  const [method, setMethod] = useState<Method | undefined>(pending.methods[0]);
  const [busy, setBusy] = useState(false);
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const [problem, setProblem] = useState("");

  async function sendCode() {
    setSending(true);
    setProblem("");
    try {
      await browserApi("/identity/auth/second-factor/email-code", { method: "POST", body: { challenge: pending.challenge } });
      setSent(true);
    } catch {
      setProblem(t("error.generic"));
    } finally {
      setSending(false);
    }
  }

  async function verify(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!method) return;
    const code = String(new FormData(event.currentTarget).get("code") ?? "").trim();
    setBusy(true);
    setProblem("");
    try {
      await browserApi("/identity/auth/second-factor", { method: "POST", body: { challenge: pending.challenge, proof: { kind: method, code } } });
      onSignedIn();
    } catch (failure) {
      setProblem(failure instanceof ApiError && failure.status < 500 ? t("factor.failed") : t("error.generic"));
      setBusy(false);
    }
  }

  return (
    <>
      <Typography variant="h1" sx={{ fontSize: { xs: "1.75rem", md: "2rem" } }}>
        {t("factor.title")}
      </Typography>
      <Typography color="text.secondary" sx={{ mt: 1, mb: 4 }}>
        {method ? t("factor.lead") : t("factor.none")}
      </Typography>
      {method ? (
        <Stack spacing={2.5} component="form" onSubmit={(event) => void verify(event)} aria-label={t("factor.title")}>
          {pending.methods.length > 1 ? (
            <ToggleButtonGroup
              exclusive
              fullWidth
              size="small"
              color="primary"
              value={method}
              aria-label={t("factor.method")}
              onChange={(_, value: Method | null) => {
                if (value) setMethod(value);
                setProblem("");
              }}
            >
              {pending.methods.map((m) => (
                <ToggleButton key={m} value={m} sx={{ textTransform: "none", fontWeight: 600 }}>
                  {t(`factor.method.${m}`)}
                </ToggleButton>
              ))}
            </ToggleButtonGroup>
          ) : null}
          {method === "email" ? (
            <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
              <Button variant="tonal" icon="mail" busy={sending} onClick={() => void sendCode()}>
                {sent ? t("factor.resend") : t("factor.send")}
              </Button>
              {sent ? (
                <Typography variant="body2" color="text.secondary" role="status">
                  {t("factor.sent")}
                </Typography>
              ) : null}
            </Stack>
          ) : null}
          <TextField
            key={method}
            name="code"
            label={method === "recovery_code" ? t("factor.method.recovery_code") : t("factor.code")}
            autoComplete="one-time-code"
            inputMode={method === "recovery_code" ? "text" : "numeric"}
            required
            autoFocus
          />
          {problem ? (
            <Alert severity="error" role="alert">
              {problem}
            </Alert>
          ) : null}
          <Button type="submit" variant="filled" size="large" busy={busy} fullWidth>
            {t("factor.submit")}
          </Button>
        </Stack>
      ) : null}
      <Button variant="text" icon="arrow_back" onClick={onBack} sx={{ mt: 2 }}>
        {t("auth.back")}
      </Button>
    </>
  );
}
