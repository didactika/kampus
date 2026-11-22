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
 * The content of a course by section: each section folds, with how many activities it has and how far the person has
 * gone in it; each activity has the icon of its type and the state of the person on it.
 */

import Accordion from "@mui/material/Accordion";
import AccordionDetails from "@mui/material/AccordionDetails";
import AccordionSummary from "@mui/material/AccordionSummary";
import Box from "@mui/material/Box";
import LinearProgress from "@mui/material/LinearProgress";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import { Icon, StatusBadge } from "@kampus/ui";

import type { ActivityState, CourseModule, Section } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { STATE_TONES, TYPE_ICONS, isDone } from "./activity-look";

/** The props of `CourseOutline`. */
export interface CourseOutlineProps {
  readonly sections: readonly Section[];
  readonly modules: readonly (readonly CourseModule[])[];
  readonly progress: readonly ActivityState[];
  /** Whether every section starts folded. */
  readonly collapsed?: boolean;
}

/** `<CourseOutline sections={…} modules={…} progress={…} />` */
export function CourseOutline({ sections, modules, progress, collapsed = false }: CourseOutlineProps) {
  const { t } = useI18n();
  const stateOf = new Map(progress.map((p) => [p.course_module, p.state as string]));
  return (
    <Stack spacing={1.5} component="section" aria-label={t("course.tab.content")}>
      {sections.map((section, at) => {
        const list = modules[at] ?? [];
        const done = list.filter((m) => isDone(stateOf.get(m.id))).length;
        return (
          <Accordion key={section.id} defaultExpanded={!collapsed && at < 2} disableGutters elevation={0}>
            <AccordionSummary expandIcon={<Icon name="keyboard_arrow_down" />} sx={{ px: 2.5, py: 0.5 }}>
              <Stack direction="row" spacing={2} sx={{ alignItems: "center", flex: 1, mr: 2 }}>
                <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, borderRadius: 2.5, flex: "none", backgroundImage: "var(--k-palette-gradient-soft)", color: "primary.main", typography: "subtitle1" }}>
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
                    <LinearProgress variant="determinate" value={(done / list.length) * 100} aria-label={t("course.progress.done", { done, total: list.length })} sx={{ blockSize: 8 }} />
                  </Box>
                ) : null}
              </Stack>
            </AccordionSummary>
            <AccordionDetails sx={{ px: 2.5, pt: 0 }}>
              {list.length === 0 ? (
                <Typography variant="body2" color="text.secondary">
                  {t("course.empty.section")}
                </Typography>
              ) : (
                <List disablePadding>
                  {list.map((module) => {
                    const state = stateOf.get(module.id);
                    return (
                      <ListItem key={module.id} disableGutters sx={{ py: 1, borderTop: 1, borderColor: "divider" }} secondaryAction={state ? <StatusBadge tone={STATE_TONES[state] ?? "neutral"} label={t(`state.${state}`)} /> : null}>
                        <ListItemIcon sx={{ minInlineSize: 48 }}>
                          <Box sx={{ display: "grid", placeItems: "center", inlineSize: 36, blockSize: 36, borderRadius: 2, bgcolor: isDone(state) ? "success.container" : "primary.container", color: isDone(state) ? "success.onContainer" : "primary.onContainer" }}>
                            <Icon name={isDone(state) ? "check" : (TYPE_ICONS[module.type] ?? "auto_stories")} size="small" />
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
  );
}
