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


/**
 * A course: read on the server with everything the person may see of it (sections, activities and their state,
 * grades, dates of the next 30 days); a part the person may not see, or that the installation does not offer, is
 * left out.
 */

import type { Metadata } from "next";

import { optional, serverApi } from "../../../../api/server";
import type { ActivityState, CalendarItem, Category, Course, CourseModule, Grade, Section, Tree } from "../../../../api/types";
import { CourseView } from "../../../../features/course/CourseView";

/** The props of a page of a course. */
interface CoursePageProps {
  readonly params: Promise<{ readonly id: string }>;
}

/** The title of the page: the name of the course. */
export async function generateMetadata({ params }: CoursePageProps): Promise<Metadata> {
  const { id } = await params;
  const course = await optional<Course | null>(`/courses/courses/${id}`, null);
  return { title: course?.full_name ?? "" };
}

/** A course as the person sees it. */
export default async function CoursePage({ params }: CoursePageProps) {
  const { id } = await params;
  const [course, sections] = await Promise.all([serverApi<Course>(`/courses/courses/${id}`), serverApi<Section[]>(`/courses/courses/${id}/sections`)]);
  const from = new Date();
  const to = new Date(from.getTime() + 30 * 24 * 3600 * 1000);
  const window = `from=${encodeURIComponent(from.toISOString())}&to=${encodeURIComponent(to.toISOString())}`;
  const [category, modules, tree, grades, progress, items] = await Promise.all([
    optional<Category | null>(`/courses/categories/${course.category}`, null),
    Promise.all(sections.map((s) => optional<CourseModule[]>(`/activities/sections/${s.id}/course-modules`, []))),
    optional<Tree>(`/records/courses/${id}/records`, { records: [] }),
    optional<Grade[]>(`/records/courses/${id}/my-grades`, []),
    optional<ActivityState[]>(`/progress/courses/${id}/my-progress`, []),
    optional<CalendarItem[]>(`/calendar/courses/${id}/calendar?${window}`, []),
  ]);
  return (
    <CourseView
      course={course}
      categoryPath={category?.name ?? null}
      sections={sections}
      modules={modules}
      progress={progress}
      grades={grades}
      recordTitles={Object.fromEntries(tree.records.map((r) => [r.id, r.title]))}
      items={items}
    />
  );
}
