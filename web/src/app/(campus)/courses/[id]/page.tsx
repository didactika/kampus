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
 * A course: its head, then the widgets its teachers arranged for its page (courses.course), each taking what it shows
 * from what the page read once for the person; the default page when nothing was arranged. Whoever may arrange it
 * gets the editor.
 */

import type { Metadata } from "next";

import { optional } from "../../../../api/server";
import type { CourseWithOwners, Layout, Placement } from "../../../../api/types";
import { courseData } from "../../../../features/course/course-data";
import { CourseHeader } from "../../../../features/course/CourseHeader";
import { STANDARD_GRID } from "../../../../features/dashboard/default-layout";
import { ArrangedPage } from "../../../../features/layouts/ArrangedPage";
import { currentAccount } from "../../../../features/session/current-account";
import { WIDGETS } from "../../../../plugins/widgets.generated";

/** The page kind of a course. */
const COURSE_PAGE = "courses.course";

/** The page of a course until its teachers arrange their own: the content, and beside it the summary, progress and grades. */
const DEFAULT_COURSE: readonly Placement[] = [
  { widget: "widget_course_content", x: 0, y: 0, w: 8, h: 10, settings: { collapsed: false }, pinned: false, locked: false, held: false, permission: null },
  { widget: "widget_course_summary", x: 8, y: 0, w: 4, h: 4, settings: {}, pinned: false, locked: false, held: false, permission: null },
  { widget: "widget_course_progress", x: 8, y: 4, w: 4, h: 3, settings: { heading: "" }, pinned: false, locked: false, held: false, permission: null },
  { widget: "widget_course_grades", x: 8, y: 7, w: 4, h: 4, settings: { heading: "" }, pinned: false, locked: false, held: false, permission: null },
  { widget: "widget_upcoming", x: 0, y: 10, w: 4, h: 4, settings: { days: 30 }, pinned: false, locked: false, held: false, permission: null },
];

/** The props of a page of a course. */
interface CoursePageProps {
  readonly params: Promise<{ readonly id: string }>;
}

/** The title of the page: the name of the course. */
export async function generateMetadata({ params }: CoursePageProps): Promise<Metadata> {
  const { id } = await params;
  const course = await optional<CourseWithOwners | null>(`/courses/courses/${id}`, null);
  return { title: course?.full_name ?? "" };
}

/** A course as the person sees it. */
export default async function CoursePage({ params }: CoursePageProps) {
  const { id } = await params;
  const [account, data] = [(await currentAccount())!, await courseData(id)];
  const layout = await optional<Layout | null>(`/customise/scopes/${data.course.scope}/layouts/${COURSE_PAGE}`, null);
  const placements = layout?.placements.length ? layout.placements : DEFAULT_COURSE;
  const context = { campus: account.campus, course: data };
  const widgets = await Promise.all(
    placements.map(async (placement) => {
      const plugin = WIDGETS[placement.widget];
      return { placement, node: plugin ? await plugin.render({ settings: placement.settings, context, size: { w: placement.w, h: placement.h } }) : null };
    }),
  );
  return (
    <>
      <CourseHeader course={data.course} categoryPath={data.categoryPath} activities={data.modules.flat().length} />
      <ArrangedPage
        scope={data.course.scope}
        pageKind={COURSE_PAGE}
        grid={layout ? { columns: layout.grid.columns, row: layout.grid.row_pixels } : STANDARD_GRID}
        widgets={widgets}
        editable={layout?.editable ?? false}
        lockable={layout?.lockable ?? false}
        available={(layout?.widgets ?? []).filter((w) => w.code in WIDGETS)}
      />
    </>
  );
}
