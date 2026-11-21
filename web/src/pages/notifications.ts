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
// @package   web/src
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

import { type Inbox, api } from "../lib/api";
import { html } from "../lib/html";
import { formatDate, t } from "../lib/i18n";
import { type Context, navigate } from "../lib/router";
import { me } from "../lib/session";

/** What a notification says: its type, in the words of the person. */
function describe(type: string): string {
  return type.replace(/[._]/g, " ");
}

/** The inbox of the caller. */
export async function notifications(context: Context): Promise<void> {
  const { root } = context;
  if (!(await me())) return navigate("/signin");
  const inbox = await api<Inbox>("/notify/inbox?limit=50");
  root.innerHTML = html`
    <h1>${t("notifications.title")}</h1>
    <p aria-live="polite">${t("notifications.unread", { n: inbox.unread })}</p>
    ${inbox.unread > 0 ? html`<k-button id="read-all" variant="quiet">${t("notifications.markall")}</k-button>` : html``}
    ${inbox.notifications.length === 0
      ? html`<p class="muted">${t("notifications.empty")}</p>`
      : html`<ul>
          ${inbox.notifications.map(
            (n) => html`<li class="card">
              <strong>${describe(n.type)}</strong>
              <time datetime="${n.created_at}" class="muted">${formatDate(n.created_at)}</time>
              ${n.read_at ? html`<span class="badge">${t("notifications.read")}</span>` : html``}
            </li>`,
          )}
        </ul>`}
  `.value;
  root.querySelector("#read-all")?.addEventListener("click", () => {
    void api("/notify/inbox/read-all", { method: "POST" }).then(() => notifications(context));
  });
}
