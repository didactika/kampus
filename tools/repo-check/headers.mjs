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
 * The standard file header of DEC-083: which files need it, how it is written in each comment syntax,
 * and how a missing one is added.
 */
import fs from 'node:fs';
import path from 'node:path';

/** Comment syntax per file kind: line prefix, or opening, line prefix and closing for block comments. */
const STYLES = {
  slash: { line: '//' },
  hash: { line: '#' },
  dash: { line: '--' },
  block: { open: '/*', line: ' *', close: ' */' },
  markup: { open: '<!--', line: '  ', close: '-->' },
};

/** File name patterns mapped to their comment syntax. */
const KINDS = [
  [/\.(rs|js|mjs|cjs|ts|tsx|jsx)$/, 'slash'],
  [/\.(toml|ya?ml|sh|bash)$/, 'hash'],
  [/(^|\/)(Dockerfile|\.gitignore|\.gitattributes|\.dockerignore|\.env\.example|\.editorconfig)$/, 'hash'],
  [/\.(css|scss)$/, 'block'],
  [/\.(svelte|html)$/, 'markup'],
  [/\.sql$/, 'dash'],
];

/** Paths that never carry the header: the specification, generated or binary files, the licence. */
const EXEMPT = [/^docs\//, /\.md$/, /\.json$/, /\.lock$/, /^LICENSE$/, /\.(svg|png|jpg|ico|woff2?)$/, /(^|\/)node_modules\//, /^target\//];

export const MARKER = 'This file is part of Kampus';

/**
 * Returns the comment syntax of a file, or null when it needs no header.
 *
 * @param {string} file Path relative to the repository root, with forward slashes.
 * @returns {string|null} A key of STYLES.
 */
export function styleOf(file) {
  if (EXEMPT.some((re) => re.test(file))) return null;
  return KINDS.find(([re]) => re.test(file))?.[1] ?? null;
}

/**
 * Derives the `@package` of a file from where it lives.
 *
 * @param {string} file Path relative to the repository root.
 * @returns {string} The crate, the component or `kampus`.
 */
export function packageOf(file) {
  const parts = file.split('/');
  if (parts[0] === 'crates' && parts.length > 2) return parts[1];
  if (['tools', 'web', 'handbook'].includes(parts[0]) && parts.length > 2) return `${parts[0]}/${parts[1]}`;
  if (['web', 'handbook'].includes(parts[0])) return parts[0];
  return 'kampus';
}

/**
 * Builds the header text of a file.
 *
 * @param {string} style A key of STYLES.
 * @param {{pkg: string, author: string, since: string, year: string}} fields Header fields.
 * @returns {string} The header, ending with a blank line.
 */
export function header(style, { pkg, author, since, year }) {
  const s = STYLES[style];
  const body = [
    `${MARKER} - Knowledge Academic Modular Platform for Universal Study.`,
    '',
    `Copyright (c) ${year} Didactika. All rights reserved.`,
    'Proprietary and confidential: see the LICENSE file at the root of the repository.',
    'SPDX-License-Identifier: LicenseRef-Didactika-Proprietary',
    '',
    `@package   ${pkg}`,
    `@author    ${author}`,
    `@since     ${since}`,
  ];
  const lines = body.map((l) => (l ? `${s.line} ${l}` : s.line).trimEnd());
  return [...(s.open ? [s.open] : []), ...lines, ...(s.close ? [s.close] : []), '', ''].join('\n');
}

/**
 * Tells whether a file starts with a complete header (after an optional shebang).
 *
 * @param {string} text File contents.
 * @returns {boolean} True when every field is present in the first lines.
 */
export function hasHeader(text) {
  const head = text.split('\n').slice(0, 16).join('\n');
  return head.includes(MARKER) && /SPDX-License-Identifier: LicenseRef-Didactika-Proprietary/.test(head)
    && /@package\s+\S/.test(head) && /@author\s+\S/.test(head) && /@since\s+\S/.test(head);
}

/**
 * Checks, and optionally fixes, the headers of the given files.
 *
 * @param {string} root Repository root.
 * @param {string[]} files Paths relative to the root.
 * @param {{fix: boolean, author: string, since: string}} options Fix mode and the fields of new headers.
 * @returns {{problems: string[], summary: string}} Problems left and a one-line summary.
 */
export function checkHeaders(root, files, { fix, author, since }) {
  const problems = [];
  let checked = 0;
  let fixed = 0;
  for (const file of files) {
    const style = styleOf(file);
    if (!style) continue;
    checked += 1;
    const full = path.join(root, file);
    const text = fs.readFileSync(full, 'utf8');
    if (hasHeader(text)) continue;
    if (!fix) {
      problems.push(`${file}: missing the standard header (run tools/repo-check with --fix)`);
      continue;
    }
    const block = header(style, { pkg: packageOf(file), author, since, year: new Date().getFullYear().toString() });
    const shebang = text.startsWith('#!') ? `${text.split('\n')[0]}\n` : '';
    fs.writeFileSync(full, shebang + block + text.slice(shebang.length));
    fixed += 1;
  }
  return { problems, summary: `${checked} file(s) with headers${fixed ? `, ${fixed} added` : ''}` };
}
