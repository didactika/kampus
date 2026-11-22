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


/**
 * The catalogue of a campus as the person sees it: every category of its tree with its path, and every course in
 * them. It is read on the server once per request; the catalogue filters, sorts and pages it in the browser.
 */

import { cache } from "react";

import { serverApi } from "../../api/server";
import type { Category, Course } from "../../api/types";

/** A category with the names of the categories above it. */
export interface CatalogueCategory {
  readonly id: string;
  readonly name: string;
  /** Its names from the top, such as "Science / Physics". */
  readonly path: string;
}

/** A course with the category it is in. */
export interface CatalogueCourse extends Course {
  readonly categoryName: string;
  readonly categoryPath: string;
}

/** The catalogue of a campus. */
export interface Catalogue {
  readonly categories: CatalogueCategory[];
  readonly courses: CatalogueCourse[];
}

/** The deepest level of categories read. */
const DEPTH = 8;

/** The categories of a campus, depth first, each with its path. */
async function categoriesOf(campus: string): Promise<CatalogueCategory[]> {
  const found: CatalogueCategory[] = [];
  let level: { readonly id: string | null; readonly path: string }[] = [{ id: null, path: "" }];
  for (let depth = 0; depth < DEPTH && level.length > 0; depth += 1) {
    const children = await Promise.all(
      level.map(async (parent) => {
        const query = parent.id ? `?parent=${encodeURIComponent(parent.id)}` : "";
        const list = await serverApi<Category[]>(`/courses/campuses/${encodeURIComponent(campus)}/categories${query}`);
        return list.map((c) => ({ id: c.id, name: c.name, path: parent.path ? `${parent.path} / ${c.name}` : c.name }));
      }),
    );
    const next = children.flat();
    found.push(...next);
    level = next;
  }
  return found;
}

/** The catalogue of a campus, once per request. */
export const catalogueOf = cache(async (campus: string): Promise<Catalogue> => {
  const categories = await categoriesOf(campus);
  const lists = await Promise.all(
    categories.map(async (category) =>
      (await serverApi<Course[]>(`/courses/categories/${category.id}/courses`)).map((course) => ({
        ...course,
        categoryName: category.name,
        categoryPath: category.path,
      })),
    ),
  );
  return { categories, courses: lists.flat() };
});
