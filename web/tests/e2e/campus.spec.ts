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

import { tAccount as t } from "./support";

test("the home greets the person and offers the edit mode to whoever may arrange it", async ({ page }) => {
  await page.goto("/");
  // The welcome may carry a heading the campus wrote, so the test asks only for a heading with text.
  await expect(page.getByRole("heading", { level: 1 }).first()).toHaveText(/\S/);
  await expect(page.getByRole("heading", { name: t("catalogue.title") })).toBeVisible();
  const mode = page.getByRole("switch", { name: t("editmode.label") });
  // The switch shows once the page has offered it, a moment after it renders.
  await mode.waitFor({ timeout: 5000 }).catch(() => undefined);
  if (await mode.isVisible()) {
    await mode.check();
    await expect(page.getByRole("button", { name: t("dashboard.reset") })).toBeVisible();
    await mode.uncheck();
    await expect(page.getByRole("button", { name: t("dashboard.reset") })).toBeHidden();
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

test("changing the password refuses a new one with spaces before sending it", async ({ page }) => {
  await page.goto("/account?tab=security");
  const form = page.getByRole("form", { name: t("password.title") });
  await form.getByRole("textbox", { name: t("password.new"), exact: true }).fill("with a space");
  await expect(form.getByText(t("password.rule.whitespace"))).toBeVisible();
  await form.getByRole("textbox", { name: t("password.new"), exact: true }).fill("without-a-space");
  await expect(form.getByText(t("password.rule.whitespace"))).toBeHidden();
  await expect(form.getByText(t("password.hint"))).toBeVisible();
});

test("the security of the account offers an authenticator app with its QR code and key", async ({ page }) => {
  await page.goto("/account?tab=security");
  await expect(page.getByRole("heading", { name: t("factors.title") })).toBeVisible();
  await page.getByRole("button", { name: t("factors.add.totp") }).click();
  const dialog = page.getByRole("dialog", { name: t("factors.add.totp") });
  await expect(dialog.getByRole("img", { name: t("totp.qr") })).toBeVisible();
  await expect(dialog.getByText(t("totp.secret"), { exact: true })).toBeVisible();
  await dialog.getByRole("button", { name: t("action.cancel") }).click();
  await expect(dialog).toBeHidden();
});

test("the profile offers its unsaved changes in a floating bar and asks before leaving them", async ({ page }) => {
  await page.goto("/account");
  const given = page.getByRole("textbox", { name: t("profile.given") });
  const before = await given.inputValue();
  await given.fill(`${before}-x`);
  const bar = page.getByRole("region", { name: t("account.tab.profile") });
  await expect(bar).toContainText(t("changes.unsaved"));
  await page.getByRole("navigation", { name: t("nav.primary") }).getByRole("link").first().click();
  const leave = page.getByRole("dialog", { name: t("leave.title") });
  await expect(leave).toBeVisible();
  await leave.getByRole("button", { name: t("leave.stay") }).click();
  await expect(page).toHaveURL(/\/account$/);
  await bar.getByRole("button", { name: t("changes.discard") }).click();
  await expect(given).toHaveValue(before);
  await expect(bar).toBeHidden();
});

test("a widget of the home moves and resizes with the keyboard and every change is saved at once", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("heading", { level: 1 }).first().waitFor();
  const mode = page.getByRole("switch", { name: t("editmode.label") });
  // The switch shows once the page has offered it, a moment after it renders.
  await mode.waitFor({ timeout: 5000 }).catch(() => undefined);
  test.skip(!(await mode.isVisible()), "the account cannot arrange the home");
  const me = (await (await page.request.get("/api/v1/identity/me")).json()) as { campus: string; scope: string };
  // The home is personal: what the person arranges is kept in their own scope, and the campus keeps its own.
  const layout = `/api/v1/customise/scopes/${me.scope}/layouts/customise.dashboard`;
  const campusLayout = `/api/v1/customise/scopes/${me.campus}/layouts/customise.dashboard`;
  const campusBefore = await (await page.request.get(campusLayout)).text();
  type Placed = { widget: string; x: number; y: number; w: number; h: number; settings: unknown };
  type Read = { placements: Placed[]; origin: { from: string; scope?: string } };
  const read = async () => (await (await page.request.get(layout)).json()) as Read;
  const welcomeOf = async () => (await read()).placements.find((p) => p.widget === "widget_welcome");
  const start = await read();
  const own = start.origin.from === "scope" && start.origin.scope === me.scope;
  const before = start.placements;
  const welcome = before.find((p) => p.widget === "widget_welcome");
  test.skip(!welcome, "the home has no welcome");
  await mode.check();
  try {
    const frame = page.getByRole("group", { name: new RegExp(`^${t("widget_welcome.name")}:`) }).first();
    await frame.focus();
    // Up or down moves it past the next widget, since widgets rise and a plain row would settle back: up when it is
    // below another, down otherwise, and then back.
    const [there, back] = welcome!.y > 0 ? ["ArrowUp", "ArrowDown"] : ["ArrowDown", "ArrowUp"];
    await page.keyboard.press(there);
    await expect.poll(async () => (await welcomeOf())?.y, { timeout: 5000 }).not.toBe(welcome!.y);
    expect(await (await page.request.get(campusLayout)).text(), "the home of the campus does not change").toBe(campusBefore);
    await frame.focus();
    await page.keyboard.press(back);
    await expect.poll(async () => (await welcomeOf())?.y, { timeout: 5000 }).toBe(welcome!.y);
    // Shift and an arrow resize it, within the sizes the widget declares: shorter when it can be, taller otherwise.
    const smallest = 3;
    await frame.focus();
    await page.keyboard.press(welcome!.h > smallest ? "Shift+ArrowUp" : "Shift+ArrowDown");
    await expect.poll(async () => (await welcomeOf())?.h, { timeout: 5000 }).not.toBe(welcome!.h);
  } finally {
    // Leaving the mode saves whatever is still pending, so nothing the page saves lands after the home is restored.
    if (await mode.isChecked()) await mode.uncheck();
    await page.waitForLoadState("networkidle");
    // The home of the person is left exactly as it was: their own arrangement, or the campus's when they had none.
    if (own) {
      await page.request.put(layout, { headers: { "x-kampus": "1" }, data: { placements: before.map(({ widget, x, y, w, h, settings }) => ({ widget, x, y, w, h, settings })) } });
    } else {
      await page.request.delete(layout, { headers: { "x-kampus": "1" } });
    }
  }
});

test("the settings of a widget are checked before they apply", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("heading", { level: 1 }).first().waitFor();
  const mode = page.getByRole("switch", { name: t("editmode.label") });
  await mode.waitFor({ timeout: 5000 }).catch(() => undefined);
  test.skip(!(await mode.isVisible()), "the account cannot arrange the home");
  await mode.check();
  const catalogue = page.getByRole("group", { name: new RegExp(`^${t("widget_catalogue.name")}:`) }).first();
  test.skip(!(await catalogue.isVisible()), "the home has no catalogue");
  await catalogue.hover();
  await catalogue.getByRole("button", { name: t("dashboard.settings") }).click();
  const panel = page.getByRole("dialog");
  const sizes = panel.getByRole("textbox", { name: t("setting.sizes") });
  await sizes.fill("5,10,XIII,20");
  await expect(panel.getByText(t("setting.error.sizes", { max: 500 }))).toBeVisible();
  // Applying with a wrong setting keeps the dialog open, showing what is wrong.
  await panel.getByRole("button", { name: t("dashboard.apply") }).click();
  await expect(panel).toBeVisible();
  await sizes.fill("5, 10, 200");
  await expect(panel.getByRole("button", { name: t("dashboard.apply") })).toBeEnabled();
  await panel.getByRole("button", { name: t("dashboard.cancel") }).first().click();
  await mode.uncheck();
});

test("a course being arranged keeps its content and summary: they move, but are never removed", async ({ page }) => {
  await page.goto("/courses");
  await page.locator('a[href^="/courses/"]').first().click();
  await page.getByRole("heading", { level: 1 }).first().waitFor();
  const mode = page.getByRole("switch", { name: t("editmode.label") });
  await mode.waitFor({ timeout: 5000 }).catch(() => undefined);
  test.skip(!(await mode.isVisible()), "the account cannot arrange the course");
  await mode.check();
  try {
    for (const widget of ["widget_course_content", "widget_course_summary"]) {
      const frame = page.getByRole("group", { name: new RegExp(`^${t(`${widget}.name`)}:`) });
      await expect(frame, `${widget} is on the page`).toHaveCount(1);
      await frame.hover();
      await expect(frame.getByRole("button", { name: t("dashboard.remove") })).toHaveCount(0);
      await expect(frame.getByRole("img", { name: t("grid.fixed") })).toBeVisible();
    }
  } finally {
    // Nothing was changed, so leaving the mode saves nothing.
    await mode.uncheck();
  }
});

test("a text of the home is written on the widget itself and kept per language", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("heading", { level: 1 }).first().waitFor();
  const mode = page.getByRole("switch", { name: t("editmode.label") });
  await mode.waitFor({ timeout: 5000 }).catch(() => undefined);
  test.skip(!(await mode.isVisible()), "the account cannot arrange the home");
  const me = (await (await page.request.get("/api/v1/identity/me")).json()) as { scope: string };
  const layout = `/api/v1/customise/scopes/${me.scope}/layouts/customise.dashboard`;
  type Placed = { widget: string; x: number; y: number; w: number; h: number; pinned: boolean; settings: Record<string, unknown> };
  const read = async () => (await (await page.request.get(layout)).json()) as { placements: Placed[]; origin: { from: string; scope?: string } };
  const start = await read();
  const own = start.origin.from === "scope" && start.origin.scope === me.scope;
  test.skip(!start.placements.some((p) => p.widget === "widget_text"), "the home has no text widget");
  await mode.check();
  try {
    const heading = page.getByRole("textbox", { name: t("editor.text", { setting: "heading" }) }).first();
    await heading.click();
    await heading.press("End");
    await heading.pressSequentially(" e2e");
    // The text is kept in the language it was written in, with the others it had.
    await expect
      .poll(async () => JSON.stringify((await read()).placements.find((p) => p.widget === "widget_text")?.settings["heading"]), { timeout: 5000 })
      .toContain("e2e");
  } finally {
    if (await mode.isChecked()) await mode.uncheck();
    await page.waitForLoadState("networkidle");
    if (own) {
      await page.request.put(layout, { headers: { "x-kampus": "1" }, data: { placements: start.placements.map(({ widget, x, y, w, h, pinned, settings }) => ({ widget, x, y, w, h, pinned, settings })) } });
    } else {
      await page.request.delete(layout, { headers: { "x-kampus": "1" } });
    }
  }
});

test("every page of the campus has its frame: the notifications, the language and the foot with the version", async ({ page }) => {
  await page.goto("/courses");
  await page.getByRole("button", { name: t("nav.account") }).click();
  await page.getByRole("menuitem", { name: new RegExp(t("nav.language.title")) }).click();
  await expect(page.getByRole("menu").last().getByRole("menuitem")).not.toHaveCount(0);
  await page.keyboard.press("Escape");
  await page.keyboard.press("Escape");
  await expect(page.getByRole("contentinfo")).toContainText(t("footer.version", { version: "0.1.0" }));
  await page.getByRole("banner").getByRole("link", { name: new RegExp(t("nav.notifications")) }).click();
  await expect(page.getByRole("heading", { level: 1, name: t("notifications.title") })).toBeVisible();
});

test("the administration is found from the bar, searched, and a setting is saved here and inherited again", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("banner").getByRole("link", { name: t("nav.admin") }).click();
  await expect(page.getByRole("heading", { level: 1, name: t("admin.title") })).toBeVisible();
  await page.getByRole("searchbox", { name: t("admin.search.all") }).fill("password");
  await page.getByRole("link", { name: /Shortest password/ }).click();
  await expect(page).toHaveURL(/\/admin\/settings\/identity#identity\.password_min_length$/);
  const row = page.getByRole("group", { name: "Shortest password" });
  const field = row.getByRole("spinbutton");
  // Only a setting that takes the default is changed, so taking the inherited value again leaves the campus as it was.
  const before = await field.inputValue();
  const defaulted = row.getByText(t("admin.default", { value: before }));
  test.skip(await defaulted.isVisible(), "the campus sets this value itself");
  try {
    await field.fill(String(Number(before) + 1));
    await expect(row.getByText(t("admin.unsaved"))).toBeVisible();
    await page.getByRole("button", { name: t("admin.save") }).click();
    await expect(page.getByText(t("admin.saved"))).toBeVisible();
    await expect(defaulted).toBeVisible();
    await row.getByRole("button", { name: t("admin.actions.of", { name: "Shortest password" }) }).click();
    await page.getByRole("menuitem", { name: t("admin.history") }).click();
    await expect(page.getByRole("dialog").getByText(t("admin.history.set", { value: String(Number(before) + 1) })).first()).toBeVisible();
    await page.getByRole("button", { name: t("admin.close") }).click();
    await row.getByRole("button", { name: t("admin.actions.of", { name: "Shortest password" }) }).click();
    await page.getByRole("menuitem", { name: t("admin.reset") }).click();
    await expect(defaulted).toBeHidden();
  } finally {
    // Whatever failed, the campus takes the inherited value again.
    await page.evaluate(async () => {
      const me = (await (await fetch("/api/v1/identity/me")).json()) as { campus: string };
      await fetch(`/api/v1/settings/scopes/${me.campus}/identity.password_min_length`, { method: "DELETE", headers: { "X-Kampus": "1" } });
    });
  }
});

test("the matrix of roles changes a value at once and the check explains why a person may", async ({ page }) => {
  await page.goto("/admin/roles/compare");
  await page.getByLabel(t("admin.roles.search")).fill("access.view_as");
  const cell = page.getByRole("button", { name: /^Mentor, access\.view_as: / });
  const before = (await cell.getAttribute("aria-label"))!.split(": ").pop()!;
  const choices = ["allow", "deny", "forbid", "none"] as const;
  const original = choices.find((c) => t(`access.value.${c}`) === before)!;
  const other = original === "deny" ? "allow" : "deny";
  try {
    await cell.click();
    await page.getByRole("menuitem", { name: t(`access.value.${other}`) }).click();
    await expect(page.getByText(t("admin.roles.saved"))).toBeVisible();
    await expect(cell).toHaveAccessibleName(`Mentor, access.view_as: ${t(`access.value.${other}`)}`);
  } finally {
    // Whatever failed, the role gives the permission what it gave before.
    await cell.click();
    await page.getByRole("menuitem", { name: t(`access.value.${original}`) }).click();
    await expect(cell).toHaveAccessibleName(`Mentor, access.view_as: ${before}`);
  }

  await page.getByRole("navigation", { name: t("admin.sections") }).getByRole("link", { name: t("admin.page.check") }).click();
  await page.getByLabel(t("admin.account.search")).fill("admin");
  await page.getByRole("listitem").getByRole("button").first().click();
  await page.getByRole("button", { name: t("admin.explain.check") }).click();
  await expect(page.getByRole("heading", { name: t("admin.explain.allowed") })).toBeVisible();
});

test("the accounts of the campus are found, chosen together and opened", async ({ page }) => {
  await page.goto("/admin/people");
  await expect(page.getByRole("heading", { level: 1, name: t("admin.page.people") })).toBeVisible();
  await page.getByLabel(t("admin.people.search")).fill("admin");
  const row = page.getByRole("row", { name: /admin/ }).first();
  await expect(row).toBeVisible();
  await row.getByRole("checkbox").check();
  await expect(page.getByRole("toolbar", { name: t("admin.people.bulk") })).toBeVisible();
  await page.getByRole("button", { name: t("admin.people.unchoose") }).click();
  await row.getByRole("link").click();
  await expect(page.getByRole("tab", { name: t("admin.people.tab.profile") })).toHaveAttribute("aria-selected", "true");
  await page.getByRole("tab", { name: t("admin.people.tab.roles") }).click();
  await expect(page.getByRole("heading", { name: t("admin.people.roles.held") })).toBeVisible();
});

test("the courses of the campus are browsed by category and a course opens with its fields", async ({ page }) => {
  await page.goto("/admin/courses");
  await expect(page.getByRole("heading", { level: 1, name: t("admin.page.courses") })).toBeVisible();
  const tree = page.getByRole("navigation", { name: t("admin.courses.tree") });
  await tree.getByRole("link").first().click();
  await expect(page).toHaveURL(/category=/);
  const course = page.locator("tbody a").first();
  const name = (await course.textContent())!.trim();
  await course.click();
  await expect(page.getByRole("heading", { level: 1, name })).toBeVisible();
  await expect(page.getByLabel(t("admin.courses.field.full_name"))).toHaveValue(name);
});

test("a role opens with its permissions by module, found by their names", async ({ page }) => {
  await page.goto("/admin/roles");
  await page.getByRole("link", { name: new RegExp(t("access.role.teacher")) }).first().click();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await page.getByLabel(t("admin.roles.search")).fill(t("access.review.permission"));
  const row = page.getByRole("group", { name: t("access.review.permission") });
  await expect(row).toBeVisible();
  await expect(row.getByRole("button", { pressed: true })).toHaveCount(1);
});
