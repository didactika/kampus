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
// @package   tools/repo-check
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/**
 * The plugins of the repository keep to their contract: each plugin of a type lives in crates/<type>/plugins/<name>
 * with the folders its type asks for, and the web part of a widget imports only what a plugin may use, so removing a
 * plugin is removing its folder and nothing inside the client depends on it.
 */

import fs from 'node:fs';
import path from 'node:path';

/** What the web part of a plugin may import: itself, React, the design system, the API types and the host API. */
const ALLOWED = [/^\.\.?\//, /^react$/, /^react-dom$/, /^@mui\/material(\/|$)/, /^next\/link$/, /^next\/image$/, /^@kampus\/ui$/, /^@kampus\/sdk$/, /^@kampus\/web\/host(\/client)?$/];

/** The files and folders the web part of a widget plugin must have. */
const WIDGET_WEB = ['package.json', 'src/index.ts', 'strings/en.json'];

/** Every file under a directory. */
function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)]));
}

/**
 * Checks the plugins of the repository.
 *
 * @param {string} root Repository root.
 * @returns {{problems: string[], summary: string}} Problems found and a one-line summary.
 */
export function checkPlugins(root) {
  const problems = [];
  let count = 0;
  const crates = path.join(root, 'crates');
  for (const type of fs.readdirSync(crates)) {
    const folder = path.join(crates, type, 'plugins');
    if (type !== 'widget' || !fs.existsSync(folder)) continue;
    for (const name of fs.readdirSync(folder)) {
      count += 1;
      const plugin = path.join(folder, name);
      const where = path.relative(root, plugin).split(path.sep).join('/');
      for (const file of WIDGET_WEB) {
        if (!fs.existsSync(path.join(plugin, 'web', file))) problems.push(`${where}/web/${file}: a widget plugin needs it`);
      }
      const expected = `@kampus/widget-${name.replace(/_/g, '-')}`;
      const manifest = path.join(plugin, 'web', 'package.json');
      if (fs.existsSync(manifest) && JSON.parse(fs.readFileSync(manifest, 'utf8')).name !== expected) {
        problems.push(`${where}/web/package.json: the package of this plugin is ${expected}`);
      }
      const declaration = path.join(plugin, 'src', 'settings', 'customise.rs');
      if (!fs.existsSync(declaration) || !/pub const WIDGETS: &\[WidgetDeclaration\]/.test(fs.readFileSync(declaration, 'utf8'))) {
        problems.push(`${where}/src/settings/customise.rs: declares the widget in WIDGETS`);
      }
      const src = path.join(plugin, 'web', 'src');
      if (!fs.existsSync(src)) continue;
      for (const file of walk(src).filter((f) => /\.tsx?$/.test(f))) {
        const text = fs.readFileSync(file, 'utf8');
        const rel = path.relative(root, file).split(path.sep).join('/');
        const client = /^\s*["']use client["'];/m.test(text);
        for (const [statement, source] of text.matchAll(/^\s*(?:import|export)[^'"]*?from\s+["']([^"']+)["']/gm)) {
          if (!ALLOWED.some((rule) => rule.test(source))) problems.push(`${rel}: a plugin imports ${source}; it may use only @kampus/ui, @kampus/sdk and @kampus/web/host`);
          // A type is erased when it compiles, so a client component may read the types of the server host.
          if (client && source === '@kampus/web/host' && !/^\s*import type /.test(statement)) problems.push(`${rel}: a client component imports @kampus/web/host/client, never the server host`);
        }
      }
    }
  }
  return { problems, summary: `${count} widget plugin(s) keep to their contract` };
}
