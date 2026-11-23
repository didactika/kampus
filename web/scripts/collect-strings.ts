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
 * Writes `src/i18n/strings.generated.ts`: every text of the client in every language, from one JSON file per language
 * in each place that has texts. The shell keeps its texts in `src/i18n/strings/<language>.json`; the UI module of a
 * Kampus module or a plugin keeps its own in `crates/<module>/web/strings/<language>.json`, under keys that start
 * with the name of its module. Adding a language is adding those files; adding a module adds its folder. A text a
 * language lacks falls back to English. Run with `npm run strings -w @kampus/web`; the build runs it first.
 */

import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const web = path.resolve(import.meta.dirname, "..");
const repo = path.resolve(web, "..");

/** A folder of strings and the prefix every key of it must start with, if any. */
interface Source {
  readonly folder: string;
  readonly prefix: string | null;
}

const sources: Source[] = [{ folder: path.join(web, "src/i18n/strings"), prefix: null }];
const crates = path.join(repo, "crates");
for (const module of existsSync(crates) ? readdirSync(crates).sort() : []) {
  const folder = path.join(crates, module, "web/strings");
  if (existsSync(folder)) sources.push({ folder, prefix: `${module}.` });
  // The plugins of a type: crates/<type>/plugins/<name>/web/strings, under <type>_<name>.
  const plugins = path.join(crates, module, "plugins");
  for (const name of existsSync(plugins) ? readdirSync(plugins).sort() : []) {
    const strings = path.join(plugins, name, "web/strings");
    if (existsSync(strings)) sources.push({ folder: strings, prefix: `${module}_${name}.` });
  }
}

const bundles: Record<string, Record<string, string>> = {};
for (const { folder, prefix } of sources) {
  for (const file of readdirSync(folder).filter((f) => /^[a-z]{2,3}(-[A-Z]{2})?\.json$/.test(f)).sort()) {
    const language = file.replace(/\.json$/, "");
    const texts = JSON.parse(readFileSync(path.join(folder, file), "utf8")) as Record<string, unknown>;
    for (const [key, text] of Object.entries(texts)) {
      if (typeof text !== "string") throw new Error(`${folder}/${file}: ${key} is not a text`);
      if (prefix && !key.startsWith(prefix)) throw new Error(`${folder}/${file}: ${key} must start with ${prefix}`);
      const bundle = (bundles[language] ??= {});
      if (key in bundle) throw new Error(`${folder}/${file}: ${key} is defined twice`);
      bundle[key] = text;
    }
  }
}
if (!bundles["en"]) throw new Error("the shell has no English texts");

const header = readFileSync(path.join(web, "src/i18n/translator.ts"), "utf8").split("\n").slice(0, 33).join("\n");
const body = Object.entries(bundles)
  .sort(([a], [b]) => (a === "en" ? -1 : b === "en" ? 1 : a.localeCompare(b)))
  .map(([language, texts]) => {
    const lines = Object.entries(texts)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([key, text]) => `    ${JSON.stringify(key)}: ${JSON.stringify(text)},`);
    return `  ${JSON.stringify(language)}: {\n${lines.join("\n")}\n  },`;
  })
  .join("\n");
writeFileSync(
  path.join(web, "src/i18n/strings.generated.ts"),
  `${header}

/** Every text of the client by language, from the strings folders. Written by \`scripts/collect-strings.ts\`; do not edit. */
export const BUNDLES: Readonly<Record<string, Readonly<Record<string, string>>>> = {
${body}
};
`,
);
