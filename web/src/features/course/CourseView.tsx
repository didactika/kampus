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


"use client";

/**
 * A course as the person sees it: a head with its cover, its names, its category, dates and language, and how far the
 * person has gone; then its content by section, the grades of the person and the dates ahead, as tabs.
 */

import Accordion from "@mui/material/Accordion";
import AccordionDetails from "@mui/material/AccordionDetails";
import AccordionSummary from "@mui/material/AccordionSummary";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import Chip from "@mui/material/Chip";
import LinearProgress from "@mui/material/LinearProgress";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Tabs from "@mui/material/Tabs";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { useId, useState } from "react";

import { CoverArt, Icon, ProgressRing, StatusBadge, type StatusTone } from "@kampus/ui";

import type { ActivityState, CalendarItem, Course, CourseModule, Grade, Section } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { subjectIcon } from "../courses/subject";
import { EmptyState } from "../shell/EmptyState";

/** The tone of each state of an activity. */
const TONES: Readonly<Record<string, StatusTone>> = {
  not_started: "neutral",
  in_progress: "info",
  done: "success",
  done_passed: "success",
  done_failed: "danger",
};

/** The icon of each type of activity. */
const TYPE_ICONS: Readonly<Record<string, string>> = { "activities.page": "description", "activities.file": "attach_file" };

/** The icon of each kind of date. */
const DATE_ICONS: Readonly<Record<string, string>> = { opens: "event", due: "schedule", closes: "event", cutoff: "warning", entry: "calendar_month" };

/** Whether a state counts as done. */
const isDone = (state: string | undefined) => state === "done" || state === "done_passed";

/** The props of `CourseView`. */
export interface CourseViewProps {
  readonly course: Course;
  readonly categoryPath: string | null;
  readonly sections: readonly Section[];
  readonly modules: readonly (readonly CourseModule[])[];
  readonly progress: readonly ActivityState[];
  readonly grades: readonly Grade[];
  readonly recordTitles: Readonly<Record<string, string>>;
  readonly items: readonly CalendarItem[];
}

/** `<CourseView course={…} … />` */
export function CourseView({ course, categoryPath, sections, modules, progress, grades, recordTitles, items }: CourseViewProps) {
  const { t, day, date, number } = useI18n();
  const [tab, setTab] = useState(0);
  const tabsId = useId();
  const stateOf = new Map(progress.map((p) => [p.course_module, p.state]));
  const all = modules.flat();
  const done = all.filter((m) => isDone(stateOf.get(m.id))).length;
  const percent = all.length ? (done / all.length) * 100 : 0;
  const dates =
    course.starts_at && course.ends_at ? t("course.dates", { from: day(course.starts_at), to: day(course.ends_at) }) : course.starts_at ? t("course.starts", { date: day(course.starts_at) }) : null;

  return (
    <>
      <Button component={Link} href="/" variant="text" startIcon={<Icon name="arrow_back" size="small" />} sx={{ mb: 2, ml: -1.5 }}>
        {t("nav.courses")}
      </Button>
      <Card sx={{ mb: 4, overflow: "hidden", boxShadow: 2, border: 0 }}>
        <CoverArt seed={course.id} icon={subjectIcon(course.full_name, course.short_name, categoryPath)} height={180} />
        <Stack direction={{ xs: "column", md: "row" }} spacing={3} sx={{ p: { xs: 3, md: 4 }, alignItems: { md: "center" } }}>
          <Box sx={{ flex: 1, minInlineSize: 0 }}>
            <Typography variant="overline" color="text.secondary">
              {[course.code ?? course.short_name, categoryPath].filter(Boolean).join(" · ")}
            </Typography>
            <Typography variant="h1" sx={{ mt: 0.5 }}>
              {course.full_name}
            </Typography>
            {course.summary ? (
              <Typography color="text.secondary" sx={{ mt: 1.5, maxInlineSize: "70ch" }}>
                {course.summary}
              </Typography>
            ) : null}
            <Stack direction="row" useFlexGap sx={{ flexWrap: "wrap", gap: 1, mt: 2 }}>
              {dates ? <Chip icon={<Icon name="event" size="small" />} label={dates} variant="outlined" /> : null}
              {course.language_code ? <Chip icon={<Icon name="translate" size="small" />} label={course.language_code.toUpperCase()} variant="outlined" /> : null}
              <Chip icon={<Icon name="menu_book" size="small" />} label={t("course.section.count", { n: all.length })} variant="outlined" />
            </Stack>
          </Box>
          {all.length > 0 ? (
            <Stack spacing={1} sx={{ alignItems: "center", p: 2.5, borderRadius: 4, bgcolor: "primary.container", color: "primary.onContainer", minInlineSize: 200 }}>
              <ProgressRing value={percent} label={t("course.progress.label")} />
              <Typography variant="body2" sx={{ textAlign: "center" }}>
                {t("course.progress.done", { done, total: all.length })}
              </Typography>
            </Stack>
          ) : null}
        </Stack>
      </Card>

      <Tabs value={tab} onChange={(_, next: number) => setTab(next)} aria-label={course.full_name} sx={{ mb: 3 }}>
        {[
          ["course.tab.content", "menu_book"],
          ["course.tab.grades", "check_circle"],
          ["course.tab.calendar", "calendar_month"],
        ].map(([label, icon], at) => (
          <Tab key={label} id={`${tabsId}-tab-${at}`} aria-controls={`${tabsId}-panel-${at}`} icon={<Icon name={icon!} size="small" />} iconPosition="start" label={t(label!)} />
        ))}
      </Tabs>

      <Box role="tabpanel" id={`${tabsId}-panel-0`} aria-labelledby={`${tabsId}-tab-0`} hidden={tab !== 0}>
        <Stack spacing={1.5}>
          {sections.map((section, at) => {
            const list = modules[at] ?? [];
            const sectionDone = list.filter((m) => isDone(stateOf.get(m.id))).length;
            return (
              <Accordion key={section.id} defaultExpanded={at < 2} disableGutters sx={{ borderRadius: 3, "&::before": { display: "none" }, border: 1, borderColor: "divider" }} elevation={0}>
                <AccordionSummary expandIcon={<Icon name="keyboard_arrow_down" />} sx={{ px: 2.5, py: 0.5 }}>
                  <Stack direction="row" spacing={2} sx={{ alignItems: "center", flex: 1, mr: 2 }}>
                    <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, borderRadius: 2.5, bgcolor: "secondary.container", color: "secondary.onContainer", flex: "none", typography: "subtitle1" }}>
                      {at + 1}
                    </Box>
                    <Box sx={{ flex: 1, minInlineSize: 0 }}>
                      <Typography variant="h4" component="h3">
                        {section.title ?? t("course.untitled", { n: at + 1 })}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {t("course.section.count", { n: list.length })}
                      </Typography>
                    </Box>
                    {list.length > 0 ? (
                      <Box sx={{ inlineSize: 120, display: { xs: "none", sm: "block" } }}>
                        <LinearProgress variant="determinate" value={(sectionDone / list.length) * 100} aria-label={t("course.progress.done", { done: sectionDone, total: list.length })} sx={{ blockSize: 8, borderRadius: 999 }} />
                      </Box>
                    ) : null}
                  </Stack>
                </AccordionSummary>
                <AccordionDetails sx={{ px: 2.5, pt: 0 }}>
                  {list.length === 0 ? (
                    <Typography variant="body2" color="text.secondary">
                      {t("course.nothing")}
                    </Typography>
                  ) : (
                    <List disablePadding>
                      {list.map((module) => {
                        const state = stateOf.get(module.id);
                        return (
                          <ListItem key={module.id} disableGutters sx={{ py: 1, borderTop: 1, borderColor: "divider" }} secondaryAction={state ? <StatusBadge tone={TONES[state] ?? "neutral"} label={t(`state.${state}`)} /> : null}>
                            <ListItemIcon sx={{ minInlineSize: 48 }}>
                              <Box sx={{ display: "grid", placeItems: "center", inlineSize: 36, blockSize: 36, borderRadius: 2, bgcolor: isDone(state) ? "success.container" : "primary.container", color: isDone(state) ? "success.onContainer" : "primary.onContainer" }}>
                                <Icon name={isDone(state) ? "check" : (TYPE_ICONS[module.type_code] ?? "auto_stories")} size="small" />
                              </Box>
                            </ListItemIcon>
                            <ListItemText primary={module.title} slotProps={{ primary: { variant: "subtitle1" } }} />
                          </ListItem>
                        );
                      })}
                    </List>
                  )}
                </AccordionDetails>
              </Accordion>
            );
          })}
        </Stack>
      </Box>

      <Box role="tabpanel" id={`${tabsId}-panel-1`} aria-labelledby={`${tabsId}-tab-1`} hidden={tab !== 1}>
        <Card>
          {grades.length === 0 ? (
            <EmptyState icon="check_circle" message={t("course.nothing")} />
          ) : (
            <Table>
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
                    <TableCell>{recordTitles[grade.record] ?? grade.record}</TableCell>
                    <TableCell align="right">
                      <Typography variant="subtitle1" component="span">
                        {grade.final === null ? (grade.text ?? "") : number(grade.final)}
                      </Typography>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </Card>
      </Box>

      <Box role="tabpanel" id={`${tabsId}-panel-2`} aria-labelledby={`${tabsId}-tab-2`} hidden={tab !== 2}>
        <Card>
          <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pt: 2 }}>
            {t("calendar.next")}
          </Typography>
          {items.length === 0 ? (
            <EmptyState icon="calendar_month" message={t("course.nothing")} />
          ) : (
            <List>
              {items.map((item) => (
                <ListItem key={`${item.kind}-${item.starts_at}-${item.title}`} sx={{ px: 2.5 }}>
                  <ListItemIcon>
                    <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, borderRadius: 2.5, bgcolor: "accent.container", color: "accent.onContainer" }}>
                      <Icon name={DATE_ICONS[item.kind] ?? "event"} size="small" />
                    </Box>
                  </ListItemIcon>
                  <ListItemText primary={item.title} secondary={<><strong>{t(`calendar.${item.kind}`)}</strong> · <time dateTime={item.starts_at}>{date(item.starts_at)}</time></>} />
                </ListItem>
              ))}
            </List>
          )}
        </Card>
      </Box>
    </>
  );
}
