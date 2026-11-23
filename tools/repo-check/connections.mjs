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
 * A service never holds a connection of the pool, or a transaction, while it calls another service: the other one
 * takes a connection of its own, and under load every connection ends up held by a request waiting for a second
 * one, until the pool times out. While holding one, a service passes it on (`check_in`, `require_in`, `course_in`,
 * `policy_in` and the other `_in` variants), or gives it back first (`drop`).
 */

import fs from 'node:fs';
import path from 'node:path';

/** Fields of a service that touch no database, so holding a connection while they run starves nothing. */
const HARMLESS = new Set(['hasher.verify', 'hasher.hash']);

/**
 * Every `.rs` file under a folder, outside tests.
 *
 * @param {string} dir The folder.
 * @returns {string[]} The files.
 */
function sources(dir) {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) return ['tests', 'target', 'web'].includes(entry.name) ? [] : sources(file);
    return entry.name.endsWith('.rs') ? [file] : [];
  });
}

/**
 * Finds calls to another service made while a connection or a transaction is held.
 *
 * @param {string} root The repository.
 * @returns {{problems: string[], summary: string}} What it found.
 */
export function checkConnections(root) {
  const holder = /let mut (\w+) = self\.(?:db\.pool\(\)\.acquire\(\)|db\.begin\(\))/;
  const problems = [];
  let held = 0;
  for (const file of sources(path.join(root, 'crates'))) {
    const lines = fs.readFileSync(file, 'utf8').split('\n');
    for (let i = 0; i < lines.length; i++) {
      const found = lines[i].match(holder);
      if (!found) continue;
      held += 1;
      const name = found[1];
      const indent = lines[i].match(/^\s*/)[0].length;
      for (let j = i + 1; j < lines.length; j++) {
        const line = lines[j];
        if (line.trim() && line.match(/^\s*/)[0].length < indent) break;
        if (new RegExp(`drop\\(${name}\\)|${name}\\.commit\\(\\)|${name}\\.rollback\\(\\)`).test(line)) break;
        if (j > i + 1 && !/[;{}]\s*$/.test(lines[j - 1]) && !/^\s*(let|if|match|for|while|\(|self)/.test(line)) continue;
        let end = j;
        let statement = line;
        while (!/[;{}]\s*$/.test(lines[end]) && end < j + 6 && end + 1 < lines.length) statement += ` ${lines[++end].trim()}`;
        const call = statement.match(/self\s*\.(?!db\b|table\b)(\w+)\s*\.(\w+)\(/);
        if (!call || !/\.await/.test(statement) || new RegExp(`&mut \\*?${name}\\b`).test(statement)) continue;
        if (HARMLESS.has(`${call[1]}.${call[2]}`)) continue;
        const where = path.relative(root, file).replaceAll('\\', '/');
        problems.push(`${where}:${j + 1}: self.${call[1]}.${call[2]}(…) while holding \`${name}\` (line ${i + 1}); pass it on with an \`_in\` variant or drop it first`);
      }
    }
  }
  return { problems, summary: `${held} held connection(s) never wait for another` };
}
