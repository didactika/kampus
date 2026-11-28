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
 * The courses of the campus by category: the tree of categories beside the courses of the one chosen. A category
 * opens to show the ones under it; a category or a course is created from the head of the page, in the category chosen.
 */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";

import { Button, Icon, StatusBadge, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { Category, Course } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { formatDay } from "../../../i18n/translator";
import { PageHeader } from "../../shell/PageHeader";
import { stateTone } from "./words";

/** The props of `CourseBrowser`. */
export interface CourseBrowserProps {
  readonly campus: string;
  readonly top: readonly Category[];
  /** The category chosen, with the categories above it, top first. */
  readonly trail: readonly Category[];
  readonly courses: readonly Course[] | null;
}

/** A category of the tree and, once opened, the ones under it. */
function Node({ campus, category, chosen, open: initial, depth }: { readonly campus: string; readonly category: Category; readonly chosen: readonly string[]; readonly open: boolean; readonly depth: number }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(initial);
  const [children, setChildren] = useState<readonly Category[] | null>(null);
  const here = chosen[chosen.length - 1] === category.id;
  const load = async () => {
    if (children === null) setChildren(await browserApi<Category[]>(`/courses/campuses/${campus}/categories?parent=${category.id}`).catch(() => []));
  };
  useEffect(() => {
    if (open && children === null) void load();
  }, [open]);
  return (
    <li>
      <Box sx={{ display: "flex", alignItems: "center", pl: depth * 1.5 }}>
        <IconButton size="small" aria-label={open ? t("admin.courses.close", { name: category.name }) : t("admin.courses.open", { name: category.name })} aria-expanded={open} onClick={() => setOpen(!open)}>
          <Icon name={open ? "keyboard_arrow_down" : "chevron_right"} size="small" />
        </IconButton>
        <ButtonBase
          component={Link}
          href={`/admin/courses?category=${category.id}`}
          aria-current={here ? "page" : undefined}
          sx={{ flex: 1, justifyContent: "flex-start", gap: 1, px: 1, py: 0.75, borderRadius: 2, textAlign: "start", typography: "body2", fontWeight: here ? 700 : 500, bgcolor: here ? "color-mix(in srgb, var(--k-palette-primary-main) 12%, transparent)" : "transparent", "&:hover": { bgcolor: "action.hover" } }}
        >
          <Icon name="category" size="small" />
          <Box component="span" sx={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {category.name}
          </Box>
        </ButtonBase>
      </Box>
      {open && children?.length ? (
        <Box component="ul" sx={{ listStyle: "none", m: 0, p: 0 }}>
          {children.map((child) => (
            <Node key={child.id} campus={campus} category={child} chosen={chosen} open={chosen.includes(child.id) && chosen[chosen.length - 1] !== child.id} depth={depth + 1} />
          ))}
        </Box>
      ) : null}
    </li>
  );
}

/** `<CourseBrowser campus={campus} top={top} trail={trail} courses={courses} />` */
export function CourseBrowser({ campus, top, trail, courses }: CourseBrowserProps) {
  const { t, language } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const chosen = trail.map((c) => c.id);
  const current = trail[trail.length - 1] ?? null;
  const [creating, setCreating] = useState<"category" | "course" | null>(null);
  const [fields, setFields] = useState({ name: "", short: "", code: "" });
  const [busy, setBusy] = useState(false);

  async function create() {
    setBusy(true);
    try {
      if (creating === "category") {
        const made = await browserApi<Category>(`/courses/campuses/${campus}/categories`, {
          method: "POST",
          body: { parent: current?.id ?? null, name: fields.name.trim(), code: fields.code.trim() || null },
        });
        toast(t("admin.courses.category.made"), "success");
        router.push(`/admin/courses?category=${made.id}`);
      } else if (current) {
        const made = await browserApi<Course>(`/courses/categories/${current.id}/courses`, {
          method: "POST",
          body: { full_name: fields.name.trim(), short_name: fields.short.trim() || fields.name.trim(), code: fields.code.trim() || null },
        });
        toast(t("admin.courses.course.made"), "success");
        router.push(`/admin/courses/${made.id}`);
      }
      setCreating(null);
      setFields({ name: "", short: "", code: "" });
      router.refresh();
    } catch {
      toast(t("admin.courses.failed"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack spacing={2.5}>
      <PageHeader
        title={t("admin.page.courses")}
        lead={t("admin.page.courses.lead")}
        actions={
          <Stack direction="row" spacing={1}>
            <Button variant="outlined" icon="category" onClick={() => setCreating("category")}>
              {current ? t("admin.courses.category.new.under") : t("admin.courses.category.new")}
            </Button>
            {current ? (
              <Button variant="filled" icon="add" onClick={() => setCreating("course")}>
                {t("admin.courses.course.new")}
              </Button>
            ) : null}
          </Stack>
        }
      />
      <Box sx={{ display: "grid", gap: 2.5, gridTemplateColumns: { xs: "minmax(0, 1fr)", lg: "280px minmax(0, 1fr)" }, alignItems: "start" }}>
        <Box component="nav" aria-label={t("admin.courses.tree")} sx={{ border: 1, borderColor: "divider", borderRadius: 3, bgcolor: "background.paper", p: 1, maxBlockSize: { lg: "70vh" }, overflowY: "auto" }}>
          {top.length ? (
            <Box component="ul" sx={{ listStyle: "none", m: 0, p: 0 }}>
              {top.map((category) => (
                <Node key={category.id} campus={campus} category={category} chosen={chosen} open={chosen.includes(category.id) && chosen[chosen.length - 1] !== category.id} depth={0} />
              ))}
            </Box>
          ) : (
            <Typography variant="body2" color="text.secondary" sx={{ p: 2 }}>
              {t("admin.courses.tree.none")}
            </Typography>
          )}
        </Box>
        <Box sx={{ minInlineSize: 0 }}>
          {!current ? (
            <Typography color="text.secondary" sx={{ p: 3, border: 1, borderStyle: "dashed", borderColor: "divider", borderRadius: 3, textAlign: "center" }}>
              {t("admin.courses.choose")}
            </Typography>
          ) : (
            <Stack spacing={1.5}>
              <Stack direction="row" spacing={1} sx={{ alignItems: "baseline", flexWrap: "wrap" }}>
                <Typography variant="h3" component="h2">
                  {current.name}
                </Typography>
                {current.code ? (
                  <Typography variant="body2" color="text.secondary">
                    {current.code}
                  </Typography>
                ) : null}
                {current.visibility.mode === "hidden" ? <StatusBadge label={t("admin.courses.hidden")} tone="neutral" /> : null}
              </Stack>
              <Box sx={{ border: 1, borderColor: "divider", borderRadius: 3, overflowX: "auto", bgcolor: "background.paper" }}>
                <Table size="small" aria-label={t("admin.courses.of", { name: current.name })}>
                  <TableHead>
                    <TableRow>
                      <TableCell>{t("admin.courses.course")}</TableCell>
                      <TableCell>{t("admin.courses.state")}</TableCell>
                      <TableCell sx={{ display: { xs: "none", md: "table-cell" } }}>{t("admin.courses.dates")}</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {(courses ?? []).map((course) => (
                      <TableRow key={course.id} hover>
                        <TableCell>
                          <Typography component={Link} href={`/admin/courses/${course.id}`} variant="body2" sx={{ fontWeight: 600, color: "text.primary", textDecoration: "none", "&:hover": { textDecoration: "underline" } }}>
                            {course.full_name}
                          </Typography>
                          <Typography variant="caption" color="text.secondary" component="div">
                            {[course.short_name, course.code].filter(Boolean).join(" · ")}
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <StatusBadge label={t(`admin.courses.state.${course.state}`)} tone={stateTone(course.state)} />
                        </TableCell>
                        <TableCell sx={{ display: { xs: "none", md: "table-cell" } }}>
                          <Typography variant="body2" color="text.secondary">
                            {course.starts_at ? `${formatDay(language, course.starts_at)}${course.ends_at ? ` – ${formatDay(language, course.ends_at)}` : ""}` : t("admin.courses.nodates")}
                          </Typography>
                        </TableCell>
                      </TableRow>
                    ))}
                    {(courses ?? []).length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3}>
                          <Typography color="text.secondary" sx={{ py: 3, textAlign: "center" }}>
                            {t("admin.courses.none")}
                          </Typography>
                        </TableCell>
                      </TableRow>
                    ) : null}
                  </TableBody>
                </Table>
              </Box>
            </Stack>
          )}
        </Box>
      </Box>
      <Dialog open={creating !== null} onClose={() => setCreating(null)} fullWidth maxWidth="xs">
        <DialogTitle>{creating === "course" ? t("admin.courses.course.new") : current ? t("admin.courses.category.new.under") : t("admin.courses.category.new")}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            {current ? (
              <Typography variant="body2" color="text.secondary">
                {t("admin.courses.in", { name: current.name })}
              </Typography>
            ) : null}
            <TextField label={creating === "course" ? t("admin.courses.field.full_name") : t("admin.courses.field.name")} required autoFocus value={fields.name} onChange={(event) => setFields({ ...fields, name: event.target.value })} />
            {creating === "course" ? (
              <TextField label={t("admin.courses.field.short_name")} hint={t("admin.courses.field.short_name.hint")} width="medium" value={fields.short} onChange={(event) => setFields({ ...fields, short: event.target.value })} />
            ) : null}
            <TextField label={t("admin.courses.field.code")} hint={t("admin.courses.field.code.hint")} width="short" value={fields.code} onChange={(event) => setFields({ ...fields, code: event.target.value })} />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button variant="text" onClick={() => setCreating(null)}>
            {t("changes.discard")}
          </Button>
          <Button variant="filled" icon="add" busy={busy} disabled={!fields.name.trim()} onClick={() => void create()}>
            {t("admin.courses.create")}
          </Button>
        </DialogActions>
      </Dialog>
    </Stack>
  );
}
