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


/** What the person asks of the catalogue: read from and written to the address, and applied to the courses. */

import { describe, expect, it } from "vitest";

import type { CatalogueCourse } from "../../src/features/courses/catalogue";
import { FRESH, counts, matching, readQuery, writeQuery } from "../../src/features/courses/query";
import { subjectIcon } from "../../src/features/courses/subject";

/** A course of the catalogue. */
function course(id: string, fields: Partial<CatalogueCourse> = {}): CatalogueCourse {
  return ({
    id,
    category: "science",
    short_name: id.toUpperCase(),
    full_name: id,
    code: null,
    summary: null,
    language_code: null,
    starts_at: null,
    ends_at: null,
    state: "published",
    categoryName: "Science",
    categoryPath: "Science",
    ...fields,
  }) as CatalogueCourse;
}

const courses = [
  course("Física cuántica", { code: "PHY2", starts_at: "2027-02-01T00:00:00Z" }),
  course("Physics I", { code: "PHY1", starts_at: "2026-09-01T00:00:00Z" }),
  course("Art history", { category: "arts", categoryPath: "Arts", state: "draft" }),
];

describe("the query of the catalogue", () => {
  it("survives the address and leaves out what is as in a fresh catalogue", () => {
    const query = { ...FRESH, q: "phys", categories: ["science"], states: ["draft"], view: "list" as const, page: 2, size: 24 };
    expect(readQuery(writeQuery(query))).toEqual(query);
    expect(writeQuery(FRESH).toString()).toBe("");
    expect(readQuery(new URLSearchParams("")), "an address without parameters is a fresh catalogue").toEqual(FRESH);
    expect(readQuery(new URLSearchParams("sort=nope&page=-3&size=x"))).toEqual(FRESH);
  });

  it("searches every word, without case or accents, in names, codes, summaries and categories", () => {
    expect(matching(courses, { ...FRESH, q: "FISICA" }, "es").map((c) => c.id)).toEqual(["Física cuántica"]);
    expect(matching(courses, { ...FRESH, q: "phy 1" }, "en").map((c) => c.id)).toEqual(["Physics I"]);
    expect(matching(courses, { ...FRESH, q: "arts" }, "en").map((c) => c.id)).toEqual(["Art history"]);
  });

  it("filters by category and state, and orders by name, start or code", () => {
    expect(matching(courses, { ...FRESH, categories: ["arts"] }, "en")).toHaveLength(1);
    expect(matching(courses, { ...FRESH, states: ["published"] }, "en")).toHaveLength(2);
    expect(matching(courses, { ...FRESH, sort: "starts" }, "en").map((c) => c.id)).toEqual(["Physics I", "Física cuántica", "Art history"]);
    expect(matching(courses, { ...FRESH, sort: "code" }, "en")[0]!.id).toBe("Art history");
  });

  it("counts the courses of each value", () => {
    expect(Object.fromEntries(counts(courses, (c) => c.state))).toEqual({ published: 2, draft: 1 });
  });
});

describe("the subject of a course", () => {
  it("is told from its names and category", () => {
    expect(subjectIcon("Física cuántica")).toBe("science");
    expect(subjectIcon("Intro", "MAT101")).toBe("calculate");
    expect(subjectIcon("Something")).toBe("auto_stories");
  });
});
