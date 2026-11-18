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
import { checkArchitecture } from './architecture.mjs';
import { checkHeaders } from './headers.mjs';
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
];
const problems = results.flatMap((r) => r.problems);
if (problems.length) {
  console.error(problems.join('\n'));
  console.error(`${problems.length} repository problem(s).`);
  process.exit(1);
}
console.log(`Repository OK: ${results.map((r) => r.summary).join('; ')}.`);
