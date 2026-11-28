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
 * One design across the client: the patterns of the design system are used, not drawn again. A soft icon tile is the
 * IconTile of `@kampus/ui`; the title of a page comes from the head of its page (PageHeader, the head of a course, the
 * head of a screen of access, the brand panel), never from a heading written by hand.
 */

import fs from 'node:fs';
import path from 'node:path';

/** The only files that write the title of a page. */
const HEADS = [
  'web/src/features/shell/PageHeader.tsx',
  'web/src/features/course/CourseHeader.tsx',
  'web/src/features/session/AuthHeading.tsx',
  'web/src/features/session/BrandPanel.tsx',
  'web/src/app/not-found.tsx',
];

/** Every file under a directory, but the generated and the installed. */
function walk(dir) {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    if (['node_modules', '.next', 'ui'].includes(e.name)) return [];
    return e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)];
  });
}

/**
 * Checks the client and the web parts of the plugins.
 *
 * @param {string} root Repository root.
 * @returns {{problems: string[], summary: string}} Problems found and a one-line summary.
 */
export function checkDesign(root) {
  const plugins = fs.existsSync(path.join(root, 'crates'))
    ? fs.readdirSync(path.join(root, 'crates')).flatMap((type) => {
        const folder = path.join(root, 'crates', type, 'plugins');
        return fs.existsSync(folder) ? fs.readdirSync(folder).map((name) => path.join(folder, name, 'web', 'src')) : [];
      })
    : [];
  const files = [path.join(root, 'web', 'src'), ...plugins].flatMap(walk).filter((f) => /\.tsx?$/.test(f));
  const problems = [];
  for (const file of files) {
    const rel = path.relative(root, file).split(path.sep).join('/');
    const text = fs.readFileSync(file, 'utf8');
    if (text.includes('gradient-soft')) problems.push(`${rel}: a soft icon tile is the IconTile of @kampus/ui, not drawn again`);
    if (/<Typography[^>]*variant="h1"/.test(text) && !HEADS.includes(rel)) problems.push(`${rel}: the title of a page comes from PageHeader, not a heading written by hand`);
  }
  return { problems, summary: `${files.length} file(s) of the client keep one design` };
}
