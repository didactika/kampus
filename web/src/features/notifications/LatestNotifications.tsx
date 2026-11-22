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

/** A short list of the latest notifications, for a widget: unread ones stand out, and a link leads to the inbox. */

import Card from "@mui/material/Card";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useId } from "react";

import { Button, StatusBadge } from "@kampus/ui";

import type { Inbox } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { NotificationRow } from "./NotificationRow";

/** `<LatestNotifications inbox={…} />` */
export function LatestNotifications({ inbox }: { readonly inbox: Inbox }) {
  const { t } = useI18n();
  const heading = useId();
  return (
    <Card component="section" aria-labelledby={heading}>
      <Stack direction="row" sx={{ alignItems: "center", justifyContent: "space-between", px: 2.5, pt: 2, pb: 1 }}>
        <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
          <Typography id={heading} variant="h4" component="h2">
            {t("widget.notify.inbox")}
          </Typography>
          {inbox.unread > 0 ? <StatusBadge tone="primary" label={String(inbox.unread)} /> : null}
        </Stack>
        <Button variant="text" size="small" href="/notifications">
          {t("inbox.all")}
        </Button>
      </Stack>
      {inbox.notifications.length === 0 ? (
        <EmptyState icon="notifications" message={t("notifications.empty")} />
      ) : (
        <Stack spacing={0.5} sx={{ p: 1 }}>
          {inbox.notifications.map((n) => (
            <NotificationRow key={n.id} notification={n} dense />
          ))}
        </Stack>
      )}
    </Card>
  );
}
