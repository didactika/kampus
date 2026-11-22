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
 * The catalogue of courses: a toolbar to search, filter by category and state, order and choose cards or a list;
 * the courses that match; and the pager. What the person asks stays in the address of the page.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import Grid from "@mui/material/Grid";
import NativeSelect from "@mui/material/NativeSelect";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useCallback, useEffect, useId, useMemo, useState } from "react";

import { Button, FilterGroup, Icon, ListToolbar, Pager, ViewToggle, pageSize } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import type { Catalogue } from "./catalogue";
import { CourseCard, CourseRow } from "./CourseCard";
import { FRESH, SORTS, counts, matching, readQuery, writeQuery, type CatalogueQuery } from "./query";

/** The props of `CourseCatalogue`. */
export interface CourseCatalogueProps {
  /** The catalogue of the campus. */
  readonly catalogue: Catalogue;
  /** The parameters of the address the page was asked with. */
  readonly initial: string;
}

/** `<CourseCatalogue catalogue={…} initial="q=phys" />` */
export function CourseCatalogue({ catalogue, initial }: CourseCatalogueProps) {
  const { t, language } = useI18n();
  const [query, setQuery] = useState<CatalogueQuery>(() => readQuery(new URLSearchParams(initial)));
  const sortId = useId();

  const change = useCallback((next: Partial<CatalogueQuery>) => {
    setQuery((current) => ({ ...current, page: 0, ...next }));
  }, []);

  useEffect(() => {
    const params = writeQuery(query).toString();
    window.history.replaceState(null, "", params ? `?${params}` : window.location.pathname);
  }, [query]);

  const found = useMemo(() => matching(catalogue.courses, query, language), [catalogue.courses, query, language]);
  const size = pageSize(query.size, found.length);
  const per = size === 0 ? found.length : size;
  const pages = Math.max(1, Math.ceil(found.length / Math.max(per, 1)));
  const page = Math.min(query.page, pages - 1);
  const shown = found.slice(page * per, page * per + per);

  const byCategory = counts(catalogue.courses, (c) => c.category);
  const byState = counts(catalogue.courses, (c) => c.state);
  const active = query.categories.length + query.states.length;

  return (
    <Box component="section" aria-labelledby="catalogue">
      <Stack direction="row" sx={{ alignItems: "baseline", justifyContent: "space-between", mb: 2 }}>
        <Typography id="catalogue" variant="h2">
          {t("catalogue.title")}
        </Typography>
        <Typography variant="body2" color="text.secondary" aria-live="polite">
          {t("catalogue.found", { n: found.length })}
        </Typography>
      </Stack>
      <ListToolbar
        search={query.q}
        onSearch={(q) => change({ q })}
        searchLabel={t("catalogue.search")}
        filtersLabel={t("catalogue.filters")}
        resetLabel={t("catalogue.reset")}
        activeFilters={active}
        onReset={() => change({ categories: [], states: [] })}
        filters={
          <>
            <FilterGroup
              title={t("catalogue.category")}
              values={catalogue.categories.filter((c) => byCategory.has(c.id)).map((c) => ({ value: c.id, label: c.path, count: byCategory.get(c.id) }))}
              selected={query.categories}
              onChange={(categories) => change({ categories })}
            />
            {byState.size > 1 ? (
              <FilterGroup
                title={t("catalogue.state")}
                values={[...byState.keys()].map((s) => ({ value: s, label: t(`coursestate.${s}`), count: byState.get(s) }))}
                selected={query.states}
                onChange={(states) => change({ states })}
              />
            ) : null}
          </>
        }
        view={
          <>
            <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
              <Box component="label" htmlFor={sortId} sx={{ display: "flex", color: "text.secondary" }} title={t("catalogue.sort")}>
                <Icon name="sort" label={t("catalogue.sort")} />
              </Box>
              <NativeSelect
                id={sortId}
                value={query.sort}
                onChange={(event) => change({ sort: event.target.value as CatalogueQuery["sort"] })}
                disableUnderline
                sx={{ px: 1.5, borderRadius: 999, border: 1, borderColor: "divider", typography: "body2", bgcolor: "background.paper" }}
              >
                {SORTS.map((sort) => (
                  <option key={sort} value={sort}>
                    {t(`sort.${sort}`)}
                  </option>
                ))}
              </NativeSelect>
            </Stack>
            <ViewToggle
              label={t("catalogue.view")}
              value={query.view}
              onChange={(view) => setQuery((current) => ({ ...current, view: view as CatalogueQuery["view"] }))}
              options={[
                { value: "grid", label: t("view.grid"), icon: "grid_view" },
                { value: "list", label: t("view.list"), icon: "view_list" },
              ]}
            />
          </>
        }
      />
      {found.length === 0 ? (
        <Card>
          <EmptyState
            icon="search"
            message={catalogue.courses.length === 0 ? t("courses.empty") : t("catalogue.nomatch")}
            action={
              catalogue.courses.length > 0 ? (
                <Button variant="tonal" icon="filter_alt_off" onClick={() => setQuery(FRESH)}>
                  {t("catalogue.reset")}
                </Button>
              ) : null
            }
          />
        </Card>
      ) : query.view === "grid" ? (
        <Grid container spacing={2.5}>
          {shown.map((course) => (
            <Grid key={course.id} size={{ xs: 12, sm: 6, lg: 4, xl: 3 }}>
              <CourseCard course={course} />
            </Grid>
          ))}
        </Grid>
      ) : (
        <Card sx={{ p: 1 }}>
          <Stack divider={<Box sx={{ borderBottom: 1, borderColor: "divider", mx: 1.5 }} />}>
            {shown.map((course) => (
              <CourseRow key={course.id} course={course} />
            ))}
          </Stack>
        </Card>
      )}
      {found.length > 0 ? (
        <Pager
          total={found.length}
          page={page}
          size={size}
          onPage={(next) => setQuery((current) => ({ ...current, page: next }))}
          onSize={(next) => change({ size: next })}
          labels={{
            perPage: t("pager.perpage"),
            all: t("pager.all"),
            previous: t("pager.previous"),
            next: t("pager.next"),
            range: (from, to, total) => t("pager.range", { from, to, total }),
          }}
        />
      ) : null}
    </Box>
  );
}
