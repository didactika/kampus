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


/** What a signed-in person does in the campus: the home, the catalogue, a course, the notifications and the account. */

import { expect, test } from "@playwright/test";

import { t } from "./support";

test("the home greets the person and offers to arrange it to whoever may", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText(t("home.greeting", { name: "" }).replace(/,\s*$/, ""));
  await expect(page.getByRole("heading", { name: t("catalogue.title") })).toBeVisible();
  const customise = page.getByRole("button", { name: t("dashboard.customise") });
  if (await customise.isVisible()) {
    await customise.click();
    await expect(page.getByText(t("dashboard.editing"))).toBeVisible();
    await page.getByRole("button", { name: t("dashboard.cancel") }).click();
    await expect(page.getByText(t("dashboard.editing"))).toBeHidden();
  }
});

test("the catalogue searches, keeps the search in the address and shows a list", async ({ page }) => {
  await page.goto("/courses");
  const counter = page.getByText(/^\d+ courses?$/);
  const before = Number((await counter.textContent())!.split(" ")[0]);
  await page.getByRole("searchbox", { name: t("catalogue.search") }).fill("zzzz-nothing");
  await expect(page.getByText(t("catalogue.nomatch"))).toBeVisible();
  await expect(page).toHaveURL(/q=zzzz-nothing/);
  await page.getByRole("button", { name: t("catalogue.reset") }).click();
  await expect(counter).toHaveText(t("catalogue.found", { n: before }));
  await page.getByRole("button", { name: t("view.list") }).click();
  await expect(page).toHaveURL(/view=list/);
});

test("a course opens from the catalogue with its content", async ({ page }) => {
  await page.goto("/courses");
  const first = page.locator('a[href^="/courses/"]').first();
  const name = (await first.getByRole("heading").textContent())!.trim();
  await first.click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(name);
  await expect(page.getByRole("region", { name: t("course.tab.content") })).toBeVisible();
});

test("the notifications switch between all and unread, and lead to their preferences", async ({ page }) => {
  await page.goto("/notifications");
  await expect(page.getByRole("heading", { level: 1, name: t("notifications.title") })).toBeVisible();
  await page.getByRole("tab", { name: new RegExp(t("notifications.tab.unread")) }).click();
  await expect(page).toHaveURL(/unread=1/);
  await page.getByRole("link", { name: t("notifications.preferences") }).click();
  await expect(page.getByRole("heading", { level: 1, name: t("prefs.title") })).toBeVisible();
});

test("the account shows the profile and the security of the person", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: t("nav.account") }).click();
  await page.getByRole("menuitem", { name: t("account.title") }).click();
  await expect(page).toHaveURL(/\/account$/);
  await expect(page.getByLabel(t("profile.given"))).toBeVisible();
  await page.getByRole("tab", { name: t("account.tab.security") }).click();
  await expect(page.getByRole("heading", { name: t("sessions.title") })).toBeVisible();
});
