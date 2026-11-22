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

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import { Button, Icon, StatusBadge } from "@kampus/ui";

import type { Inbox } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { describe } from "./describe";

/** `<LatestNotifications inbox={…} />` */
export function LatestNotifications({ inbox }: { readonly inbox: Inbox }) {
  const { t, date } = useI18n();
  return (
    <Card component="section" aria-labelledby="latest-notifications">
      <Stack direction="row" sx={{ alignItems: "center", justifyContent: "space-between", px: 2.5, pt: 2, pb: 1 }}>
        <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
          <Typography id="latest-notifications" variant="h4" component="h2">
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
        <Stack component="ul" sx={{ listStyle: "none", m: 0, p: 1 }}>
          {inbox.notifications.map((n) => (
            <Stack component="li" key={n.id} direction="row" spacing={1.5} sx={{ alignItems: "flex-start", p: 1.5, borderRadius: 2, bgcolor: n.read_at ? "transparent" : "primary.container" }}>
              <Box sx={{ display: "grid", placeItems: "center", inlineSize: 36, blockSize: 36, borderRadius: 2, flex: "none", bgcolor: n.read_at ? "surface.containerHigh" : "background.paper", color: "primary.main" }}>
                <Icon name="notifications" size="small" />
              </Box>
              <Box sx={{ minInlineSize: 0 }}>
                <Typography variant="subtitle2" noWrap>
                  {describe(n.type)}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  <time dateTime={n.created_at}>{date(n.created_at)}</time>
                </Typography>
              </Box>
            </Stack>
          ))}
        </Stack>
      )}
    </Card>
  );
}
