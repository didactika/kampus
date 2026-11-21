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

import { type Category, type Course, api } from "../lib/api";
import { html } from "../lib/html";
import { t } from "../lib/i18n";
import { type Context, navigate } from "../lib/router";
import { me } from "../lib/session";

/** The courses of the campus, by category. */
export async function courses({ root }: Context): Promise<void> {
  const account = await me();
  if (!account) return navigate("/signin");
  root.innerHTML = html`<h1>${t("courses.title")}</h1><p>${t("loading")}</p>`.value;
  const categories = await api<Category[]>(`/courses/campuses/${account.campus}/categories`);
  const groups = await Promise.all(
    categories.map(async (category) => ({
      category,
      courses: await api<Course[]>(`/courses/categories/${category.id}/courses`),
    })),
  );
  const shown = groups.filter((g) => g.courses.length > 0);
  root.innerHTML = html`
    <h1>${t("courses.title")}</h1>
    ${shown.length === 0
      ? html`<p class="muted">${t("courses.empty")}</p>`
      : shown.map(
          (g) => html`
            <section class="card" aria-labelledby="cat-${g.category.id}">
              <h2 id="cat-${g.category.id}">${g.category.name}</h2>
              <ul>
                ${g.courses.map(
                  (c) => html`<li><a data-link href="/courses/${c.id}">${c.full_name}</a> <span class="muted">${c.short_name}</span></li>`,
                )}
              </ul>
            </section>
          `,
        )}
  `.value;
}
