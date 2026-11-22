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
 * Writes `src/icons/symbols.generated.ts`: the Material Symbols the design system uses (rounded, weight 400) as SVG
 * text, so any bundler takes them as plain code. Run it with `npm run icons -w @kampus/ui` after changing the list.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";

/** The icons of the design system, by their name in Material Symbols. */
const NAMES = [
  "add",
  "archive",
  "arrow_back",
  "attach_file",
  "auto_stories",
  "brightness_auto",
  "calculate",
  "calendar_month",
  "category",
  "check",
  "check_circle",
  "chevron_left",
  "chevron_right",
  "close",
  "code",
  "content_copy",
  "dark_mode",
  "delete",
  "description",
  "done_all",
  "download",
  "draft",
  "edit",
  "error",
  "event",
  "filter_alt_off",
  "filter_list",
  "grid_view",
  "help",
  "history",
  "home",
  "inbox",
  "info",
  "keyboard_arrow_down",
  "keyboard_arrow_up",
  "language",
  "light_mode",
  "logout",
  "mail",
  "mark_email_read",
  "menu",
  "menu_book",
  "more_vert",
  "notifications",
  "open_in_new",
  "palette",
  "person",
  "public",
  "refresh",
  "schedule",
  "school",
  "science",
  "search",
  "settings",
  "sort",
  "translate",
  "trending_up",
  "tune",
  "upload",
  "view_list",
  "visibility",
  "visibility_off",
  "warning",
];

const require = createRequire(import.meta.url);
const header = readFileSync(new URL("../src/icons/registry.ts", import.meta.url), "utf8").split("\n").slice(0, 33).join("\n");
const entries = NAMES.map((name) => {
  const svg = readFileSync(require.resolve(`@material-symbols/svg-400/rounded/${name}.svg`), "utf8");
  const path = /<path d="([^"]+)"/.exec(svg)?.[1];
  if (!path) throw new Error(`no path in ${name}`);
  return `  ${name}: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="${path}"/></svg>',`;
});
writeFileSync(
  new URL("../src/icons/symbols.generated.ts", import.meta.url),
  `${header}

/** Material Symbols (Apache License 2.0), rounded, weight 400. Written by \`scripts/generate-icons.ts\`; do not edit. */
export const SYMBOLS: Readonly<Record<string, string>> = {
${entries.join("\n")}
};
`,
);
