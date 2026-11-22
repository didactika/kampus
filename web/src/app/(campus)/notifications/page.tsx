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


/** The notifications of the person: all of them, or the unread ones with `?unread=1`. */

import type { Metadata } from "next";

import { serverApi } from "../../../api/server";
import type { Inbox as InboxPage } from "../../../api/types";
import { Inbox } from "../../../features/notifications/Inbox";
import { requestI18n } from "../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("notifications.title") };
}

/** The props of the page. */
interface NotificationsProps {
  readonly searchParams: Promise<Record<string, string | string[] | undefined>>;
}

/** The first page of the inbox, read on the server. */
export default async function NotificationsPage({ searchParams }: NotificationsProps) {
  const unreadOnly = (await searchParams)["unread"] === "1";
  const inbox = await serverApi<InboxPage>(`/notify/inbox?limit=20${unreadOnly ? "&unread=true" : ""}`);
  return <Inbox key={unreadOnly ? "unread" : "all"} inbox={inbox} unreadOnly={unreadOnly} />;
}
