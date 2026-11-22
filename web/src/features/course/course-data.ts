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


/**
 * What the page of a course reads once, on the server, for the person of the request: the course and its category,
 * its sections with their activities, and the progress, grades and dates of the person. Every widget of the page
 * takes what it shows from here, so none asks the API again. A part the person may not see, or that the installation
 * does not offer, is empty.
 */

import { optional, serverApi } from "../../api/server";
import type { ActivityState, CalendarItem, Category, CourseModule, CourseWithOwners, Grade, Section, Tree } from "../../api/types";

/** The days of dates read ahead, the most a widget may show. */
export const DAYS_AHEAD = 90;

/** What the page of a course read. */
export interface CourseData {
  readonly course: CourseWithOwners;
  readonly categoryPath: string | null;
  readonly sections: readonly Section[];
  readonly modules: readonly (readonly CourseModule[])[];
  readonly progress: readonly ActivityState[];
  readonly grades: readonly Grade[];
  readonly recordTitles: Readonly<Record<string, string>>;
  readonly items: readonly CalendarItem[];
}

/** Reads what the page of a course shows. */
export async function courseData(id: string): Promise<CourseData> {
  const [course, sections] = await Promise.all([serverApi<CourseWithOwners>(`/courses/courses/${id}`), serverApi<Section[]>(`/courses/courses/${id}/sections`)]);
  const from = new Date();
  const to = new Date(from.getTime() + DAYS_AHEAD * 24 * 3600 * 1000);
  const window = `from=${encodeURIComponent(from.toISOString())}&to=${encodeURIComponent(to.toISOString())}`;
  const [category, modules, tree, grades, progress, items] = await Promise.all([
    optional<Category | null>(`/courses/categories/${course.category}`, null),
    Promise.all(sections.map((s) => optional<CourseModule[]>(`/activities/sections/${s.id}/course-modules`, []))),
    optional<Tree>(`/records/courses/${id}/records`, { categories: [], records: [] }),
    optional<Grade[]>(`/records/courses/${id}/my-grades`, []),
    optional<ActivityState[]>(`/progress/courses/${id}/my-progress`, []),
    optional<CalendarItem[]>(`/calendar/courses/${id}/calendar?${window}`, []),
  ]);
  return {
    course,
    categoryPath: category?.name ?? null,
    sections,
    modules,
    progress,
    grades,
    recordTitles: Object.fromEntries(tree.records.map((r) => [r.id, r.title])),
    items,
  };
}
