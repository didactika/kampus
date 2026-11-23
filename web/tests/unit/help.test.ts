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


/** The help link of each screen leads to its guide, in the language of the reader. */

import { describe, expect, it } from "vitest";

import { helpAddress, helpTopic } from "../../src/features/navigation/help";

describe("help of each screen", () => {
  it("leads every page to its guide", () => {
    expect(helpTopic("/")).toBe("learners/");
    expect(helpTopic("/courses")).toBe("learners/joining-a-course/");
    expect(helpTopic("/courses/0193")).toBe("learners/joining-a-course/");
    expect(helpTopic("/notifications/preferences")).toBe("learners/notifications/");
    expect(helpTopic("/account")).toBe("learners/your-account/");
    expect(helpTopic("/recovery/complete")).toBe("learners/your-account/");
  });

  it("writes the address in the language of the reader, and none without a handbook", () => {
    expect(helpAddress("https://docs.example.org", "en", "/account")).toBe("https://docs.example.org/learners/your-account/");
    expect(helpAddress("https://docs.example.org/", "es", "/account")).toBe("https://docs.example.org/es/learners/your-account/");
    expect(helpAddress(null, "en", "/account")).toBeNull();
  });
});
