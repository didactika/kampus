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

/** What a page of a campus shows when it fails: a message, and a way to try again; the frame keeps working. */

import Alert from "@mui/material/Alert";
import Stack from "@mui/material/Stack";

import { Button } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** The page that failed. */
export default function CampusError({ error, reset }: { readonly error: Error & { digest?: string }; readonly reset: () => void }) {
  const { t } = useI18n();
  const denied = error.message === "permission_denied";
  return (
    <Stack spacing={2} sx={{ alignItems: "flex-start" }}>
      <Alert severity="error" role="alert">
        {denied ? t("error.denied") : t("error.generic")}
      </Alert>
      <Button variant="tonal" icon="refresh" onClick={reset}>
        {t("error.retry")}
      </Button>
    </Stack>
  );
}
