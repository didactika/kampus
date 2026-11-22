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
 * A form that chooses a new password: the password and its copy, checked as they are typed (no blank characters,
 * both equal) and against the rules of the campus when sent. Other fields, such as the current password or a code,
 * come before them.
 */

import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import { useState, type FormEvent, type ReactNode } from "react";

import { Button } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";
import { PasswordField } from "./PasswordField";
import { hasBlank, rejectedPassword } from "./password-rules";

/** The props of `NewPasswordForm`. */
export interface NewPasswordFormProps {
  /** The accessible name of the form. */
  readonly label: string;
  /** The label of the button that sends it. */
  readonly submit: string;
  /** Sends the password; a failure it throws is explained under the password or above the button. */
  readonly onSubmit: (password: string, form: FormData) => Promise<void>;
  /** The message of a failure that is not about the new password; generic otherwise. */
  readonly describe?: (failure: unknown) => string | undefined;
  /** Fields before the new password. */
  readonly children?: ReactNode;
  /** Whether the button fills the width, as on the pages before signing in. */
  readonly wide?: boolean;
}

/** `<NewPasswordForm label="Activate" submit="Save password" onSubmit={(password) => save(password)} />` */
export function NewPasswordForm({ label, submit, onSubmit, describe, children, wide = false }: NewPasswordFormProps) {
  const { t } = useI18n();
  const [busy, setBusy] = useState(false);
  const [password, setPassword] = useState("");
  const [copy, setCopy] = useState("");
  const [rejected, setRejected] = useState("");
  const [problem, setProblem] = useState("");
  const [tried, setTried] = useState(false);

  const blank = hasBlank(password) ? t("password.rule.whitespace") : "";
  const mismatch = (tried || copy.length >= password.length) && copy !== "" && copy !== password ? t("password.mismatch") : "";

  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setTried(true);
    if (blank || copy !== password) return;
    const element = event.currentTarget;
    setBusy(true);
    setRejected("");
    setProblem("");
    try {
      await onSubmit(password, new FormData(element));
      element.reset();
      setPassword("");
      setCopy("");
      setTried(false);
    } catch (failure) {
      const rule = rejectedPassword(failure, t);
      if (rule) setRejected(rule);
      else setProblem(describe?.(failure) ?? t("error.generic"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack component="form" spacing={2.5} onSubmit={(event) => void send(event)} aria-label={label} noValidate={false}>
      {children}
      <PasswordField
        name="new"
        label={t("password.new")}
        autoComplete="new-password"
        required
        value={password}
        onChange={(event) => {
          setPassword(event.target.value);
          setRejected("");
        }}
        hint={blank || rejected ? undefined : t("password.hint")}
        error={blank || rejected || undefined}
      />
      <PasswordField
        name="confirm"
        label={t("password.confirm")}
        autoComplete="new-password"
        required
        value={copy}
        onChange={(event) => setCopy(event.target.value)}
        error={mismatch || undefined}
      />
      {problem ? (
        <Alert severity="error" role="alert">
          {problem}
        </Alert>
      ) : null}
      <Box>
        <Button type="submit" variant="filled" icon={wide ? undefined : "lock"} size={wide ? "large" : undefined} fullWidth={wide} busy={busy}>
          {submit}
        </Button>
      </Box>
    </Stack>
  );
}
