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

import {
  type ActivityState,
  type CalendarItem,
  type Course,
  type CourseModule,
  type Grade,
  type Section,
  type Tree,
  ApiError,
  api,
} from "../lib/api";
import { html } from "../lib/html";
import { formatDate, formatNumber, t } from "../lib/i18n";
import { type Context, navigate } from "../lib/router";
import { me } from "../lib/session";

/** Calls the API; a part the person may not see, or that this installation does not offer, is left out. */
async function optional<T>(path: string, fallback: T): Promise<T> {
  try {
    return await api<T>(path);
  } catch (failure) {
    if (failure instanceof ApiError && failure.status < 500) return fallback;
    throw failure;
  }
}

/** A course: its sections and course modules, the caller's grades and progress, and the next dates. */
export async function course({ root, params }: Context): Promise<void> {
  const account = await me();
  if (!account) return navigate("/signin");
  const id = params["id"] ?? "";
  root.innerHTML = html`<p>${t("loading")}</p>`.value;
  const found = await api<Course>(`/courses/courses/${id}`);
  const sections = await api<Section[]>(`/courses/courses/${id}/sections`);
  const modules = await Promise.all(
    sections.map((s) => optional<CourseModule[]>(`/activities/sections/${s.id}/course-modules`, [])),
  );
  const from = new Date();
  const to = new Date(from.getTime() + 30 * 24 * 3600 * 1000);
  const [tree, grades, progress, items] = await Promise.all([
    optional<Tree>(`/records/courses/${id}/records`, { records: [] }),
    optional<Grade[]>(`/records/courses/${id}/my-grades`, []),
    optional<ActivityState[]>(`/progress/courses/${id}/my-progress`, []),
    optional<CalendarItem[]>(
      `/calendar/courses/${id}/calendar?from=${encodeURIComponent(from.toISOString())}&to=${encodeURIComponent(to.toISOString())}`,
      [],
    ),
  ]);
  const titleOf = new Map(tree.records.map((r) => [r.id, r.title]));
  const stateOf = new Map(progress.map((p) => [p.course_module, p.state]));
  root.innerHTML = html`
    <p><a data-link href="/">${t("back")}</a></p>
    <h1>${found.full_name}</h1>
    ${found.summary ? html`<p>${found.summary}</p>` : html``}
    <section class="card" aria-labelledby="sections">
      <h2 id="sections">${t("course.sections")}</h2>
      ${sections.map(
        (s, i) => html`
          <h3>${s.title ?? t("course.untitled", { n: i + 1 })}</h3>
          ${(modules[i] ?? []).length === 0
            ? html`<p class="muted">${t("course.nothing")}</p>`
            : html`<ul>
                ${(modules[i] ?? []).map(
                  (m) => html`<li>${m.title} ${stateOf.has(m.id) ? html`<span class="badge">${t(`state.${stateOf.get(m.id)}`)}</span>` : html``}</li>`,
                )}
              </ul>`}
        `,
      )}
    </section>
    <section class="card" aria-labelledby="grades">
      <h2 id="grades">${t("course.grades")}</h2>
      ${grades.length === 0
        ? html`<p class="muted">${t("course.nothing")}</p>`
        : html`<table>
            <thead><tr><th scope="col">${t("grades.record")}</th><th scope="col">${t("grades.grade")}</th></tr></thead>
            <tbody>
              ${grades.map(
                (g) => html`<tr><td>${titleOf.get(g.record) ?? g.record}</td><td>${g.final === null ? g.text ?? "" : formatNumber(g.final)}</td></tr>`,
              )}
            </tbody>
          </table>`}
    </section>
    <section class="card" aria-labelledby="calendar">
      <h2 id="calendar">${t("calendar.title")} <span class="muted">· ${t("calendar.next")}</span></h2>
      ${items.length === 0
        ? html`<p class="muted">${t("course.nothing")}</p>`
        : html`<ul>
            ${items.map((i) => html`<li><time datetime="${i.starts_at}">${formatDate(i.starts_at)}</time> — ${t(`calendar.${i.kind}`)}: ${i.title}</li>`)}
          </ul>`}
    </section>
  `.value;
}
