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


/** The language of a request, its direction, its messages and its formats. */

import { describe, expect, it } from "vitest";

import { MESSAGES } from "../../src/i18n/messages";
import { directionOf, formatNumber, languageOf, translator } from "../../src/i18n/translator";

describe("the language of a request", () => {
  it("is the first the shell speaks, English otherwise", () => {
    expect(languageOf("fr-FR,es;q=0.8,en;q=0.5")).toBe("es");
    expect(languageOf("de-DE")).toBe("en");
    expect(languageOf(null)).toBe("en");
  });

  it("gives the direction of the language", () => {
    expect(directionOf("es")).toBe("ltr");
    expect(directionOf("ar")).toBe("rtl");
  });
});

describe("messages", () => {
  it("fill their placeholders and fall back to English, then to the key", () => {
    const t = translator("es");
    expect(t("notifications.unread", { n: 3 })).toBe("3 sin leer");
    expect(t("no.such.key")).toBe("no.such.key");
  });

  it("are the same keys in every language", () => {
    const english = Object.keys(MESSAGES["en"]!).sort();
    for (const [language, bundle] of Object.entries(MESSAGES)) {
      expect(Object.keys(bundle).sort(), language).toEqual(english);
    }
  });

  it("format numbers in the language", () => {
    expect(formatNumber("es", "1234.5")).toBe("1234,5");
  });
});
