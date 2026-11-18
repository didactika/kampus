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
 * Version and changelog rules of DEC-081: one workspace version, equal to the release branch it is on,
 * a well-formed changelog, and an entry in it for every change outside the specification.
 */
import fs from 'node:fs';
import path from 'node:path';

const SEMVER = /^(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?$/;
const GROUPS = ['Added', 'Changed', 'Deprecated', 'Removed', 'Fixed', 'Security', 'Upgrade'];

/**
 * Reads the platform version from `[workspace.package]` of the root manifest.
 *
 * @param {string} root Repository root.
 * @returns {string|null} The version, or null without a workspace.
 */
export function workspaceVersion(root) {
  const manifest = path.join(root, 'Cargo.toml');
  if (!fs.existsSync(manifest)) return null;
  const section = fs.readFileSync(manifest, 'utf8').split(/^\[/m).find((s) => s.startsWith('workspace.package]'));
  return section?.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? null;
}

/**
 * Compares two semantic versions.
 *
 * @param {string} a First version.
 * @param {string} b Second version.
 * @returns {number} Negative, zero or positive.
 */
function compare(a, b) {
  const pa = a.match(SEMVER).slice(1, 4).map(Number);
  const pb = b.match(SEMVER).slice(1, 4).map(Number);
  return pa[0] - pb[0] || pa[1] - pb[1] || pa[2] - pb[2];
}

/**
 * Checks the version, the changelog and, when the changed files are known, the changelog entry.
 *
 * @param {string} root Repository root.
 * @param {{branch: string|null, changed: string[]|null}} context Current branch and files changed by the pull request.
 * @returns {{problems: string[], summary: string}} Problems found and a one-line summary.
 */
export function checkVersions(root, { branch, changed }) {
  const problems = [];
  const version = workspaceVersion(root);
  if (version && !SEMVER.test(version)) problems.push(`Cargo.toml: workspace version ${version} is not semantic`);
  const release = branch?.match(/^release\/(\d+\.\d+\.\d+)$/)?.[1];
  if (version && release && version !== release) problems.push(`Cargo.toml: workspace version ${version} differs from the branch release/${release}`);

  const file = path.join(root, 'CHANGELOG.md');
  if (!fs.existsSync(file)) {
    problems.push('CHANGELOG.md: missing');
    return { problems, summary: 'no changelog' };
  }
  const text = fs.readFileSync(file, 'utf8');
  const sections = [...text.matchAll(/^## \[([^\]]+)\](?: - (\d{4}-\d{2}-\d{2}))?$/gm)];
  if (sections[0]?.[1] !== 'Unreleased') problems.push('CHANGELOG.md: the first section must be [Unreleased]');
  const released = sections.slice(1);
  for (const [, v, date] of released) {
    if (!SEMVER.test(v)) problems.push(`CHANGELOG.md: [${v}] is not a semantic version`);
    if (!date) problems.push(`CHANGELOG.md: [${v}] has no release date`);
  }
  for (let i = 1; i < released.length; i += 1) {
    if (SEMVER.test(released[i - 1][1]) && SEMVER.test(released[i][1]) && compare(released[i - 1][1], released[i][1]) <= 0) {
      problems.push(`CHANGELOG.md: [${released[i - 1][1]}] must be newer than [${released[i][1]}]`);
    }
  }
  if (version && released[0] && SEMVER.test(released[0][1]) && compare(version, released[0][1]) < 0) {
    problems.push(`Cargo.toml: workspace version ${version} is older than the last release ${released[0][1]}`);
  }
  for (const m of text.matchAll(/^### (.+)$/gm)) if (!GROUPS.includes(m[1].trim())) problems.push(`CHANGELOG.md: unknown group "${m[1].trim()}"`);

  if (changed) {
    const needsEntry = changed.filter((f) => !/^docs\//.test(f) && !/^\.github\//.test(f) && !/\.md$/.test(f));
    if (needsEntry.length && !changed.includes('CHANGELOG.md')) {
      problems.push(`CHANGELOG.md: no entry for the changes in ${needsEntry.slice(0, 5).join(', ')}${needsEntry.length > 5 ? '…' : ''}`);
    }
  }
  return { problems, summary: `version ${version ?? 'none'}, changelog with ${released.length} release(s)` };
}
