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

/**
 * A course in the catalogue, as a card or as a row. Its name opens it; its cover says its subject; its code, its
 * category, its dates and its language say the rest at a glance; a course not yet published or archived says so on
 * its cover.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Link from "next/link";

import { CoverArt, Icon, StatusBadge, type StatusTone } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";
import type { CatalogueCourse } from "./catalogue";
import { subjectIcon } from "./subject";

/** The tone of a state a course is not usually in. */
const UNUSUAL: Readonly<Partial<Record<CatalogueCourse["state"], StatusTone>>> = {
  draft: "warning",
  archived: "neutral",
  deletion_scheduled: "danger",
};

/** What a course says about its dates and language. */
function Facts({ course }: { readonly course: CatalogueCourse }) {
  const { t, day } = useI18n();
  const dates =
    course.starts_at && course.ends_at
      ? t("course.dates", { from: day(course.starts_at), to: day(course.ends_at) })
      : course.starts_at
        ? t("course.starts", { date: day(course.starts_at) })
        : null;
  if (!dates && !course.language_code) return null;
  return (
    <Stack direction="row" useFlexGap sx={{ flexWrap: "wrap", gap: 2, color: "text.secondary" }}>
      {dates ? (
        <Stack direction="row" spacing={0.75} sx={{ alignItems: "center" }}>
          <Icon name="event" size="small" />
          <Typography variant="caption">{dates}</Typography>
        </Stack>
      ) : null}
      {course.language_code ? (
        <Stack direction="row" spacing={0.75} sx={{ alignItems: "center" }}>
          <Icon name="translate" size="small" />
          <Typography variant="caption">{course.language_code.toUpperCase()}</Typography>
        </Stack>
      ) : null}
    </Stack>
  );
}

/** The badge of a course in an unusual state. */
function State({ course }: { readonly course: CatalogueCourse }) {
  const { t } = useI18n();
  const tone = UNUSUAL[course.state];
  return tone ? <StatusBadge tone={tone} label={t(`coursestate.${course.state}`)} /> : null;
}

/** `<CourseCard course={…} />`: a course as a card of the grid. */
export function CourseCard({ course }: { readonly course: CatalogueCourse }) {
  const icon = subjectIcon(course.full_name, course.short_name, course.categoryPath);
  return (
    <Card
      sx={{
        blockSize: "100%",
        transition: (theme) => theme.transitions.create(["box-shadow", "transform"], { duration: 250 }),
        "&:hover": { boxShadow: 4, transform: "translateY(-2px)" },
        "@media (prefers-reduced-motion: reduce)": { "&:hover": { transform: "none" } },
      }}
    >
      <CardActionArea component={Link} href={`/courses/${course.id}`} sx={{ blockSize: "100%", display: "flex", flexDirection: "column", alignItems: "stretch" }}>
        <CoverArt seed={course.id} icon={icon} height={128}>
          <State course={course} />
        </CoverArt>
        <Stack spacing={1} sx={{ p: 2.5, flex: 1 }}>
          <Typography variant="overline" color="text.secondary" noWrap>
            {[course.code ?? course.short_name, course.categoryPath].join(" · ")}
          </Typography>
          <Typography variant="h4" component="h3" sx={{ display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical", overflow: "hidden" }}>
            {course.full_name}
          </Typography>
          {course.summary ? (
            <Typography variant="body2" color="text.secondary" sx={{ display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical", overflow: "hidden" }}>
              {course.summary}
            </Typography>
          ) : null}
          <Box sx={{ flex: 1 }} />
          <Facts course={course} />
        </Stack>
      </CardActionArea>
    </Card>
  );
}

/** `<CourseRow course={…} />`: a course as a row of the list. */
export function CourseRow({ course }: { readonly course: CatalogueCourse }) {
  const icon = subjectIcon(course.full_name, course.short_name, course.categoryPath);
  return (
    <CardActionArea component={Link} href={`/courses/${course.id}`} sx={{ display: "flex", alignItems: "center", gap: 2, p: 1.5, borderRadius: 2 }}>
      <Box sx={{ inlineSize: 72, flex: "none", borderRadius: 2, overflow: "hidden" }}>
        <CoverArt seed={course.id} height={56} />
      </Box>
      <Box sx={{ flex: 1, minInlineSize: 0 }}>
        <Typography variant="subtitle1" noWrap>
          {course.full_name}
        </Typography>
        <Typography variant="body2" color="text.secondary" noWrap>
          {[course.code ?? course.short_name, course.categoryPath].join(" · ")}
        </Typography>
      </Box>
      <Box sx={{ display: { xs: "none", md: "block" } }}>
        <Facts course={course} />
      </Box>
      <State course={course} />
      <Box sx={{ color: "text.secondary", display: "flex" }}>
        <Icon name={icon} />
      </Box>
    </CardActionArea>
  );
}
