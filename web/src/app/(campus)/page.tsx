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

/** The home of a campus: its courses, grouped by category. */

import Box from "@mui/material/Box";
import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
import type { Metadata } from "next";

import { serverApi } from "../../api/server";
import type { Category, Course } from "../../api/types";
import { CourseCard } from "../../features/courses/CourseCard";
import { currentAccount } from "../../features/session/current-account";
import { EmptyState } from "../../features/shell/EmptyState";
import { PageHeader } from "../../features/shell/PageHeader";
import { requestI18n } from "../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("courses.title") };
}

/** The courses of the campus of the person, by category. */
export default async function CoursesPage() {
  const { t } = await requestI18n();
  const account = (await currentAccount())!;
  const categories = await serverApi<Category[]>(`/courses/campuses/${account.campus}/categories`);
  const groups = await Promise.all(
    categories.map(async (category) => ({ category, courses: await serverApi<Course[]>(`/courses/categories/${category.id}/courses`) })),
  );
  const shown = groups.filter((group) => group.courses.length > 0);
  return (
    <>
      <PageHeader title={t("courses.title")} lead={t("courses.lead")} />
      {shown.length === 0 ? <EmptyState icon="school" message={t("courses.empty")} /> : null}
      {shown.map(({ category, courses }) => (
        <Box component="section" key={category.id} aria-labelledby={`category-${category.id}`} sx={{ mb: 5 }}>
          <Typography id={`category-${category.id}`} variant="h3" component="h2" sx={{ mb: 2 }}>
            {category.name}
          </Typography>
          <Grid container spacing={2.5}>
            {courses.map((course) => (
              <Grid key={course.id} size={{ xs: 12, sm: 6, lg: 4 }}>
                <CourseCard course={course} />
              </Grid>
            ))}
          </Grid>
        </Box>
      ))}
    </>
  );
}
