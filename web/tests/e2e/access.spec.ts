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


/** The pages before signing in: recovering the access, activating an account, and the rules of a new password. */

import { expect, test } from "@playwright/test";

import { t } from "./support";

test.use({ storageState: { cookies: [], origins: [] } });

test("the sign-in leads to recovering the access, which never tells whether the email has an account", async ({ page }) => {
  await page.goto("/sign-in");
  await page.getByRole("link", { name: t("signin.forgot") }).click();
  await expect(page).toHaveURL(/\/recovery$/);
  await page.getByRole("textbox", { name: t("recovery.email") }).fill("nobody@example.org");
  await page.getByRole("button", { name: t("recovery.submit") }).click();
  await expect(page.getByText(t("recovery.sent"))).toBeVisible();
  await page.getByRole("button", { name: t("auth.signin") }).click();
  await expect(page).toHaveURL(/\/sign-in$/);
});

test("a link without its token says so", async ({ page }) => {
  for (const path of ["/recovery/complete", "/invitation", "/sign-up/confirm", "/email/confirm"]) {
    await page.goto(path);
    await expect(page.getByText(t("link.invalid"))).toBeVisible();
  }
});

test("a new password never holds spaces, and both copies must match", async ({ page }) => {
  await page.goto("/invitation#token=not-a-real-token");
  await expect(page).toHaveURL(/\/invitation$/);
  const password = page.getByRole("textbox", { name: t("password.new"), exact: true });
  await password.fill("a long and quiet river");
  await expect(page.getByText(t("password.rule.whitespace"))).toBeVisible();
  await password.fill("a-long-and-quiet-river");
  await expect(page.getByText(t("password.rule.whitespace"))).toBeHidden();
  await page.getByRole("textbox", { name: t("password.confirm"), exact: true }).fill("a-long-and-quiet-rivers");
  await page.getByRole("button", { name: t("password.store") }).click();
  await expect(page.getByText(t("password.mismatch"))).toBeVisible();
  await page.getByRole("textbox", { name: t("password.confirm"), exact: true }).fill("a-long-and-quiet-river");
  await page.getByRole("button", { name: t("password.store") }).click();
  await expect(page.getByRole("form", { name: t("activate.title") }).getByRole("alert")).toHaveText(t("link.invalid"));
});
