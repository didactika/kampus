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
 * The inbox of the person: all or only the unread ones, grouped by day, newest first, each with its look and a way to
 * mark it read or open what it is about; every one at once can be marked read, and older ones load on request.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";

import { Button, StatusBadge, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Inbox as InboxPage, Notification } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { PageHeader } from "../shell/PageHeader";
import { NotificationRow } from "./NotificationRow";

/** The props of `Inbox`. */
export interface InboxProps {
  /** The first page. */
  readonly inbox: InboxPage;
  /** Whether it shows only the unread ones. */
  readonly unreadOnly: boolean;
}

/** The start of the day of a moment, as text, for grouping. */
function dayOf(iso: string): string {
  const d = new Date(iso);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).toISOString();
}

/** `<Inbox inbox={…} unreadOnly={false} />` */
export function Inbox({ inbox, unreadOnly }: InboxProps) {
  const { t, day } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [items, setItems] = useState<Notification[]>(inbox.notifications);
  const [next, setNext] = useState(inbox.next);
  const [unread, setUnread] = useState(inbox.unread);
  const [busy, setBusy] = useState(false);

  async function read(id: string) {
    try {
      const updated = await browserApi<Notification>(`/notify/inbox/${id}/read`, { method: "POST" });
      setItems((list) => (unreadOnly ? list.filter((n) => n.id !== id) : list.map((n) => (n.id === id ? updated : n))));
      setUnread((n) => Math.max(0, n - 1));
    } catch {
      toast(t("error.generic"), "danger");
    }
  }

  async function readAll() {
    setBusy(true);
    try {
      await browserApi("/notify/inbox/read-all", { method: "POST" });
      toast(t("notifications.allread"), "success");
      router.refresh();
      const stamp = new Date().toISOString();
      setItems((list) => (unreadOnly ? [] : list.map((n) => ({ ...n, read_at: n.read_at ?? stamp }))));
      setUnread(0);
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  async function more() {
    if (!next) return;
    setBusy(true);
    try {
      const page = await browserApi<InboxPage>(`/notify/inbox?limit=20&after=${next}${unreadOnly ? "&unread=true" : ""}`);
      setItems((list) => [...list, ...page.notifications]);
      setNext(page.next);
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  const today = dayOf(new Date().toISOString());
  const yesterday = dayOf(new Date(Date.now() - 864e5).toISOString());
  const groups = new Map<string, Notification[]>();
  for (const n of items) groups.set(dayOf(n.created_at), [...(groups.get(dayOf(n.created_at)) ?? []), n]);
  const title = (key: string) => (key === today ? t("notifications.today") : key === yesterday ? t("notifications.yesterday") : day(key));

  return (
    <>
      <PageHeader
        title={t("notifications.title")}
        lead={t("notifications.lead")}
        actions={
          <>
            <Button variant="outlined" icon="tune" href="/notifications/preferences">
              {t("notifications.preferences")}
            </Button>
            {unread > 0 ? (
              <Button variant="filled" icon="done_all" busy={busy} onClick={() => void readAll()}>
                {t("notifications.markall")}
              </Button>
            ) : null}
          </>
        }
      />
      <Tabs value={unreadOnly ? 1 : 0} sx={{ mb: 3 }} aria-label={t("notifications.title")}>
        <Tab component={Link} href="/notifications" label={t("notifications.tab.all")} />
        <Tab
          component={Link}
          href="/notifications?unread=1"
          label={
            <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
              <span>{t("notifications.tab.unread")}</span>
              {unread > 0 ? <StatusBadge tone="primary" label={String(unread)} /> : null}
            </Stack>
          }
        />
      </Tabs>
      {items.length === 0 ? (
        <Card>
          <EmptyState icon={unreadOnly ? "done_all" : "notifications"} message={unreadOnly ? t("notifications.uptodate") : t("notifications.empty")} />
        </Card>
      ) : (
        <Stack spacing={3}>
          {[...groups.entries()].map(([key, list]) => (
            <Box key={key} component="section" aria-label={title(key)}>
              <Typography variant="overline" color="text.secondary" sx={{ display: "block", mb: 1, ml: 1 }}>
                {title(key)}
              </Typography>
              <Card sx={{ p: 1 }}>
                <Stack spacing={0.5}>
                  {list.map((n) => (
                    <NotificationRow key={n.id} notification={n} onRead={(id) => void read(id)} />
                  ))}
                </Stack>
              </Card>
            </Box>
          ))}
          {next ? (
            <Button variant="outlined" icon="keyboard_arrow_down" busy={busy} onClick={() => void more()} sx={{ alignSelf: "center" }}>
              {t("notifications.more")}
            </Button>
          ) : null}
        </Stack>
      )}
    </>
  );
}
