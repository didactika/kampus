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

/** The sign-in form: the name and the password, one clear message when they are not right, and the campus after. */

import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useState, type FormEvent } from "react";

import { Button, TextField } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";

/** The form that signs a person in. */
export function SignInForm() {
  const { t } = useI18n();
  const router = useRouter();
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState("");

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
    <Card variant="elevation" sx={{ inlineSize: "100%", maxInlineSize: 420, boxShadow: 3, borderRadius: (theme) => `${theme.shape.large}px` }}>
      <CardContent sx={{ p: { xs: 3, sm: 5 } }}>
        <Stack spacing={3} component="form" onSubmit={(event) => void submit(event)} noValidate={false}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
            <Box component="img" src="/kampus-icon.svg" alt="" sx={{ inlineSize: 48, blockSize: 48 }} />
            <Typography variant="h1" component="h1">
              {t("signin.title")}
            </Typography>
          </Box>
          <TextField name="name" label={t("signin.name")} autoComplete="username" required autoFocus />
          <TextField name="password" type="password" label={t("signin.password")} autoComplete="current-password" required />
          {problem ? (
            <Alert severity="error" role="alert">
              {problem}
            </Alert>
          ) : null}
          <Button type="submit" variant="filled" size="large" busy={busy}>
            {t("signin.submit")}
          </Button>
        </Stack>
      </CardContent>
    </Card>
  );
}
