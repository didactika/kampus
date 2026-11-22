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
 * Confirming a new email from its link. The person confirms with a button rather than on opening the page, so a mail
 * scanner that opens links never confirms in their place.
 */

import Stack from "@mui/material/Stack";
import { useState } from "react";

import { Button, Loading } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { useI18n } from "../../i18n/I18nProvider";
import { useLinkToken } from "./link-token";
import { Outcome, StepHeading } from "./Outcome";

/** The page that confirms a new email. */
export function EmailConfirmation() {
  const { t } = useI18n();
  const token = useLinkToken();
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<"done" | "invalid" | null>(null);

  if (token === undefined) return <Loading label={t("loading")} />;
  if (!token || result === "invalid") {
    return (
      <Outcome title={t("email.confirm.title")} severity="error">
        {t("link.invalid")}
      </Outcome>
    );
  }
  if (result === "done") {
    return (
      <Outcome title={t("email.confirm.title")} severity="success">
        {t("email.confirm.done")}
      </Outcome>
    );
  }

  async function confirm() {
    setBusy(true);
    try {
      await browserApi("/identity/auth/email/confirm", { method: "POST", body: { token } });
      setResult("done");
    } catch {
      setResult("invalid");
    }
  }

  return (
    <>
      <StepHeading title={t("email.confirm.title")} lead={t("email.confirm.lead")} />
      <Stack>
        <Button variant="filled" size="large" fullWidth busy={busy} onClick={() => void confirm()}>
          {t("email.confirm.submit")}
        </Button>
      </Stack>
    </>
  );
}
