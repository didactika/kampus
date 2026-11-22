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


/**
 * The end-to-end tests of the web client: a real browser against a running Kampus, signing in once and reusing the
 * session. The address and the account come from the environment (`KAMPUS_E2E_URL`, `KAMPUS_E2E_USER`,
 * `KAMPUS_E2E_PASSWORD`), never from the code.
 */

import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "tests/e2e",
  outputDir: ".e2e-results",
  fullyParallel: true,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: process.env["KAMPUS_E2E_URL"] ?? "http://localhost:3100",
    locale: "en-GB",
    extraHTTPHeaders: { "Accept-Language": "en-GB,en;q=0.9" },
    trace: "retain-on-failure",
  },
  projects: [
    { name: "session", testMatch: /session\.setup\.ts/ },
    {
      name: "light",
      use: { ...devices["Desktop Chrome"], colorScheme: "light", storageState: ".e2e-results/session.json" },
      dependencies: ["session"],
      testIgnore: /session\.setup\.ts/,
    },
    {
      name: "dark",
      use: { ...devices["Desktop Chrome"], colorScheme: "dark", storageState: ".e2e-results/session.json" },
      dependencies: ["session"],
      testMatch: /accessibility\.spec\.ts/,
    },
  ],
});
