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

/** The inbox of the person: how many are unread, each notification with its date, and marking them all as read. */

import Card from "@mui/material/Card";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useState } from "react";

import { Button, Icon, StatusBadge, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Inbox as InboxPage } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { PageHeader } from "../shell/PageHeader";

/** What a notification says: its type, in plain words. */
function describe(type: string): string {
  const words = type.replace(/[._]/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
}

/** `<Inbox inbox={…} />` */
export function Inbox({ inbox }: { readonly inbox: InboxPage }) {
  const { t, date } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [busy, setBusy] = useState(false);

  async function readAll() {
    setBusy(true);
    try {
      await browserApi("/notify/inbox/read-all", { method: "POST" });
      toast(t("notifications.allread"), "success");
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <PageHeader
        title={t("notifications.title")}
        lead={t("notifications.unread", { n: inbox.unread })}
        actions={
          inbox.unread > 0 ? (
            <Button variant="tonal" icon="check" busy={busy} onClick={() => void readAll()}>
              {t("notifications.markall")}
            </Button>
          ) : null
        }
      />
      <Card>
        {inbox.notifications.length === 0 ? (
          <EmptyState icon="notifications" message={t("notifications.empty")} />
        ) : (
          <List disablePadding>
            {inbox.notifications.map((n, at) => (
              <ListItem
                key={n.id}
                divider={at < inbox.notifications.length - 1}
                secondaryAction={n.read_at ? <StatusBadge label={t("notifications.read")} /> : null}
                sx={{ py: 1.5, bgcolor: n.read_at ? "transparent" : "primary.container" }}
              >
                <ListItemIcon sx={{ color: n.read_at ? "text.secondary" : "primary.onContainer" }}>
                  <Icon name="notifications" />
                </ListItemIcon>
                <ListItemText
                  primary={<Typography variant="subtitle1">{describe(n.type)}</Typography>}
                  secondary={<time dateTime={n.created_at}>{date(n.created_at)}</time>}
                />
              </ListItem>
            ))}
          </List>
        )}
      </Card>
    </>
  );
}
