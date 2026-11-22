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


"use client";

/** The head of the page of a course: the way back, its cover, its code and category, its name, summary and facts. */

import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import Chip from "@mui/material/Chip";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Link from "next/link";

import { CoverArt, Icon, StatusBadge } from "@kampus/ui";

import type { CourseWithOwners } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { subjectIcon } from "../courses/subject";

/** The props of `CourseHeader`. */
export interface CourseHeaderProps {
  readonly course: CourseWithOwners;
  readonly categoryPath: string | null;
  /** How many activities the course has. */
  readonly activities: number;
}

/** `<CourseHeader course={…} categoryPath="Science" activities={12} />` */
export function CourseHeader({ course, categoryPath, activities }: CourseHeaderProps) {
  const { t, day } = useI18n();
  const dates =
    course.starts_at && course.ends_at ? t("course.dates", { from: day(course.starts_at), to: day(course.ends_at) }) : course.starts_at ? t("course.starts", { date: day(course.starts_at) }) : null;
  return (
    <>
      <Button component={Link} href="/courses" variant="text" startIcon={<Icon name="arrow_back" size="small" />} sx={{ mb: 2, ml: -1.5 }}>
        {t("nav.courses")}
      </Button>
      <Card sx={{ mb: 3, overflow: "hidden", border: 0, boxShadow: 2 }}>
        <CoverArt seed={course.id} icon={subjectIcon(course.full_name, course.short_name, categoryPath)} height={160}>
          {course.state !== "published" ? <StatusBadge tone={course.state === "draft" ? "warning" : "neutral"} label={t(`coursestate.${course.state}`)} /> : null}
        </CoverArt>
        <Box sx={{ p: { xs: 3, md: 4 } }}>
          <Typography variant="overline" color="text.secondary">
            {[course.code ?? course.short_name, categoryPath].filter(Boolean).join(" · ")}
          </Typography>
          <Typography variant="h1" sx={{ mt: 0.5 }}>
            {course.full_name}
          </Typography>
          {course.summary ? (
            <Typography color="text.secondary" sx={{ mt: 1.5, maxInlineSize: "75ch" }}>
              {course.summary}
            </Typography>
          ) : null}
          <Stack direction="row" useFlexGap sx={{ flexWrap: "wrap", gap: 1, mt: 2 }}>
            {dates ? <Chip icon={<Icon name="event" size="small" />} label={dates} variant="outlined" /> : null}
            {course.language_code ? <Chip icon={<Icon name="translate" size="small" />} label={course.language_code.toUpperCase()} variant="outlined" /> : null}
            <Chip icon={<Icon name="menu_book" size="small" />} label={t("course.section.count", { n: activities })} variant="outlined" />
          </Stack>
        </Box>
      </Card>
    </>
  );
}
