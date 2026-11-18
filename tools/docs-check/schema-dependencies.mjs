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
// @package   tools/docs-check
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/**
 * Checks dependency rule 2 of the architecture on the data models: a foreign key may point only to a
 * table of the same module or of a module earlier in the module table of `architecture.md`.
 * Returns the list of problems found.
 */
import fs from 'node:fs';
import path from 'node:path';

const crateOf = { enrol: 'kampus-enrol', api: 'kampus-api', plugins: 'kampus-plugin-host' };

export function checkSchemaDependencies(root) {
  const design = path.join(root, 'docs/03-design');
  const dir = path.join(design, 'data-model');
  if (!fs.existsSync(dir)) return [];
  const order = [...fs.readFileSync(path.join(design, 'architecture.md'), 'utf8').matchAll(/^\| [^|]+ \| `(kampus-[a-z0-9-]+)` \|/gm)].map((m) => m[1]);
  const rank = (schema) => order.indexOf(crateOf[schema] ?? `kampus-${schema}`);
  const problems = [];
  for (const file of fs.readdirSync(dir).filter((f) => f.endsWith('.md') && f !== 'README.md' && f !== 'conventions.md')) {
    let schema = null;
    for (const line of fs.readFileSync(path.join(dir, file), 'utf8').split('\n')) {
      const heading = line.match(/^## Schema `([a-z0-9_]+)`/);
      const single = line.match(/PostgreSQL schema `([a-z0-9_]+)`/);
      if (heading) schema = heading[1];
      else if (single && !schema) schema = single[1];
      const row = line.match(/^\| `([a-z0-9_]+)` \|(.*)$/);
      if (!row || !schema) continue;
      for (const m of row[2].matchAll(/FK → `([a-z0-9_]+)\.([a-z0-9_]+)`/g)) {
        if (m[1] === schema) continue;
        if (rank(m[1]) < 0 || rank(schema) < 0) problems.push(`docs/03-design/data-model/${file}: ${schema}.${row[1]} references unknown module schema ${m[1]}`);
        else if (rank(m[1]) > rank(schema)) problems.push(`docs/03-design/data-model/${file}: ${schema}.${row[1]} references ${m[1]}.${m[2]}, a module later in the ranking`);
      }
    }
  }
  return problems;
}
