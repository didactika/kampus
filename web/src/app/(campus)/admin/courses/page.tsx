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

/** The courses of the campus by category, with the category asked with `?category=`, read on the server. */

import type { Metadata } from "next";

import { optional } from "../../../../api/server";
import type { Category, Course } from "../../../../api/types";
import { CourseBrowser } from "../../../../features/admin/courses/CourseBrowser";
import { currentAccount } from "../../../../features/session/current-account";
import { requestI18n } from "../../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: `${t("admin.page.courses")} · ${t("admin.title")}` };
}

/** The props of the page. */
interface CoursesProps {
  readonly searchParams: Promise<Record<string, string | string[] | undefined>>;
}

/** The deepest a trail of categories is followed. */
const DEEPEST = 20;

/** The categories and the courses of the one chosen. */
export default async function CoursesPage({ searchParams }: CoursesProps) {
  const asked = (await searchParams)["category"];
  const campus = (await currentAccount())!.campus;
  const top = await optional<Category[]>(`/courses/campuses/${campus}/categories`, []);
  const trail: Category[] = [];
  let next = typeof asked === "string" ? asked : null;
  while (next && trail.length < DEEPEST) {
    const category: Category | null = await optional<Category | null>(`/courses/categories/${next}`, null);
    if (!category) break;
    trail.unshift(category);
    next = category.parent;
  }
  const current = trail[trail.length - 1];
  const courses = current ? await optional<Course[] | null>(`/courses/categories/${current.id}/courses`, null) : null;
  return <CourseBrowser campus={campus} top={top} trail={trail} courses={courses} />;
}
