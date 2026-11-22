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

/** The end of a step before signing in: a heading, what happened, and the way back to the sign-in. */

import Alert from "@mui/material/Alert";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import type { ReactNode } from "react";

import { Button } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** The props of `Outcome` and `StepHeading`. */
interface StepProps {
  readonly title: string;
  readonly lead?: string;
}

/** The heading of a step: its title and what it is for. */
export function StepHeading({ title, lead }: StepProps) {
  return (
    <>
      <Typography variant="h1" sx={{ fontSize: { xs: "1.75rem", md: "2rem" } }}>
        {title}
      </Typography>
      {lead ? (
        <Typography color="text.secondary" sx={{ mt: 1, mb: 4 }}>
          {lead}
        </Typography>
      ) : (
        <Stack sx={{ mb: 4 }} />
      )}
    </>
  );
}

/** `<Outcome title="Account active" severity="success">You can now sign in.</Outcome>` */
export function Outcome({ title, severity, children }: { readonly title: string; readonly severity: "success" | "error" | "info"; readonly children: ReactNode }) {
  const { t } = useI18n();
  const router = useRouter();
  return (
    <>
      <StepHeading title={title} />
      <Stack spacing={3}>
        <Alert severity={severity} role={severity === "error" ? "alert" : "status"}>
          {children}
        </Alert>
        <Button variant="filled" size="large" fullWidth onClick={() => router.push("/sign-in")}>
          {t("auth.signin")}
        </Button>
      </Stack>
    </>
  );
}

/** The way back to the sign-in, under a form. */
export function BackToSignIn() {
  const { t } = useI18n();
  const router = useRouter();
  return (
    <Button variant="text" icon="arrow_back" onClick={() => router.push("/sign-in")} sx={{ mt: 2 }}>
      {t("auth.back")}
    </Button>
  );
}
