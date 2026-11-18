// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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
 * Architecture rules of the Rust workspace, checked against `docs/03-design/architecture.md`.
 *
 * - Every crate in `crates/` is a module of the module table, or an adapter `kampus-<port>-<name>` of a port in it.
 * - A module depends only on modules listed before it in the table (dependency rule 1).
 * - Only adapters of a port and `kampus-server` depend on an adapter (dependency rule 5).
 */
import fs from 'node:fs';
import path from 'node:path';

/**
 * Lists the `kampus-*` crates among the normal dependencies of a manifest.
 *
 * @param {string} manifest Text of a `Cargo.toml`.
 * @returns {string[]} Crate names.
 */
function moduleDependencies(manifest) {
  const deps = new Set();
  let section = '';
  for (const line of manifest.split('\n')) {
    const header = line.match(/^\s*\[([^\]]+)\]\s*$/);
    if (header) {
      section = header[1].trim();
      continue;
    }
    const inline = section === 'dependencies' && line.match(/^\s*(kampus-[a-z0-9-]+)\s*[.=]/);
    if (inline) deps.add(inline[1]);
    const table = section.match(/^dependencies\.(kampus-[a-z0-9-]+)$/);
    if (table) deps.add(table[1]);
  }
  return [...deps];
}

/**
 * Checks the crates of the workspace.
 *
 * @param {string} root Repository root.
 * @returns {{problems: string[], summary: string}} Problems found and a one-line summary.
 */
export function checkArchitecture(root) {
  const architecture = fs.readFileSync(path.join(root, 'docs/03-design/architecture.md'), 'utf8');
  const order = [...architecture.matchAll(/^\| [^|]+ \| `(kampus-[a-z0-9-]+)` \|/gm)].map((m) => m[1]);
  const portOf = (name) => order.find((module) => name.startsWith(`${module}-`) && !order.includes(name));
  const cratesDir = path.join(root, 'crates');
  const crates = fs.existsSync(cratesDir)
    ? fs.readdirSync(cratesDir).filter((d) => fs.existsSync(path.join(cratesDir, d, 'Cargo.toml')))
    : [];
  const problems = [];
  for (const dir of crates) {
    const manifest = fs.readFileSync(path.join(cratesDir, dir, 'Cargo.toml'), 'utf8');
    const name = manifest.match(/^name\s*=\s*"([^"]+)"/m)?.[1] ?? dir;
    const port = portOf(name);
    if (!order.includes(name) && !port) {
      problems.push(`${name}: not in the module table of the architecture and not an adapter of a port`);
      continue;
    }
    for (const dep of moduleDependencies(manifest)) {
      const depPort = portOf(dep);
      if (depPort) {
        if (name !== 'kampus-server' && port !== depPort) problems.push(`${name}: depends on the adapter ${dep}; modules depend only on ports`);
      } else if (!order.includes(dep)) {
        problems.push(`${name}: depends on ${dep}, which is not in the module table`);
      } else if (!port && order.indexOf(dep) >= order.indexOf(name)) {
        problems.push(`${name}: depends on ${dep}, which is not listed before it`);
      }
    }
  }
  return { problems, summary: `${crates.length} crate(s) follow the dependency rules` };
}
