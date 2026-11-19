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
 * The adapter registry of the server. Every adapter in `crates/<port>/adapters/<name>`, and every
 * external adapter declared in `crates/server/adapters.toml`, becomes a dependency of `kampus-server` and an
 * entry of `crates/server/src/wiring/adapters.rs`. With `--fix` both are written; otherwise a stale registry is a
 * problem, so an adapter dropped into its folder is never forgotten.
 */
import fs from 'node:fs';
import path from 'node:path';
import { header } from './headers.mjs';

/** The marker lines of the generated block in the server manifest. */
const START = '# adapters:start';
const END = '# adapters:end';

/**
 * Reads the package name of a manifest.
 *
 * @param {string} file Path of a `Cargo.toml`.
 * @returns {string|null} The name.
 */
function packageName(file) {
  return fs.readFileSync(file, 'utf8').match(/^name\s*=\s*"([^"]+)"/m)?.[1] ?? null;
}

/**
 * Lists the adapters of the repository and the external ones.
 *
 * @param {string} root Repository root.
 * @returns {{port: string, crate: string, source: string}[]} Adapters sorted by port and crate.
 */
export function listAdapters(root) {
  const crates = path.join(root, 'crates');
  const adapters = [];
  if (!fs.existsSync(crates)) return adapters;
  for (const port of fs.readdirSync(crates)) {
    const dir = path.join(crates, port, 'adapters');
    if (!fs.existsSync(dir)) continue;
    for (const name of fs.readdirSync(dir)) {
      const manifest = path.join(dir, name, 'Cargo.toml');
      if (fs.existsSync(manifest)) adapters.push({ port, crate: packageName(manifest), source: `path = "../${port}/adapters/${name}"` });
    }
  }
  const external = path.join(crates, 'server', 'adapters.toml');
  if (fs.existsSync(external)) {
    for (const block of fs.readFileSync(external, 'utf8').split(/^\[\[adapter\]\]\s*$/m).slice(1)) {
      const value = (key) => block.match(new RegExp(`^${key}\\s*=\\s*["']([^\\n]*)["']\\s*$`, 'm'))?.[1];
      if (value('port') && value('crate') && value('source')) adapters.push({ port: value('port'), crate: value('crate'), source: value('source'), external: true });
    }
  }
  return adapters.sort((a, b) => `${a.port}/${a.crate}`.localeCompare(`${b.port}/${b.crate}`));
}

/**
 * Builds the dependency block of the server manifest.
 *
 * @param {{port: string, crate: string, source: string}[]} adapters The adapters.
 * @returns {string} The block, markers included.
 */
function cargoBlock(adapters) {
  return [START, ...adapters.map((a) => `${a.crate} = { ${a.source} }`), END].join('\n');
}

/**
 * Builds the body of `crates/server/src/wiring/adapters.rs`.
 *
 * @param {{port: string, crate: string}[]} adapters The adapters.
 * @returns {string} The Rust source after the header.
 */
function rustBody(adapters) {
  const ports = [...new Set(adapters.map((a) => a.port))];
  const lines = [
    '//! The adapters of every port in this build, written by `tools/repo-check --fix` from',
    '//! `crates/*/adapters/*` and `crates/server/adapters.toml`. Do not edit by hand.',
  ];
  for (const port of ports) {
    const entries = adapters.filter((a) => a.port === port).map((a) => `${a.crate.replace(/-/g, '_')}::ADAPTER`);
    lines.push('', `/// The ${port} adapters of this build.`, `pub const ${port.toUpperCase()}: &[kampus_${port}::Adapter] = &[${entries.join(', ')}];`);
  }
  const external = adapters.filter((a) => a.external).map((a) => JSON.stringify(a.crate));
  lines.push('', '/// The packages of the adapters that come from outside the repository.', `pub const EXTERNAL: &[&str] = &[${external.join(', ')}];`);
  return `${lines.join('\n')}\n`;
}

/**
 * Checks, and with `fix` writes, the adapter registry of the server.
 *
 * @param {string} root Repository root.
 * @param {{fix: boolean, author: string, since: string}} options Fix mode and the fields of a new header.
 * @returns {{problems: string[], summary: string}} Problems found and a one-line summary.
 */
export function checkAdapters(root, { fix, author, since }) {
  const manifest = path.join(root, 'crates', 'server', 'Cargo.toml');
  if (!fs.existsSync(manifest)) return { problems: [], summary: 'no server' };
  const adapters = listAdapters(root);
  const problems = [];

  const text = fs.readFileSync(manifest, 'utf8');
  const block = cargoBlock(adapters);
  const current = text.includes(START) ? text.slice(text.indexOf(START), text.indexOf(END) + END.length) : null;
  if (current !== block) {
    if (fix) {
      const next = current === null ? text.replace(/^\[dependencies\]\n/m, `[dependencies]\n${block}\n`) : text.replace(current, block);
      fs.writeFileSync(manifest, next);
    } else {
      problems.push('crates/server/Cargo.toml: the adapter dependencies are out of date (run tools/repo-check with --fix)');
    }
  }

  const source = path.join(root, 'crates', 'server', 'src', 'wiring', 'adapters.rs');
  const existing = fs.existsSync(source) ? fs.readFileSync(source, 'utf8') : null;
  const top = existing?.match(/^(?:\/\/.*\n)+\n/)?.[0]
    ?? header('slash', { pkg: 'kampus-server', author, since, year: new Date().getFullYear().toString() });
  const expected = top + rustBody(adapters);
  if (existing !== expected) {
    if (fix) fs.writeFileSync(source, expected);
    else problems.push('crates/server/src/wiring/adapters.rs: the adapter registry is out of date (run tools/repo-check with --fix)');
  }
  return { problems, summary: `${adapters.length} adapter(s) registered` };
}
