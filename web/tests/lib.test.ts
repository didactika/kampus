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
// @package   web/tests
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

import { describe, expect, it } from "vitest";
import { escape, html } from "../src/lib/html";
import { setLanguage, t } from "../src/lib/i18n";
import { match } from "../src/lib/router";

describe("html", () => {
  it("escapes what is put in markup", () => {
    expect(escape(`<script>"x"&'y'</script>`)).not.toContain("<");
    expect(html`<p>${"<b>"}</p>`.value).toBe("<p>&#60;b&#62;</p>");
  });

  it("keeps markup that is already safe, and joins lists", () => {
    const inner = html`<i>${"a&b"}</i>`;
    expect(html`<p>${inner}</p>`.value).toBe("<p><i>a&#38;b</i></p>");
    expect(html`<ul>${["x", "<y>"].map((v) => html`<li>${v}</li>`)}</ul>`.value).toBe("<ul><li>x</li><li>&#60;y&#62;</li></ul>");
  });
});

describe("router", () => {
  it("matches paths with parameters", () => {
    expect(match("/courses/:id", "/courses/abc")).toEqual({ id: "abc" });
    expect(match("/courses/:id", "/courses")).toBeNull();
    expect(match("/", "/")).toEqual({});
    expect(match("/notifications", "/courses/x")).toBeNull();
  });
});

describe("i18n", () => {
  it("uses the language asked for and falls back to English", () => {
    expect(setLanguage("es-ES")).toBe("es");
    expect(t("nav.courses")).toBe("Cursos");
    expect(setLanguage("fr")).toBe("en");
    expect(t("course.untitled", { n: 3 })).toBe("Section 3");
    expect(t("no.such.key")).toBe("no.such.key");
  });
});
