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
 * Writes the registries of the plugins of the repository with a web part, one per type: every
 * `crates/<type>/plugins/<name>/web` package is imported, so adding or removing a plugin is adding or removing its folder. Run before the client builds.
 */

import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const web = path.resolve(import.meta.dirname, "..");
const crates = path.resolve(web, "../crates");

/** A plugin type with a web part: what its plugins export, the contract of it and the registry the client reads. */
interface Kind {
  readonly type: string;
  readonly exported: string;
  readonly contract: string;
  readonly registry: string;
  readonly about: string;
}

const KINDS: readonly Kind[] = [
  { type: "widget", exported: "widget", contract: "WidgetPlugin", registry: "WIDGETS", about: "Every widget the client renders; a placement of any other is skipped, so a page never breaks." },
  { type: "outline", exported: "outline", contract: "OutlinePlugin", registry: "OUTLINES", about: "Every outline the client renders a course with; a course whose outline is not here gets the standard one." },
];

/** The packages of the web parts of the plugins of a type, by the name of their plugin. */
function pluginsOf(type: string): { readonly name: string; readonly pkg: string }[] {
  const folder = path.join(crates, type, "plugins");
  return (existsSync(folder) ? readdirSync(folder).sort() : []).map((name) => {
    const manifest = path.join(folder, name, "web", "package.json");
    if (!existsSync(manifest)) throw new Error(`crates/${type}/plugins/${name}: a ${type} plugin needs its web package`);
    const pkg = (JSON.parse(readFileSync(manifest, "utf8")) as { name: string }).name;
    const expected = `@kampus/${type}-${name.replace(/_/g, "-")}`;
    if (pkg !== expected) throw new Error(`${manifest}: the package is ${expected}`);
    return { name: `${type}_${name}`, pkg };
  });
}

const header = readFileSync(path.join(web, "src/host/widget.ts"), "utf8").split("\n").slice(0, 33).join("\n");
const packages: string[] = [];
for (const kind of KINDS) {
  const plugins = pluginsOf(kind.type);
  packages.push(...plugins.map((p) => p.pkg));
  writeFileSync(
    path.join(web, `src/plugins/${kind.type}s.generated.ts`),
    `${header}

/** The ${kind.type} plugins of this build, by code. Written by \`scripts/collect-plugins.ts\`; do not edit. */

import type { ${kind.contract} } from "../host";
${plugins.map((p) => `import { ${kind.exported} as ${p.name} } from "${p.pkg}";`).join("\n")}

/** ${kind.about} */
export const ${kind.registry}: Readonly<Record<string, ${kind.contract}>> = {
${plugins.map((p) => `  [${p.name}.code]: ${p.name},`).join("\n")}
};
`,
  );
}
// The packages alone, for the configuration of the build, which must not load the plugins themselves.
writeFileSync(path.join(web, "src/plugins/packages.generated.json"), `${JSON.stringify(packages, null, 2)}\n`);
