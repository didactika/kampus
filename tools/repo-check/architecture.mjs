// This file is part of Kampus - Knowledge Academic Modular Platform for Universal Study.
//
// Copyright (c) 2026 Didactika. All rights reserved.
// Proprietary and confidential: see the LICENSE file at the root of the repository.
// SPDX-License-Identifier: LicenseRef-Didactika-Proprietary
//
// @package   tools/repo-check
// @author    hector-ae21
// @since     0.1.0

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
