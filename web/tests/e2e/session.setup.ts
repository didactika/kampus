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


/** Signs in once, refusing a wrong password first, and keeps the session for every other test. */

import { expect, test } from "@playwright/test";

import { account, t } from "./support";

test("signs in, after refusing a wrong password", async ({ page }) => {
  const { user, password } = account();
  await page.goto("/");
  await expect(page).toHaveURL(/\/sign-in$/);
  await page.getByRole("textbox", { name: t("signin.name") }).fill(user);
  await page.getByLabel(t("signin.password")).first().fill(`${password}-wrong`);
  await page.getByRole("button", { name: t("signin.submit") }).click();
  await expect(page.getByRole("form", { name: t("signin.title") }).getByRole("alert")).toHaveText(t("signin.failed"));
  await page.getByLabel(t("signin.password")).first().fill(password);
  await page.getByRole("button", { name: t("signin.submit") }).click();
  await expect(page).toHaveURL(/\/$/);
  await page.context().storageState({ path: ".e2e-results/session.json" });
});
