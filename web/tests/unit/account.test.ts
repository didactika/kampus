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
/** How a person is named and a session is described. */

import { describe, expect, it } from "vitest";

import { deviceOf } from "../../src/features/account/device";
import { initialsOf, nameOf } from "../../src/features/account/names";
import type { Me, Profile } from "../../src/api/types";

const account = { username: "admin", email: "admin@example.org" } as Me;

describe("names", () => {
  it("prefer the chosen name, then the given and family names, then the username", () => {
    expect(nameOf({ display_name: "Dr. Ada" } as Profile, account)).toBe("Dr. Ada");
    expect(nameOf({ given_name: "Ada", family_name: "Lovelace" } as Profile, account)).toBe("Ada Lovelace");
    expect(nameOf(null, account)).toBe("admin");
  });

  it("give two initials for two words and one for one", () => {
    expect(initialsOf({ given_name: "Ada", family_name: "Lovelace" } as Profile, account)).toBe("AL");
    expect(initialsOf({ given_name: "Ada" } as Profile, account)).toBe("A");
  });
});

describe("devices", () => {
  it("tell the browser and system of a session, or the program", () => {
    expect(deviceOf("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0 Safari/537.36")).toEqual({ browser: "Chrome", system: "Linux" });
    expect(deviceOf("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/157.0 Safari/537.36 Edg/157.0")).toEqual({ browser: "Edge", system: "Windows" });
    expect(deviceOf("curl/8.17.0")).toEqual({ browser: "curl 8.17.0", system: null });
    expect(deviceOf(null)).toBeNull();
  });
});
