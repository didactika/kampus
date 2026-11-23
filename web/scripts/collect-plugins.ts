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
 * Writes the registry of the widget plugins of the repository: every `crates/widget/plugins/<name>/web` package is
 * imported, so adding or removing a plugin is adding or removing its folder. Run before the client builds.
 */

import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const web = path.resolve(import.meta.dirname, "..");
const crates = path.resolve(web, "../crates");

/** The packages of the web parts of the widget plugins, by the name of their plugin. */
const plugins: { readonly name: string; readonly pkg: string }[] = [];
{
  const folder = path.join(crates, "widget", "plugins");
  for (const name of existsSync(folder) ? readdirSync(folder).sort() : []) {
    const manifest = path.join(folder, name, "web", "package.json");
    if (!existsSync(manifest)) throw new Error(`crates/widget/plugins/${name}: a widget plugin needs its web package`);
    const pkg = (JSON.parse(readFileSync(manifest, "utf8")) as { name: string }).name;
    const expected = `@kampus/widget-${name.replace(/_/g, "-")}`;
    if (pkg !== expected) throw new Error(`${manifest}: the package is ${expected}`);
    plugins.push({ name: `widget_${name}`, pkg });
  }
}

const header = readFileSync(path.join(web, "src/host/widget.ts"), "utf8").split("\n").slice(0, 33).join("\n");
writeFileSync(
  path.join(web, "src/plugins/widgets.generated.ts"),
  `${header}

/** The widget plugins of this build, by code. Written by \`scripts/collect-plugins.ts\`; do not edit. */

import type { WidgetPlugin } from "../host";
${plugins.map((p) => `import { widget as ${p.name} } from "${p.pkg}";`).join("\n")}

/** Every widget the client renders; a placement of any other is skipped, so a page never breaks. */
export const WIDGETS: Readonly<Record<string, WidgetPlugin>> = {
${plugins.map((p) => `  [${p.name}.code]: ${p.name},`).join("\n")}
};`,
);
// The packages alone, for the configuration of the build, which must not load the plugins themselves.
writeFileSync(path.join(web, "src/plugins/packages.generated.json"), `${JSON.stringify(plugins.map((p) => p.pkg), null, 2)}
`);
