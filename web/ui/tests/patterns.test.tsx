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
 * The patterns in a real browser: the pager offers only the sizes a list needs, the toolbar searches after a pause
 * and counts the filters on, filters toggle, and a page made of them has no accessibility violations.
 */

import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-react";
import { userEvent } from "vitest/browser";
import { useState } from "react";

import { CoverArt, FilterGroup, KampusThemeProvider, ListToolbar, Pager, ProgressRing, ViewToggle, pageSize, pageSizes } from "../src";

const labels = {
  perPage: "Items per page",
  all: "All",
  previous: "Previous page",
  next: "Next page",
  range: (from: number, to: number, total: number) => `${from}–${to} of ${total}`,
};

describe("the pager", () => {
  it("offers 12, 24, 48 and 96 only below the size of the list, and All up to 500", () => {
    expect(pageSizes(20)).toEqual([12, 0]);
    expect(pageSizes(46)).toEqual([12, 24, 0]);
    expect(pageSizes(5)).toEqual([12, 0]);
    expect(pageSizes(900)).toEqual([12, 24, 48, 96]);
    expect(pageSize(25, 100), "a size not offered falls back to 12, never to the whole list").toBe(12);
    expect(pageSize(0, 100), "the whole list only on purpose").toBe(0);
    expect(pageSizes(36, 8)).toEqual([8, 12, 24, 0]);
    expect(pageSize(8, 36, 8), "a preferred size is offered").toBe(8);
    expect(pageSize(10000, 900)).toBe(12);
    expect(pageSize(24, 46)).toBe(24);
  });

  it("moves between pages and says the range", async () => {
    function Page() {
      const [page, setPage] = useState(0);
      return <Pager total={62} page={page} size={12} onPage={setPage} onSize={() => {}} labels={labels} />;
    }
    const screen = await render(
      <KampusThemeProvider>
        <Page />
      </KampusThemeProvider>,
    );
    await expect.element(screen.getByText("1–12 of 62")).toBeVisible();
    expect((screen.getByRole("button", { name: "Previous page" }).element() as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(screen.getByRole("button", { name: "Next page" }));
    await expect.element(screen.getByText("13–24 of 62")).toBeVisible();
  });
});

describe("the list toolbar", () => {
  it("searches after the person stops typing and shows how many filters are on", async () => {
    const searched: string[] = [];
    const screen = await render(
      <KampusThemeProvider>
        <ListToolbar
          search=""
          onSearch={(text) => searched.push(text)}
          searchLabel="Search courses"
          activeFilters={2}
          filters={<FilterGroup title="State" values={[{ value: "draft", label: "Draft", count: 3 }]} selected={[]} onChange={() => {}} />}
        />
      </KampusThemeProvider>,
    );
    await userEvent.fill(screen.getByRole("searchbox", { name: "Search courses" }), "phys");
    expect(searched).toEqual([]);
    await expect.poll(() => searched).toEqual(["phys"]);
    await expect.element(screen.getByText("2")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: "Filters" }));
    await expect.element(screen.getByRole("dialog", { name: "Filters" })).toBeVisible();
  });

  it("turns filter values on and off", async () => {
    function Page() {
      const [on, setOn] = useState<string[]>([]);
      return <FilterGroup title="State" values={[{ value: "draft", label: "Draft", count: 3 }, { value: "published", label: "Published" }]} selected={on} onChange={setOn} />;
    }
    const screen = await render(
      <KampusThemeProvider>
        <Page />
      </KampusThemeProvider>,
    );
    const draft = screen.getByRole("button", { name: "Draft · 3" });
    await userEvent.click(draft);
    expect(draft.element().getAttribute("aria-pressed")).toBe("true");
    await userEvent.click(draft);
    expect(draft.element().getAttribute("aria-pressed")).toBe("false");
  });
});

describe("accessibility of the patterns", () => {
  for (const scheme of ["light", "dark"] as const) {
    it(`has no WCAG 2.2 AA violations in the ${scheme} scheme`, async () => {
      document.documentElement.dataset["theme"] = scheme;
      document.documentElement.lang = "en";
      document.title = "Patterns";
      await render(
        <KampusThemeProvider>
          <main>
            <h1>Patterns</h1>
            <ListToolbar
              search=""
              onSearch={() => {}}
              searchLabel="Search courses"
              filters={<span />}
              view={<ViewToggle label="View" value="grid" onChange={() => {}} options={[{ value: "grid", label: "Cards", icon: "grid_view" }, { value: "list", label: "List", icon: "view_list" }]} />}
            />
            <FilterGroup title="State" values={[{ value: "draft", label: "Draft", count: 3 }]} selected={["draft"]} onChange={() => {}} />
            <CoverArt seed="physics" icon="science" />
            <ProgressRing value={40} label="Course progress" />
            <Pager total={62} page={0} size={12} onPage={() => {}} onSize={() => {}} labels={labels} />
          </main>
        </KampusThemeProvider>,
      );
      const result = await axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"] } });
      expect(result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`)).toEqual([]);
      delete document.documentElement.dataset["theme"];
    });
  }
});
