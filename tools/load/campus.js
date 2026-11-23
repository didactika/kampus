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
// @package   kampus
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/**
 * The load test of the API (requirement of performance: p95 under 200 ms with 1,000 concurrent people). Each virtual
 * person browses as the web client does: the home, the catalogue and a course, with the calls each page makes, and
 * a pause of one to three seconds between pages. It signs in once and shares the session, so sign-in throttling
 * does not shape the result.
 *
 * Environment: KAMPUS_LOAD_URL (the API, such as http://server:8080), KAMPUS_LOAD_USER, KAMPUS_LOAD_PASSWORD,
 * KAMPUS_LOAD_PEOPLE (1000 by default) and KAMPUS_LOAD_HOLD (how long the full load lasts, 2m by default).
 */

import http from "k6/http";
import { check, group, sleep } from "k6";

const BASE = `${__ENV.KAMPUS_LOAD_URL || "http://localhost:8100"}/api/v1`;
const PEOPLE = Number(__ENV.KAMPUS_LOAD_PEOPLE || 1000);

export const options = {
  scenarios: {
    browsing: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: Math.round(PEOPLE / 5) },
        { duration: "1m", target: PEOPLE },
        { duration: __ENV.KAMPUS_LOAD_HOLD || "2m", target: PEOPLE },
        { duration: "20s", target: 0 },
      ],
      gracefulRampDown: "10s",
    },
  },
  thresholds: {
    http_req_duration: ["p(95)<200"],
    "http_req_duration{group:::home}": ["p(95)<200"],
    "http_req_duration{group:::catalogue}": ["p(95)<200"],
    "http_req_duration{group:::course}": ["p(95)<200"],
    http_req_failed: ["rate<0.01"],
    checks: ["rate>0.99"],
  },
  summaryTrendStats: ["avg", "med", "p(90)", "p(95)", "p(99)", "max"],
};

/** Signs in once and finds the campus, a category and some courses to browse. */
export function setup() {
  const signIn = http.post(`${BASE}/identity/auth/sign-in`, JSON.stringify({ name: __ENV.KAMPUS_LOAD_USER, password: __ENV.KAMPUS_LOAD_PASSWORD }), {
    headers: { "content-type": "application/json", "x-kampus": "1" },
  });
  if (signIn.status !== 200) throw new Error(`sign-in failed: ${signIn.status}`);
  const session = signIn.cookies.kampus_session[0].value;
  const headers = { cookie: `kampus_session=${session}`, accept: "application/json" };
  const me = http.get(`${BASE}/identity/me`, { headers }).json();
  const categories = http.get(`${BASE}/courses/campuses/${me.campus}/categories`, { headers }).json();
  const category = (categories.items || categories)[0].id;
  const courses = http.get(`${BASE}/courses/categories/${category}/courses?limit=50`, { headers }).json();
  return { headers, campus: me.campus, category, courses: (courses.items || courses).map((c) => c.id) };
}

/** One person: a page, a pause, the next page. */
export default function browse(data) {
  const { headers, campus, category, courses } = data;
  const get = (path, name) => http.get(`${BASE}${path}`, { headers, tags: { name } });
  const ok = (responses) => check(responses, { "every call answered": (all) => all.every((r) => r.status >= 200 && r.status < 300) });

  group("home", () => {
    ok([
      get("/identity/me", "me"),
      get("/users/me/profile", "profile"),
      get(`/customise/scopes/${campus}/menus/customise.main`, "menu"),
      get(`/customise/scopes/${campus}/layouts/customise.dashboard`, "layout"),
      get("/notify/inbox?limit=5", "inbox"),
    ]);
  });
  sleep(1 + Math.random() * 2);

  group("catalogue", () => {
    ok([get(`/courses/campuses/${campus}/categories`, "categories"), get(`/courses/categories/${category}/courses?limit=24`, "courses")]);
  });
  sleep(1 + Math.random() * 2);

  const course = courses[Math.floor(Math.random() * courses.length)];
  const from = new Date().toISOString();
  const to = new Date(Date.now() + 30 * 86400000).toISOString();
  group("course", () => {
    ok([
      get(`/courses/courses/${course}`, "course"),
      get(`/courses/courses/${course}/sections`, "sections"),
      get(`/progress/courses/${course}/my-progress`, "my progress"),
      get(`/records/courses/${course}/my-grades`, "my grades"),
      get(`/calendar/courses/${course}/calendar?from=${from}&to=${to}`, "calendar"),
    ]);
  });
  sleep(1 + Math.random() * 2);
}
