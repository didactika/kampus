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
 * What the person asks of the catalogue (search, categories, states, order, layout, page and its size), read from and
 * written to the address of the page so a view can be shared and comes back, and applied to the courses.
 */

import type { CatalogueCourse } from "./catalogue";

/** The orders of the catalogue. */
export const SORTS = ["name", "starts", "code"] as const;

/** The layouts of the catalogue. */
export const VIEWS = ["grid", "list"] as const;

/** What the person asks of the catalogue. */
export interface CatalogueQuery {
  readonly q: string;
  readonly categories: readonly string[];
  readonly states: readonly string[];
  readonly sort: (typeof SORTS)[number];
  readonly view: (typeof VIEWS)[number];
  readonly page: number;
  readonly size: number;
}

/** The query of a fresh catalogue. */
export const FRESH: CatalogueQuery = { q: "", categories: [], states: [], sort: "name", view: "grid", page: 0, size: 12 };

/** One of a list of words, or the first. */
function oneOf<T extends string>(value: string | null | undefined, options: readonly T[]): T {
  return options.includes(value as T) ? (value as T) : options[0]!;
}

/** A whole number at least 0, or the fallback. */
function whole(value: string | null | undefined, fallback: number): number {
  if (value === null || value === undefined || value === "") return fallback;
  const n = Number(value);
  return Number.isInteger(n) && n >= 0 ? n : fallback;
}

/** The query in the parameters of an address; what they leave out is as in `start`. */
export function readQuery(params: URLSearchParams, start: CatalogueQuery = FRESH): CatalogueQuery {
  return {
    q: params.get("q") ?? start.q,
    categories: params.getAll("category"),
    states: params.getAll("state"),
    sort: params.has("sort") ? oneOf(params.get("sort"), SORTS) : start.sort,
    view: params.has("view") ? oneOf(params.get("view"), VIEWS) : start.view,
    page: whole(params.get("page"), 0),
    size: whole(params.get("size"), start.size),
  };
}

/** The parameters of an address for a query, leaving out what is as in `start`. */
export function writeQuery(query: CatalogueQuery, start: CatalogueQuery = FRESH): URLSearchParams {
  const params = new URLSearchParams();
  if (query.q) params.set("q", query.q);
  for (const id of query.categories) params.append("category", id);
  for (const state of query.states) params.append("state", state);
  if (query.sort !== start.sort) params.set("sort", query.sort);
  if (query.view !== start.view) params.set("view", query.view);
  if (query.page > 0) params.set("page", String(query.page));
  if (query.size !== start.size) params.set("size", String(query.size));
  return params;
}

/** A text without case or accents, for searching. */
function folded(text: string): string {
  return text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();
}

/** The courses that match a query, in its order. */
export function matching(courses: readonly CatalogueCourse[], query: CatalogueQuery, language: string): CatalogueCourse[] {
  const words = folded(query.q).split(/\s+/).filter(Boolean);
  const found = courses.filter((course) => {
    if (query.categories.length && !query.categories.includes(course.category)) return false;
    if (query.states.length && !query.states.includes(course.state)) return false;
    if (!words.length) return true;
    const text = folded([course.full_name, course.short_name, course.code, course.summary, course.categoryPath].filter(Boolean).join(" "));
    return words.every((word) => text.includes(word));
  });
  const collator = new Intl.Collator(language, { sensitivity: "base", numeric: true });
  const by: Record<CatalogueQuery["sort"], (a: CatalogueCourse, b: CatalogueCourse) => number> = {
    name: (a, b) => collator.compare(a.full_name, b.full_name),
    code: (a, b) => collator.compare(a.code ?? a.short_name, b.code ?? b.short_name),
    starts: (a, b) => (a.starts_at ?? "9999").localeCompare(b.starts_at ?? "9999") || collator.compare(a.full_name, b.full_name),
  };
  return found.sort(by[query.sort]);
}

/** How many courses have each value of a field, among some courses. */
export function counts(courses: readonly CatalogueCourse[], field: (course: CatalogueCourse) => string): Map<string, number> {
  const out = new Map<string, number>();
  for (const course of courses) out.set(field(course), (out.get(field(course)) ?? 0) + 1);
  return out;
}
