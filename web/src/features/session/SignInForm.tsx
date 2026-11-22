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

/** The sign-in form: the name and the password (which can be shown), one clear message when they are not right. */

import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useState, type FormEvent } from "react";

import { Button, Icon, TextField } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";

/** The form that signs a person in. */
export function SignInForm() {
  const { t } = useI18n();
  const router = useRouter();
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState("");
  const [shown, setShown] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setBusy(true);
    setProblem("");
    try {
      await browserApi("/identity/auth/sign-in", { method: "POST", body: { name: form.get("name"), password: form.get("password") } });
      router.replace("/");
      router.refresh();
    } catch (failure) {
      setProblem(failure instanceof ApiError && failure.status < 500 ? t("signin.failed") : t("error.generic"));
      setBusy(false);
    }
  }

  return (
    <Box sx={{ inlineSize: "100%", maxInlineSize: 400 }}>
      <Typography variant="h1" sx={{ fontSize: { xs: "1.75rem", md: "2rem" } }}>
        {t("signin.welcome")}
      </Typography>
      <Typography color="text.secondary" sx={{ mt: 1, mb: 4 }}>
        {t("signin.lead")}
      </Typography>
      <Stack spacing={2.5} component="form" onSubmit={(event) => void submit(event)} aria-label={t("signin.title")}>
        <TextField name="name" label={t("signin.name")} autoComplete="username" required autoFocus startAdornment={<InputAdornment position="start"><Icon name="person" size="small" /></InputAdornment>} />
        <TextField
          name="password"
          type={shown ? "text" : "password"}
          label={t("signin.password")}
          autoComplete="current-password"
          required
          startAdornment={
            <InputAdornment position="start">
              <Icon name="lock" size="small" />
            </InputAdornment>
          }
          endAdornment={
            <InputAdornment position="end">
              <IconButton edge="end" size="small" aria-label={shown ? t("signin.hide") : t("signin.show")} aria-pressed={shown} onClick={() => setShown((s) => !s)}>
                <Icon name={shown ? "visibility_off" : "visibility"} size="small" />
              </IconButton>
            </InputAdornment>
          }
        />
        {problem ? (
          <Alert severity="error" role="alert">
            {problem}
          </Alert>
        ) : null}
        <Button type="submit" variant="filled" size="large" busy={busy} fullWidth sx={{ mt: 1 }}>
          {t("signin.submit")}
        </Button>
      </Stack>
      <Stack direction="row" spacing={1} sx={{ alignItems: "center", justifyContent: "center", mt: 4, color: "text.secondary" }}>
        <Icon name="shield_person" size="small" />
        <Typography variant="caption">{t("signin.secure")}</Typography>
      </Stack>
    </Box>
  );
}
