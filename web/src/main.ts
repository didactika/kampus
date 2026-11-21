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

import { ApiError } from "./lib/api";
import { html } from "./lib/html";
import { setLanguage, t } from "./lib/i18n";
import { type Route, navigate, start } from "./lib/router";
import { me, signOut } from "./lib/session";
import { course } from "./pages/course";
import { courses } from "./pages/courses";
import { notifications } from "./pages/notifications";
import { signin } from "./pages/signin";
import { defineComponents } from "./ui/components";

setLanguage(navigator.language);
defineComponents();
document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
  el.textContent = t(el.dataset["i18n"] ?? "");
});

const app = document.getElementById("app");
if (!app) throw new Error("the page has no #app");

/** Draws the frame every page sits in: a header with the navigation and a main region. */
async function frame(): Promise<HTMLElement> {
  const account = await me();
  app!.innerHTML = html`
    <header class="shell">
      <a class="brand" data-link href="/">Kampus</a>
      ${account
        ? html`<nav aria-label="Primary">
            <ul>
              <li><a data-link href="/">${t("nav.courses")}</a></li>
              <li><a data-link href="/notifications">${t("nav.notifications")}</a></li>
            </ul>
          </nav>
          <span class="spacer"></span>
          <span class="muted">${account.username}</span>
          <k-button id="signout" variant="quiet">${t("nav.signout")}</k-button>`
        : html``}
    </header>
    <main id="main" tabindex="-1"></main>
  `.value;
  app!.querySelector("#signout")?.addEventListener("click", () => {
    void signOut().then(() => navigate("/signin"));
  });
  return app!.querySelector<HTMLElement>("main")!;
}

/** Wraps a page so that it sits in the frame and a failure is shown as a message, not as a blank page. */
function page(draw: Route["page"]): Route["page"] {
  return async (context) => {
    const main = await frame();
    try {
      await draw({ ...context, root: main });
    } catch (failure) {
      const denied = failure instanceof ApiError && (failure.status === 401 || failure.status === 403);
      if (failure instanceof ApiError && failure.status === 401) return navigate("/signin");
      main.innerHTML = html`<p class="error" role="alert">${denied ? t("error.denied") : t("error.generic")}</p>`.value;
    }
  };
}

start(
  app,
  [
    { path: "/", page: page(courses) },
    { path: "/signin", page: page(signin) },
    { path: "/courses/:id", page: page(course) },
    { path: "/notifications", page: page(notifications) },
  ],
  page(async ({ root }) => {
    root.innerHTML = html`<p>404</p>`.value;
  }),
);
