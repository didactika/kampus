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
 * The components in a real browser, with the MUI theme of a scope: they take part in forms, tie their hints and
 * errors to their fields, ask before acting, announce results, follow the theme of a scope and the overrides of
 * icons, and a page made of them has no accessibility violations in either scheme.
 */

import axe from "axe-core";
import { afterEach, describe, expect, it } from "vitest";
import { render } from "vitest-browser-react";
import { userEvent } from "vitest/browser";
import { useState, type ReactNode } from "react";

import {
  Button,
  ButtonGroup,
  ConfirmDialog,
  DIDACTIKA,
  Icon,
  KampusThemeProvider,
  Loading,
  SelectField,
  StatusBadge,
  TextField,
  ToastProvider,
  overrideIcons,
  useToast,
  type ScopeTheme,
} from "../src/index";

/** A tree in the theme of a scope. */
function themed(node: ReactNode, theme: ScopeTheme = DIDACTIKA) {
  document.documentElement.lang = "en";
  document.title = "Components";
  return render(
    <KampusThemeProvider theme={theme}>
      <main>{node}</main>
    </KampusThemeProvider>,
  );
}

afterEach(() => {
  delete document.documentElement.dataset["theme"];
});

/** The accessibility violations of the page, with their rule and target. */
async function violations(): Promise<string[]> {
  const result = await axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"] } });
  return result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

describe("form fields", () => {
  it("submit with the form, keep their label above them, and tie the hint and the error to the input", async () => {
    const submitted: FormData[] = [];
    const screen = await themed(
      <form
        onSubmit={(event) => {
          event.preventDefault();
          submitted.push(new FormData(event.currentTarget));
        }}
      >
        <TextField name="email" type="email" label="Email" hint="We send the confirmation here" required />
        <TextField name="code" label="Code" error="The code has expired" />
        <SelectField name="language" label="Language" options={[{ value: "en", label: "English" }, { value: "es", label: "Español" }]} />
        <Button type="submit" variant="filled">
          Save
        </Button>
      </form>,
    );
    const email = screen.getByRole("textbox", { name: /^Email/ });
    await userEvent.fill(email, "lea@example.org");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(submitted).toHaveLength(1);
    expect(submitted[0]!.get("email")).toBe("lea@example.org");
    expect(submitted[0]!.get("language")).toBe("en");

    const code = screen.getByLabelText("Code").element() as HTMLInputElement;
    expect(code.getAttribute("aria-invalid")).toBe("true");
    const described = code.getAttribute("aria-describedby")!.split(" ").map((id) => document.getElementById(id)?.textContent);
    expect(described).toEqual(["The code has expired"]);
    const hint = (email.element() as HTMLInputElement).getAttribute("aria-describedby");
    expect(document.getElementById(hint!)?.textContent).toBe("We send the confirmation here");
  });
});

describe("buttons", () => {
  it("cannot be pressed twice while busy", async () => {
    let pressed = 0;
    const screen = await themed(
      <Button variant="filled" busy onClick={() => (pressed += 1)}>
        Saving
      </Button>,
    );
    const button = screen.getByRole("button").element() as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    expect(button.getAttribute("aria-busy")).toBe("true");
    expect(pressed).toBe(0);
  });

  it("follow the theme of a scope", async () => {
    const theme: ScopeTheme = { ...DIDACTIKA, tokens: { radius: 0 }, light: { ...DIDACTIKA.light, primary: "#1d4ed8" } };
    const screen = await themed(<Button variant="filled">Save</Button>, theme);
    const style = getComputedStyle(screen.getByRole("button").element());
    expect(style.backgroundColor).toBe("rgb(29, 78, 216)");
    expect(style.borderTopLeftRadius).toBe("0px");
  });

  it("group under a name", async () => {
    const screen = await themed(
      <ButtonGroup label="View" connected>
        <Button>List</Button>
        <Button>Grid</Button>
      </ButtonGroup>,
    );
    expect(screen.getByRole("group", { name: "View" }).element()).toBeTruthy();
  });
});

describe("the confirm dialog", () => {
  it("asks before acting and closes without acting", async () => {
    let deleted = 0;
    function Page() {
      const [open, setOpen] = useState(true);
      return (
        <ConfirmDialog
          open={open}
          title="Delete the course?"
          confirmLabel="Delete"
          cancelLabel="Cancel"
          onConfirm={() => (deleted += 1)}
          onClose={() => setOpen(false)}
        >
          Its activities and grades are deleted too.
        </ConfirmDialog>
      );
    }
    const screen = await themed(<Page />);
    const dialog = screen.getByRole("dialog", { name: "Delete the course?" });
    expect(dialog.element()).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(deleted).toBe(1);
    await userEvent.keyboard("{Escape}");
    await expect.poll(() => document.querySelector("[role=dialog]")).toBeNull();
  });
});

describe("toasts", () => {
  it("announce a result politely and close with their button", async () => {
    function Page() {
      const toast = useToast();
      return <Button onClick={() => toast("Saved", "success")}>Save</Button>;
    }
    const screen = await themed(
      <ToastProvider closeLabel="Dismiss">
        <Page />
      </ToastProvider>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    const status = screen.getByRole("status").filter({ hasText: "Saved" });
    await expect.element(status).toBeVisible();
    expect(status.element().getAttribute("aria-live")).toBe("polite");
    await userEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    await expect.poll(() => document.body.textContent?.includes("Saved")).toBe(false);
  });
});

describe("icons", () => {
  it("draw a Material Symbol by name, hidden unless labelled, and follow the overrides of a theme", async () => {
    const screen = await themed(
      <p>
        <Icon name="delete" /> <Icon name="edit" label="Edit" />
      </p>,
    );
    const bin = document.querySelector("[data-icon=delete]")!;
    expect(bin.querySelector("svg")).not.toBeNull();
    expect(bin.getAttribute("aria-hidden")).toBe("true");
    expect(screen.getByRole("img", { name: "Edit" }).element()).toBeTruthy();

    overrideIcons({ delete: "/brand/bin.svg", edit: "javascript:alert(1)" });
    await expect.poll(() => document.querySelector("[data-icon=delete] svg")).toBeNull();
    expect(getComputedStyle(document.querySelector("[data-icon=delete]")!).maskImage).toContain("/brand/bin.svg");
    expect(document.querySelector("[data-icon=edit] svg"), "only addresses of images are taken").not.toBeNull();
    overrideIcons();
    await expect.poll(() => document.querySelector("[data-icon=delete] svg")).not.toBeNull();
  });
});

describe("accessibility", () => {
  for (const scheme of ["light", "dark"] as const) {
    it(`a page of every component has no WCAG 2.2 AA violations in the ${scheme} scheme`, async () => {
      document.documentElement.dataset["theme"] = scheme;
      await themed(
        <>
          <h1>Components</h1>
          <form>
            <TextField name="name" label="Name" hint="As it appears on your certificate" required />
            <TextField name="code" label="Code" error="The code has expired" />
            <SelectField name="language" label="Language" options={[{ value: "en", label: "English" }]} />
          </form>
          <ButtonGroup label="Actions">
            <Button variant="filled">Save</Button>
            <Button variant="tonal">Save a draft</Button>
            <Button variant="elevated">Preview</Button>
            <Button variant="outlined">Cancel</Button>
            <Button variant="text">More</Button>
            <Button variant="filled" tone="danger" icon="delete">
              Delete
            </Button>
            <Button variant="outlined" tone="success">
              Start
            </Button>
            <Button variant="filled" disabled>
              Unavailable
            </Button>
          </ButtonGroup>
          <p>
            <StatusBadge tone="success" label="Done" /> <StatusBadge tone="warning" label="Due tomorrow" />{" "}
            <StatusBadge tone="danger" label="Failed" /> <StatusBadge label="12" />
          </p>
          <Loading label="Loading grades" /> <Loading contained label="Saving" />
          <p>
            <Icon name="info" /> <Icon name="delete" label="Delete" />
          </p>
        </>,
      );
      expect(await violations()).toEqual([]);
    });
  }
});
