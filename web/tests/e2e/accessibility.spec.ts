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


/** Every page of the client meets WCAG 2.2 AA, in the scheme of its project (light and dark). */

import { test } from "@playwright/test";

import { expectAccessible, t } from "./support";

for (const path of ["/", "/courses", "/notifications", "/notifications/preferences", "/account", "/account?tab=security"]) {
  test(`${path} has no WCAG 2.2 AA violations`, async ({ page }) => {
    await page.goto(path);
    await page.waitForLoadState("networkidle");
    await expectAccessible(page);
  });
}

test("the home being arranged has no WCAG 2.2 AA violations", async ({ page }) => {
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  const mode = page.getByRole("switch", { name: t("editmode.label") });
  await mode.waitFor({ timeout: 5000 }).catch(() => undefined);
  test.skip(!(await mode.isVisible()), "the account cannot arrange the home");
  await mode.check();
  await page.getByRole("list", { name: t("grid.label") }).waitFor();
  await page.getByRole("group").first().focus();
  try {
    await expectAccessible(page);
  } finally {
    await mode.uncheck();
  }
});

test("a course has no WCAG 2.2 AA violations", async ({ page }) => {
  await page.goto("/courses");
  await page.locator('a[href^="/courses/"]').first().click();
  await page.waitForLoadState("networkidle");
  await expectAccessible(page);
});

for (const path of ["/sign-in", "/recovery", "/invitation#token=check", "/recovery/complete", "/email/confirm#token=check"]) {
  test(`${path} before signing in has no WCAG 2.2 AA violations`, async ({ browser }) => {
    const context = await browser.newContext({ storageState: { cookies: [], origins: [] }, colorScheme: test.info().project.use.colorScheme });
    const page = await context.newPage();
    await page.goto(`${test.info().project.use.baseURL}${path}`);
    await page.waitForLoadState("networkidle");
    await expectAccessible(page);
    await context.close();
  });
}
