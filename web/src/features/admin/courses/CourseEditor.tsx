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
 * A course of the campus as its administration changes it: its names, code, summary, language and dates, whether it is
 * shown, saved from the floating bar; its state changed from the head; and a way to open it.
 */

import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import FormControlLabel from "@mui/material/FormControlLabel";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { Button, ChangesBar, SectionCard, SelectField, StatusBadge, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import { ApiError } from "../../../api/problem";
import type { Course } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { LANGUAGES } from "../../../i18n/translator";
import { PageHeader } from "../../shell/PageHeader";
import { useLeaveGuard } from "../../shell/LeaveGuard";
import { nextStates, stateTone } from "./words";

/** A date of a course as a date field holds it. */
function day(iso: string | null): string {
  return iso ? iso.slice(0, 10) : "";
}

/** The props of `CourseEditor`. */
export interface CourseEditorProps {
  readonly course: Course;
  /** The name of its category. */
  readonly category: string;
}

/** `<CourseEditor course={course} category="Sciences" />` */
export function CourseEditor({ course, category }: CourseEditorProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const start = useMemo(
    () => ({
      full_name: course.full_name,
      short_name: course.short_name,
      code: course.code ?? "",
      summary: course.summary ?? "",
      language_code: course.language_code ?? "",
      starts_at: day(course.starts_at),
      ends_at: day(course.ends_at),
      shown: course.visibility.mode !== "hidden",
    }),
    [course],
  );
  const [values, setValues] = useState(start);
  const [busy, setBusy] = useState(false);
  const dirty = JSON.stringify(values) !== JSON.stringify(start);
  const guard = useLeaveGuard(dirty);
  const set = <K extends keyof typeof start>(field: K, value: (typeof start)[K]) => setValues((current) => ({ ...current, [field]: value }));
  const text = (v: string) => (v.trim() === "" ? null : v.trim());
  const instant = (v: string) => (v ? `${v}T00:00:00Z` : null);

  async function save() {
    setBusy(true);
    try {
      await browserApi<Course>(`/courses/courses/${course.id}`, {
        method: "PUT",
        headers: { "If-Match": `"${course.version}"` },
        body: {
          full_name: values.full_name.trim(),
          short_name: values.short_name.trim(),
          code: text(values.code),
          summary: text(values.summary),
          layout: course.layout,
          language_code: text(values.language_code),
          starts_at: instant(values.starts_at),
          ends_at: instant(values.ends_at),
          visibility: values.shown ? { mode: "shown" } : { mode: "hidden" },
        },
      });
      toast(t("admin.courses.saved"), "success");
      router.refresh();
    } catch (error) {
      toast(error instanceof ApiError && error.status === 412 ? t("profile.conflict") : t("admin.courses.failed"), "danger");
    } finally {
      setBusy(false);
    }
  }

  async function move(to: string) {
    setBusy(true);
    try {
      await browserApi<Course>(`/courses/courses/${course.id}/state`, { method: "PUT", headers: { "If-Match": `"${course.version}"` }, body: { to } });
      toast(t(`admin.courses.state.done.${to}`), "success");
      router.refresh();
    } catch {
      toast(t("admin.courses.failed"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack spacing={3}>
      <PageHeader
        title={course.full_name}
        lead={t("admin.courses.in", { name: category })}
        back={{ href: `/admin/courses?category=${course.category}`, label: t("admin.page.courses") }}
        meta={<StatusBadge label={t(`admin.courses.state.${course.state}`)} tone={stateTone(course.state)} />}
        actions={
          <Stack direction="row" spacing={1} sx={{ flexWrap: "wrap" }}>
            <Button variant="outlined" icon="open_in_new" href={`/courses/${course.id}`}>
              {t("admin.courses.open.course")}
            </Button>
            {nextStates(course.state).map((to) => (
              <Button key={to} variant="tonal" busy={busy} onClick={() => void move(to)}>
                {t(`admin.courses.state.to.${to}`)}
              </Button>
            ))}
          </Stack>
        }
      />
      <SectionCard title={t("admin.courses.section.identity")}>
        <Stack spacing={2.5}>
          <TextField label={t("admin.courses.field.full_name")} width="long" required value={values.full_name} onChange={(event) => set("full_name", event.target.value)} />
          <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
            <TextField label={t("admin.courses.field.short_name")} width="medium" required value={values.short_name} onChange={(event) => set("short_name", event.target.value)} />
            <TextField label={t("admin.courses.field.code")} width="short" value={values.code} onChange={(event) => set("code", event.target.value)} />
          </Stack>
          <TextField label={t("admin.courses.field.summary")} multiline minRows={3} value={values.summary} onChange={(event) => set("summary", event.target.value)} />
        </Stack>
      </SectionCard>
      <SectionCard title={t("admin.courses.section.when")}>
        <Stack spacing={2.5}>
          <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
            <TextField label={t("admin.courses.field.starts_at")} type="date" width="short" value={values.starts_at} onChange={(event) => set("starts_at", event.target.value)} />
            <TextField label={t("admin.courses.field.ends_at")} type="date" width="short" value={values.ends_at} onChange={(event) => set("ends_at", event.target.value)} />
          </Stack>
          <SelectField
            label={t("admin.courses.field.language")}
            width="medium"
            value={values.language_code}
            onChange={(value) => set("language_code", value)}
            options={[{ value: "", label: t("admin.courses.field.language.any") }, ...LANGUAGES.map((l) => ({ value: l, label: new Intl.DisplayNames([l], { type: "language" }).of(l) ?? l }))]}
          />
          <FormControlLabel control={<Switch checked={values.shown} onChange={(event) => set("shown", event.target.checked)} />} label={t("admin.courses.field.shown")} />
        </Stack>
      </SectionCard>
      <ChangesBar open={dirty} label={course.full_name} message={t("changes.unsaved")} saveLabel={t("admin.courses.save")} discardLabel={t("changes.discard")} busy={busy} onSave={() => void save()} onDiscard={() => setValues(start)} />
      {guard}
    </Stack>
  );
}
