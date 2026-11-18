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
 * Architecture rules of the Rust workspace, checked against `docs/03-design/architecture.md`.
 *
 * - Every crate in `crates/<folder>` is a module of the module table; every crate in
 *   `crates/<port>/adapters/<name>` is an adapter of that port, named `<port>-<name>`.
 * - A module depends only on modules listed before it in the table (dependency rule 1).
 * - Only `kampus-server` depends on adapters, which it registers (dependency rule 5).
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
  const cratesDir = path.join(root, 'crates');
  const read = (file) => fs.readFileSync(file, 'utf8');
  const nameOf = (manifest, fallback) => manifest.match(/^name\s*=\s*"([^"]+)"/m)?.[1] ?? fallback;
  const crates = [];
  if (fs.existsSync(cratesDir)) {
    for (const dir of fs.readdirSync(cratesDir)) {
      const manifest = path.join(cratesDir, dir, 'Cargo.toml');
      if (!fs.existsSync(manifest)) continue;
      const text = read(manifest);
      crates.push({ name: nameOf(text, dir), text, port: null });
      const adapters = path.join(cratesDir, dir, 'adapters');
      if (!fs.existsSync(adapters)) continue;
      for (const adapter of fs.readdirSync(adapters)) {
        const adapterManifest = path.join(adapters, adapter, 'Cargo.toml');
        if (fs.existsSync(adapterManifest)) {
          const adapterText = read(adapterManifest);
          crates.push({ name: nameOf(adapterText, adapter), text: adapterText, port: nameOf(text, dir) });
        }
      }
    }
  }
  /** Whether a crate name is an adapter of a port of the table, inside or outside the repository. */
  const isAdapter = (name) => crates.some((c) => c.port && c.name === name) || order.some((m) => name.startsWith(`${m}-`) && !order.includes(name));
  const problems = [];
  for (const { name, text, port } of crates) {
    if (port) {
      if (!order.includes(port)) problems.push(`${name}: adapter of ${port}, which is not in the module table`);
      if (!name.startsWith(`${port}-`)) problems.push(`${name}: an adapter of ${port} is named ${port}-<name>`);
    } else if (!order.includes(name)) {
      problems.push(`${name}: not in the module table of the architecture`);
      continue;
    }
    for (const dep of moduleDependencies(text)) {
      if (isAdapter(dep)) {
        if (name !== 'kampus-server') problems.push(`${name}: depends on the adapter ${dep}; only kampus-server chooses adapters`);
      } else if (!order.includes(dep)) {
        problems.push(`${name}: depends on ${dep}, which is not in the module table`);
      } else if (!port && order.indexOf(dep) >= order.indexOf(name)) {
        problems.push(`${name}: depends on ${dep}, which is not listed before it`);
      }
    }
  }
  return { problems, summary: `${crates.length} crate(s) follow the dependency rules` };
}
