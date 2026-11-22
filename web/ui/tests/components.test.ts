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
 * The components in a real browser: they take part in forms, follow the keyboard patterns of WAI-ARIA, move focus as
 * a dialog should, announce results, and a page made of them has no accessibility violations.
 */

import axe from "axe-core";
import { userEvent } from "vitest/browser";
import { afterEach, beforeAll, describe, expect, it } from "vitest";

import dark from "../tokens/dark.tokens.json";
import light from "../tokens/light.tokens.json";
import reference from "../tokens/reference.tokens.json";
import system from "../tokens/system.tokens.json";
import { KToasts, overrideIcons, stylesheet, toast, type KButton, type KDialog, type KInput, type KTabs, type TokenNode } from "../src/index.js";

beforeAll(() => {
  const style = document.createElement("style");
  style.textContent = stylesheet({
    reference: reference as TokenNode,
    system: system as TokenNode,
    light: light as TokenNode,
    dark: dark as TokenNode,
  });
  document.head.append(style);
  document.documentElement.lang = "en";
  document.title = "Components";
});

afterEach(() => {
  document.body.replaceChildren();
});

/** Puts markup in the page and waits for its elements to render. */
async function mount(markup: string): Promise<HTMLElement> {
  const host = document.createElement("main");
  host.innerHTML = markup;
  document.body.append(host);
  await Promise.all([...host.querySelectorAll("*")].map((e) => (e as { updateComplete?: Promise<unknown> }).updateComplete));
  return host;
}

/** The accessibility violations of the page, with their rule and target. */
async function violations(): Promise<string[]> {
  const result = await axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"] } });
  return result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

describe("form controls", () => {
  it("submit their values with the form, block an invalid form and reset with it", async () => {
    const host = await mount(`<form>
      <k-input name="email" type="email" label="Email" required></k-input>
      <k-textarea name="about" label="About you"></k-textarea>
      <k-checkbox name="remember" label="Remember me"></k-checkbox>
      <k-select name="language" label="Language"></k-select>
      <k-button type="submit" variant="filled">Save</k-button>
    </form>`);
    const form = host.querySelector("form")!;
    const select = host.querySelector("k-select")!;
    select.options = [
      { value: "en", label: "English" },
      { value: "es", label: "Español" },
    ];
    select.value = "en";
    await select.updateComplete;
    const submitted: FormData[] = [];
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      submitted.push(new FormData(form));
    });

    await userEvent.click(host.querySelector("k-button")!);
    expect(submitted, "a required field that is empty blocks the form").toHaveLength(0);

    const email = host.querySelector<KInput>("k-input")!;
    await userEvent.click(email);
    await userEvent.keyboard("lea@example.org");
    await userEvent.click(host.querySelector("k-checkbox")!.shadowRoot!.querySelector("input")!);
    await userEvent.click(host.querySelector("k-button")!);
    expect(submitted).toHaveLength(1);
    expect(submitted[0]!.get("email")).toBe("lea@example.org");
    expect(submitted[0]!.get("remember")).toBe("on");
    expect(submitted[0]!.get("language")).toBe("en");

    form.reset();
    await email.updateComplete;
    expect(email.value).toBe("");
    expect(new FormData(form).get("remember")).toBeNull();
  });

  it("show an error from the server, tied to the control, until the person types", async () => {
    const host = await mount(`<k-input name="code" label="Code" hint="Six digits"></k-input>`);
    const input = host.querySelector<KInput>("k-input")!;
    input.error = "The code has expired";
    await input.updateComplete;
    const native = input.shadowRoot!.querySelector("input")!;
    expect(native.getAttribute("aria-invalid")).toBe("true");
    const described = native
      .getAttribute("aria-describedby")!
      .split(" ")
      .map((id) => input.shadowRoot!.getElementById(id)?.textContent);
    expect(described).toEqual(["Six digits", "The code has expired"]);
    expect(input.checkValidity()).toBe(false);
    await userEvent.click(input);
    await userEvent.keyboard("1");
    expect(input.error).toBe("");
    expect(input.checkValidity()).toBe(true);
  });

  it("a busy button cannot be pressed twice", async () => {
    const host = await mount(`<form><k-button type="submit" busy>Save</k-button></form>`);
    let sent = 0;
    host.querySelector("form")!.addEventListener("submit", (event) => {
      event.preventDefault();
      sent += 1;
    });
    await userEvent.click(host.querySelector("k-button")!);
    expect(sent).toBe(0);
    expect(host.querySelector("k-button")!.shadowRoot!.querySelector("button")!.getAttribute("aria-busy")).toBe("true");
  });
});

describe("a theme of a scope", () => {
  it("changes the look through custom properties alone", async () => {
    const host = await mount(`<k-button variant="filled">Save</k-button>`);
    const button = host.querySelector<KButton>("k-button")!.shadowRoot!.querySelector("button")!;
    expect(getComputedStyle(button).backgroundColor).toBe("rgb(15, 118, 110)");
    const theme = document.createElement("style");
    theme.textContent = ":root { --k-color-primary: #1d4ed8; --k-radius: 0px; --k-motion: 0; }";
    document.head.append(theme);
    expect(getComputedStyle(button).backgroundColor).toBe("rgb(29, 78, 216)");
    expect(getComputedStyle(button).borderTopLeftRadius).toBe("0px");
    theme.remove();
  });
});

describe("icons", () => {
  it("draw a Material Symbol by name, hidden unless labelled, and follow the overrides of a theme", async () => {
    const host = await mount(`<k-icon name="delete"></k-icon><k-icon name="edit" label="Edit"></k-icon>`);
    const [bin, pencil] = [...host.querySelectorAll("k-icon")].map((i) => i.shadowRoot!.querySelector("[part=icon]")!);
    expect(bin!.querySelector("svg")).not.toBeNull();
    expect(bin!.getAttribute("aria-hidden")).toBe("true");
    expect(pencil!.getAttribute("role")).toBe("img");
    expect(pencil!.getAttribute("aria-label")).toBe("Edit");

    overrideIcons({ delete: "/brand/bin.svg", edit: "javascript:alert(1)" });
    const icons = [...host.querySelectorAll("k-icon")];
    await Promise.all(icons.map((i) => i.updateComplete));
    const overridden = icons[0]!.shadowRoot!.querySelector("[part=icon]")!;
    expect(overridden.classList.contains("mask")).toBe(true);
    expect(overridden.getAttribute("style")).toContain('url("/brand/bin.svg")');
    expect(icons[1]!.shadowRoot!.querySelector("svg"), "only addresses of images are taken").not.toBeNull();

    overrideIcons();
    await icons[0]!.updateComplete;
    expect(icons[0]!.shadowRoot!.querySelector("svg")).not.toBeNull();
  });
});

describe("the dialog", () => {
  it("takes focus, closes with Escape and gives focus back", async () => {
    const host = await mount(`<button id="opener">Open</button>
      <k-dialog heading="Delete the course?"><p>It cannot be undone.</p><k-button slot="actions">Cancel</k-button></k-dialog>`);
    const opener = host.querySelector<HTMLButtonElement>("#opener")!;
    const dialog = host.querySelector<KDialog>("k-dialog")!;
    opener.focus();
    dialog.show();
    await dialog.updateComplete;
    const native = dialog.shadowRoot!.querySelector("dialog")!;
    expect(native.open).toBe(true);
    expect(document.activeElement).not.toBe(opener);
    const closed = new Promise((resolve) => dialog.addEventListener("k-close", (e) => resolve((e as CustomEvent).detail.reason)));
    await userEvent.keyboard("{Escape}");
    expect(await closed).toBe("escape");
    await dialog.updateComplete;
    expect(native.open).toBe(false);
    expect(document.activeElement).toBe(opener);
  });
});

describe("the tabs", () => {
  it("follow the arrows, Home and End, with only the selected tab in the tab order", async () => {
    const host = await mount(`<k-tabs label="Course">
      <k-tab-panel label="Content">Sections</k-tab-panel>
      <k-tab-panel label="Grades">Grades</k-tab-panel>
      <k-tab-panel label="Participants">People</k-tab-panel>
    </k-tabs>`);
    const tabs = host.querySelector<KTabs>("k-tabs")!;
    await tabs.updateComplete;
    await tabs.updateComplete;
    const buttons = () => [...tabs.shadowRoot!.querySelectorAll<HTMLButtonElement>("[role=tab]")];
    const panels = [...host.querySelectorAll("k-tab-panel")];
    expect(buttons().map((b) => b.tabIndex)).toEqual([0, -1, -1]);
    buttons()[0]!.focus();
    await userEvent.keyboard("{ArrowRight}");
    await tabs.updateComplete;
    expect(tabs.selected).toBe(1);
    expect(panels.map((p) => p.hidden)).toEqual([true, false, true]);
    await userEvent.keyboard("{End}");
    expect(tabs.selected).toBe(2);
    await userEvent.keyboard("{ArrowRight}");
    expect(tabs.selected, "the arrows wrap").toBe(0);
    await userEvent.keyboard("{Home}");
    await tabs.updateComplete;
    expect(buttons().map((b) => b.getAttribute("aria-selected"))).toEqual(["true", "false", "false"]);
  });
});

describe("toasts", () => {
  it("announce a result in a live region and leave", async () => {
    KToasts.lifetime = 50;
    toast("Saved", "success");
    const region = document.querySelector("k-toasts")!;
    await region.updateComplete;
    const live = region.shadowRoot!.querySelector("[aria-live]")!;
    expect(live.getAttribute("aria-live")).toBe("polite");
    expect(live.textContent).toContain("Saved");
    expect(live.querySelector("button")!.getAttribute("aria-label")).toBe("Dismiss");
    await new Promise((resolve) => setTimeout(resolve, 120));
    await region.updateComplete;
    expect(live.textContent).not.toContain("Saved");
  });
});

describe("accessibility", () => {
  for (const theme of ["light", "dark"]) {
    it(`a page of every component has no WCAG 2.2 AA violations in the ${theme} theme`, async () => {
      document.documentElement.dataset["theme"] = theme;
      document.body.style.background = "var(--k-color-surface)";
      document.body.style.color = "var(--k-color-on-surface)";
      await mount(`<h1>Components</h1>
        <form>
          <k-input name="name" label="Name" hint="As it appears on your certificate" required></k-input>
          <k-textarea name="about" label="About you"></k-textarea>
          <k-checkbox name="remember" label="Remember me"></k-checkbox>
          <k-select name="language" label="Language"></k-select>
          <k-button type="submit" variant="filled">Save</k-button>
          <k-button variant="tonal">Save a draft</k-button>
          <k-button variant="elevated">Preview</k-button>
          <k-button variant="outlined">Cancel</k-button>
          <k-button variant="filled" tone="danger">Delete</k-button>
          <k-button variant="outlined" tone="success">Start</k-button>
          <k-button variant="text">More</k-button>
          <k-button variant="filled" disabled>Unavailable</k-button>
        </form>
        <k-alert tone="danger" heading="Not saved">The name is required.</k-alert>
        <k-alert tone="warning">Some files are larger than the limit.</k-alert>
        <k-alert tone="success">Saved.</k-alert>
        <k-alert tone="info">Enrolments close on Friday.</k-alert>
        <k-card heading="Physics" subhead="Twelve sections"><p>First term. <k-badge tone="success">Done</k-badge> <k-badge>12</k-badge></p><k-button slot="actions" variant="text">Open</k-button></k-card>
        <k-card variant="elevated" heading="Chemistry"></k-card>
        <k-card variant="filled" heading="Biology"></k-card>
        <k-spinner label="Loading courses"></k-spinner>
        <p><k-icon name="info"></k-icon> <k-icon name="delete" label="Delete"></k-icon></p>
        <k-tabs label="Course"><k-tab-panel label="Content">Sections</k-tab-panel><k-tab-panel label="Grades">Grades</k-tab-panel></k-tabs>`);
      const select = document.querySelector("k-select")!;
      select.options = [{ value: "en", label: "English" }];
      await select.updateComplete;
      toast("Saved", "success");
      await document.querySelector("k-toasts")!.updateComplete;
      expect(await violations()).toEqual([]);
      delete document.documentElement.dataset["theme"];
    });
  }
});
