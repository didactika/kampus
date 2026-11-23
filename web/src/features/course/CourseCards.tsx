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

/** The cards of the side of the page of a course: the progress of the person, their grades, the dates ahead and the summary. */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Stack from "@mui/material/Stack";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Typography from "@mui/material/Typography";
import { useId, type ReactNode } from "react";

import { Icon, ProgressRing } from "@kampus/ui";

import type { CalendarItem, CourseWithOwners, Grade } from "../../api/types";
import { EditableText } from "../editor/EditInPlace";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";

/**
 * A card with a heading and an icon. A heading set by a widget is written in place while its page is arranged; the
 * fallback is the title the card has otherwise.
 */
function SideCard({ heading, fallback, editable = false, icon, children }: { readonly heading?: unknown; readonly fallback: string; readonly editable?: boolean; readonly icon: string; readonly children: ReactNode }) {
  const id = useId();
  return (
    <Card component="section" aria-labelledby={id}>
      <CardContent sx={{ p: 2.5 }}>
        <Stack direction="row" spacing={1.25} sx={{ alignItems: "center", mb: 1.5 }}>
          <Box sx={{ display: "grid", placeItems: "center", inlineSize: 32, blockSize: 32, borderRadius: 2, backgroundImage: "var(--k-palette-gradient-soft)", color: "primary.main" }}>
            <Icon name={icon} size="small" />
          </Box>
          {editable ? (
            <EditableText id={id} setting="heading" value={heading} fallback={fallback} variant="h4" component="h2" />
          ) : (
            <Typography id={id} variant="h4" component="h2">
              {fallback}
            </Typography>
          )}
        </Stack>
        {children}
      </CardContent>
    </Card>
  );
}

/** How far the person has gone in the course. */
export function ProgressCard({ heading, done, total }: { readonly heading?: unknown; readonly done: number; readonly total: number }) {
  const { t } = useI18n();
  return (
    <SideCard editable heading={heading} fallback={t("course.progress")} icon="trending_up">
      {total === 0 ? (
        <EmptyState compact icon="trending_up" message={t("course.empty.progress")} />
      ) : (
        <Stack spacing={1.5} sx={{ alignItems: "center", py: 1 }}>
          <ProgressRing value={(done / total) * 100} label={t("course.progress.label")} size={112} />
          <Typography variant="body2" color="text.secondary">
            {t("course.progress.done", { done, total })}
          </Typography>
        </Stack>
      )}
    </SideCard>
  );
}

/** The grades of the person in the course. */
export function GradesCard({ heading, grades, titles }: { readonly heading?: unknown; readonly grades: readonly Grade[]; readonly titles: Readonly<Record<string, string>> }) {
  const { t, number } = useI18n();
  return (
    <SideCard editable heading={heading} fallback={t("course.grades")} icon="check_circle">
      {grades.length === 0 ? (
        <EmptyState compact icon="check_circle" message={t("course.empty.grades")} />
      ) : (
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell scope="col">{t("grades.record")}</TableCell>
              <TableCell scope="col" align="right">
                {t("grades.grade")}
              </TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {grades.map((grade) => (
              <TableRow key={grade.record} hover>
                <TableCell>{titles[grade.record] ?? grade.record}</TableCell>
                <TableCell align="right" sx={{ fontWeight: 600 }}>
                  {grade.final === null || grade.final === undefined ? (grade.text ?? "") : number(grade.final)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </SideCard>
  );
}

/** The icon of each kind of date. */
const DATE_ICONS: Readonly<Record<string, string>> = { opens: "event", due: "schedule", closes: "event", cutoff: "warning", entry: "calendar_month" };

/** The dates of the course in the days ahead. */
export function CalendarCard({ items, days }: { readonly items: readonly CalendarItem[]; readonly days: number }) {
  const { t, date } = useI18n();
  const until = Date.now() + days * 864e5;
  const shown = items.filter((item) => new Date(item.starts_at).getTime() <= until);
  return (
    <SideCard fallback={t("card.upcoming")} icon="calendar_month">
      <Typography variant="caption" color="text.secondary">
        {t("calendar.window", { n: days })}
      </Typography>
      {shown.length === 0 ? (
        <EmptyState compact icon="calendar_month" message={t("course.empty.dates")} />
      ) : (
        <List disablePadding>
          {shown.map((item) => (
            <ListItem key={`${item.kind}-${item.starts_at}-${item.title}`} disableGutters>
              <ListItemIcon sx={{ minInlineSize: 48 }}>
                <Box sx={{ display: "grid", placeItems: "center", inlineSize: 36, blockSize: 36, borderRadius: 2, bgcolor: "accent.container", color: "accent.onContainer" }}>
                  <Icon name={DATE_ICONS[item.kind] ?? "event"} size="small" />
                </Box>
              </ListItemIcon>
              <ListItemText
                primary={item.title}
                secondary={
                  <>
                    <strong>{t(`calendar.${item.kind}`)}</strong> · <time dateTime={item.starts_at}>{date(item.starts_at)}</time>
                  </>
                }
              />
            </ListItem>
          ))}
        </List>
      )}
    </SideCard>
  );
}

/** The summary of the course, its dates, language and owners. */
export function SummaryCard({ course }: { readonly course: CourseWithOwners }) {
  const { t, day } = useI18n();
  return (
    <SideCard fallback={t("card.summary")} icon="info">
      <Stack spacing={1.25}>
        {course.summary ? <Typography variant="body2">{course.summary}</Typography> : null}
        {course.starts_at ? (
          <Typography variant="body2" color="text.secondary">
            {course.ends_at ? t("course.dates", { from: day(course.starts_at), to: day(course.ends_at) }) : t("course.starts", { date: day(course.starts_at) })}
          </Typography>
        ) : null}
        {!course.summary && !course.starts_at ? (
          <Typography variant="body2" color="text.secondary">
            {t("course.nothing")}
          </Typography>
        ) : null}
      </Stack>
    </SideCard>
  );
}
