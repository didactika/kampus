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
 * A course: its sections with their course modules and the state of the person on each, the grades of the person,
 * and the dates of the next 30 days. A part the person may not see, or that the installation does not offer, is left
 * out.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Grid from "@mui/material/Grid";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemText from "@mui/material/ListItemText";
import Stack from "@mui/material/Stack";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Typography from "@mui/material/Typography";
import type { Metadata } from "next";

import { StatusBadge, type StatusTone } from "@kampus/ui";

import { optional, serverApi } from "../../../../api/server";
import type { ActivityState, CalendarItem, Course, CourseModule, Grade, Section, Tree } from "../../../../api/types";
import { EmptyState } from "../../../../features/shell/EmptyState";
import { PageHeader } from "../../../../features/shell/PageHeader";
import { requestI18n } from "../../../../i18n/server";

/** The tone of each state of an activity. */
const TONES: Readonly<Record<string, StatusTone>> = {
  not_started: "neutral",
  in_progress: "info",
  done: "success",
  done_passed: "success",
  done_failed: "danger",
};

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
  const { t, date, number } = await requestI18n();
  const [course, sections] = await Promise.all([
    serverApi<Course>(`/courses/courses/${id}`),
    serverApi<Section[]>(`/courses/courses/${id}/sections`),
  ]);
  const from = new Date();
  const to = new Date(from.getTime() + 30 * 24 * 3600 * 1000);
  const window = `from=${encodeURIComponent(from.toISOString())}&to=${encodeURIComponent(to.toISOString())}`;
  const [modules, tree, grades, progress, items] = await Promise.all([
    Promise.all(sections.map((s) => optional<CourseModule[]>(`/activities/sections/${s.id}/course-modules`, []))),
    optional<Tree>(`/records/courses/${id}/records`, { records: [] }),
    optional<Grade[]>(`/records/courses/${id}/my-grades`, []),
    optional<ActivityState[]>(`/progress/courses/${id}/my-progress`, []),
    optional<CalendarItem[]>(`/calendar/courses/${id}/calendar?${window}`, []),
  ]);
  const titleOf = new Map(tree.records.map((r) => [r.id, r.title]));
  const stateOf = new Map(progress.map((p) => [p.course_module, p.state]));

  return (
    <>
      <PageHeader title={course.full_name} lead={course.summary} back={{ href: "/", label: t("nav.courses") }} />
      <Grid container spacing={3}>
        <Grid size={{ xs: 12, md: 7, lg: 8 }}>
          <Typography variant="h3" component="h2" sx={{ mb: 2 }}>
            {t("course.sections")}
          </Typography>
          <Stack spacing={2}>
            {sections.map((section, at) => (
              <Card key={section.id} component="section" aria-labelledby={`section-${section.id}`}>
                <CardContent>
                  <Typography id={`section-${section.id}`} variant="h4" component="h3">
                    {section.title ?? t("course.untitled", { n: at + 1 })}
                  </Typography>
                  {(modules[at] ?? []).length === 0 ? (
                    <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                      {t("course.nothing")}
                    </Typography>
                  ) : (
                    <List disablePadding sx={{ mt: 1 }}>
                      {(modules[at] ?? []).map((module) => {
                        const state = stateOf.get(module.id);
                        return (
                          <ListItem key={module.id} divider disableGutters secondaryAction={state ? <StatusBadge tone={TONES[state] ?? "neutral"} label={t(`state.${state}`)} /> : null}>
                            <ListItemText primary={module.title} />
                          </ListItem>
                        );
                      })}
                    </List>
                  )}
                </CardContent>
              </Card>
            ))}
          </Stack>
        </Grid>
        <Grid size={{ xs: 12, md: 5, lg: 4 }}>
          <Stack spacing={3}>
            <Card component="section" aria-labelledby="grades">
              <CardContent>
                <Typography id="grades" variant="h4" component="h2">
                  {t("course.grades")}
                </Typography>
                {grades.length === 0 ? (
                  <EmptyState icon="check_circle" message={t("course.nothing")} />
                ) : (
                  <Table size="small" sx={{ mt: 1 }}>
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
                        <TableRow key={grade.record}>
                          <TableCell>{titleOf.get(grade.record) ?? grade.record}</TableCell>
                          <TableCell align="right">{grade.final === null ? (grade.text ?? "") : number(grade.final)}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                )}
              </CardContent>
            </Card>
            <Card component="section" aria-labelledby="calendar">
              <CardContent>
                <Typography id="calendar" variant="h4" component="h2">
                  {t("calendar.title")}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {t("calendar.next")}
                </Typography>
                {items.length === 0 ? (
                  <EmptyState icon="calendar_month" message={t("course.nothing")} />
                ) : (
                  <List disablePadding sx={{ mt: 1 }}>
                    {items.map((item) => (
                      <ListItem key={`${item.kind}-${item.starts_at}-${item.title}`} disableGutters divider>
                        <ListItemText
                          primary={item.title}
                          secondary={
                            <>
                              <Box component="span" sx={{ fontWeight: 500 }}>
                                {t(`calendar.${item.kind}`)}
                              </Box>{" "}
                              · <time dateTime={item.starts_at}>{date(item.starts_at)}</time>
                            </>
                          }
                        />
                      </ListItem>
                    ))}
                  </List>
                )}
              </CardContent>
            </Card>
          </Stack>
        </Grid>
      </Grid>
    </>
  );
}
