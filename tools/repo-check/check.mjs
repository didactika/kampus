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
 * Repository checks of Kampus: file headers (DEC-083), version and changelog (DEC-081) and the
 * dependency rules of the crates (architecture).
 *
 * Usage: node tools/repo-check/check.mjs [--fix] [--author <name>] [--branch <name>] [--files <list>] [--changed <list>]
 *
 * `--files` and `--changed` name text files with one path per line: the files to check (by default every
 * file of the tree) and the files changed by the pull request, which then need a changelog entry.
 */
import fs from 'node:fs';
import path from 'node:path';
import { checkAdapters } from './adapters.mjs';
import { checkArchitecture } from './architecture.mjs';
import { checkHeaders } from './headers.mjs';
import { checkModules } from './modules.mjs';
import { checkVersions, workspaceVersion } from './versions.mjs';

const args = process.argv.slice(2);
const option = (name) => (args.includes(name) ? args[args.indexOf(name) + 1] : null);
const root = process.cwd();
const ignored = new Set(['.git', 'node_modules', 'target', '.svelte-kit']);

/**
 * Lists every file of the tree, relative to the root.
 *
 * @param {string} dir Directory relative to the root.
 * @returns {string[]} Paths with forward slashes.
 */
function walk(dir) {
  return fs.readdirSync(path.join(root, dir), { withFileTypes: true }).flatMap((entry) => {
    if (ignored.has(entry.name)) return [];
    const rel = dir ? `${dir}/${entry.name}` : entry.name;
    return entry.isDirectory() ? walk(rel) : [rel];
  });
}

const listed = option('--files');
const files = listed ? fs.readFileSync(listed, 'utf8').split('\n').filter(Boolean) : walk('');
const changedFile = option('--changed');
const changed = changedFile ? fs.readFileSync(changedFile, 'utf8').split('\n').filter(Boolean) : null;
const branch = option('--branch') ?? process.env.GITHUB_HEAD_REF ?? process.env.GITHUB_REF_NAME ?? null;

const results = [
  checkHeaders(root, files, { fix: args.includes('--fix'), author: option('--author') ?? process.env.KAMPUS_AUTHOR ?? 'unknown', since: workspaceVersion(root) ?? '0.1.0' }),
  checkVersions(root, { branch, changed }),
  checkArchitecture(root),
  checkAdapters(root, { fix: args.includes('--fix'), author: option('--author') ?? process.env.KAMPUS_AUTHOR ?? 'unknown', since: workspaceVersion(root) ?? '0.1.0' }),
  checkModules(root, { fix: args.includes('--fix'), author: option('--author') ?? process.env.KAMPUS_AUTHOR ?? 'unknown', since: workspaceVersion(root) ?? '0.1.0' }),
];
const problems = results.flatMap((r) => r.problems);
if (problems.length) {
  console.error(problems.join('\n'));
  console.error(`${problems.length} repository problem(s).`);
  process.exit(1);
}
console.log(`Repository OK: ${results.map((r) => r.summary).join('; ')}.`);
