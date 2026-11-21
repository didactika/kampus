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

/** A problem the API answered with: its status and the code of its problem details. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
  ) {
    super(code);
  }
}

/** Asks the API and returns its JSON; a failure is an `ApiError` with the code of the problem. */
export async function api<T>(path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
  const method = init.method ?? "GET";
  const headers: Record<string, string> = { Accept: "application/json" };
  if (method !== "GET") headers["X-Kampus"] = "1";
  if (init.body !== undefined) headers["Content-Type"] = "application/json";
  const response = await fetch(`/api/v1${path}`, {
    method,
    headers,
    credentials: "same-origin",
    body: init.body === undefined ? undefined : JSON.stringify(init.body),
  });
  if (response.status === 204) return undefined as T;
  const text = await response.text();
  const data: unknown = text ? JSON.parse(text) : undefined;
  if (!response.ok) {
    const code = typeof data === "object" && data !== null && "code" in data ? String((data as { code: unknown }).code) : "unknown";
    throw new ApiError(response.status, code);
  }
  return data as T;
}

/** The account that is signed in. */
export interface Me {
  id: string;
  campus: string;
  username: string;
  email: string;
}

/** A category of courses. */
export interface Category {
  id: string;
  name: string;
}

/** A course. */
export interface Course {
  id: string;
  short_name: string;
  full_name: string;
  summary: string | null;
}

/** A section of a course. */
export interface Section {
  id: string;
  title: string | null;
}

/** A course module. */
export interface CourseModule {
  id: string;
  title: string;
  type_code: string;
}

/** A grade of the caller. */
export interface Grade {
  record: string;
  final: string | null;
  text: string | null;
}

/** A record of a course. */
export interface GradeRecord {
  id: string;
  title: string;
  max: string;
}

/** What the course records hold. */
export interface Tree {
  records: GradeRecord[];
}

/** The state of the caller on a course module. */
export interface ActivityState {
  course_module: string;
  state: string;
}

/** One thing a calendar shows. */
export interface CalendarItem {
  kind: string;
  title: string;
  starts_at: string;
  ends_at: string | null;
}

/** A notification of the inbox. */
export interface Notification {
  id: string;
  type: string;
  payload: Record<string, unknown>;
  created_at: string;
  read_at: string | null;
}

/** A page of the inbox. */
export interface Inbox {
  notifications: Notification[];
  unread: number;
}
