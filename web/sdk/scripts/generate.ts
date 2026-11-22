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
// @package   web/sdk
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/**
 * Writes `src/schema.generated.ts`: the types of every route and every schema of the API, from `openapi.json`, the
 * document the server serves, which a test of the server keeps equal to it. Run with `npm run generate -w
 * @kampus/sdk` after updating the snapshot.
 */

import { readFileSync, writeFileSync } from "node:fs";

import openapiTS, { astToString } from "openapi-typescript";

const document = JSON.parse(readFileSync(new URL("../openapi.json", import.meta.url), "utf8")) as Record<string, unknown>;
const ast = await openapiTS(document as never, { alphabetize: true, exportType: true });
const header = readFileSync(new URL("../src/index.ts", import.meta.url), "utf8").split("\n").slice(0, 33).join("\n");
writeFileSync(
  new URL("../src/schema.generated.ts", import.meta.url),
  `${header}\n\n/** The routes and schemas of the API. Written by \`scripts/generate.ts\` from \`openapi.json\`; do not edit. */\n\n${astToString(ast)}`,
);
