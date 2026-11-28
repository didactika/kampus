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

/** The label of an item of a menu in the language of the reader. */

import { describe, expect, it } from "vitest";

import { labelOf } from "../../src/features/navigation/label";

describe("labelOf", () => {
  const item = { label: "Courses", labels: { es: "Cursos", "pt-BR": "Cursos (Brasil)" } };

  it("says the label of the language of the reader, or of its main language", () => {
    expect(labelOf(item, "es")).toBe("Cursos");
    expect(labelOf(item, "es-MX")).toBe("Cursos");
    expect(labelOf(item, "pt-BR")).toBe("Cursos (Brasil)");
  });

  it("says the label it was written with for a language that has none of its own", () => {
    expect(labelOf(item, "fr")).toBe("Courses");
    expect(labelOf({ label: "Help" }, "es")).toBe("Help");
  });
});
