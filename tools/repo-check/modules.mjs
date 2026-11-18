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
 * The modules of the build and their standard layout.
 *
 * Every crate says what it is in `[package.metadata.kampus] kind`: `foundation`, `port`, `adapter`,
 * `module` or `server`. Module crates follow one layout, so every declaration has one place, and the
 * server's module registry (`crates/server/src/wiring/registry.rs` and the `# modules:start/end` block of its
 * manifest) is written from the module crates, in the order of the module table of the architecture,
 * followed by the modules declared in `crates/server/modules.toml`.
 */
import fs from 'node:fs';
import path from 'node:path';
import { header } from './headers.mjs';

const START = '# modules:start';
const END = '# modules:end';

/** The kinds a crate may be. */
const KINDS = ['foundation', 'port', 'adapter', 'module', 'server'];

const SRC_FOLDERS = [
  'db',
  'permissions',
  'model',
  'service',
  'ports',
  'support',
  'jobs',
  'events',
  'observers',
  'settings',
  'http',
  'privacy',
];

/**
 * The layout of each kind of crate: what may sit at its root besides `Cargo.toml`, `README.md`, `src` and
 * `tests`; the only files at the top of `src/`; the folders allowed there (`null`: any, named by concern);
 * the folders it must have; and the declarations it must make. Every folder under `src/` has a `mod.rs`.
 */
const LAYOUTS = {
  foundation: { root: [], files: ['lib.rs'], folders: null, required: [], declarations: [] },
  port: { root: ['adapters'], files: ['lib.rs'], folders: null, required: ['port', 'registry', 'conformance'], declarations: [] },
  adapter: {
    root: [],
    files: ['lib.rs'],
    folders: null,
    required: ['store'],
    declarations: [['src/lib.rs', /pub const ADAPTER: /, 'declares `pub const ADAPTER`']],
  },
  server: { root: ['adapters.toml', 'modules.toml'], files: ['lib.rs', 'main.rs'], folders: null, required: ['commands', 'config', 'http', 'wiring'], declarations: [] },
  module: { root: ['web', 'lang', 'assets'], files: ['lib.rs', 'module.rs'], folders: null, required: ['db', 'permissions'], declarations: null },
};

/** Files a module must have, and what each must declare. */
const REQUIRED = [
  ['src/module.rs', /pub const MODULE: Module = Module \{/, 'declares `pub const MODULE: Module`'],
  ['src/db/mod.rs', /pub use steps::/, 're-exports its steps'],
  ['src/db/steps.rs', /pub const STEPS: &\[Step\]/, 'declares `pub const STEPS: &[Step]`'],
  ['src/permissions/mod.rs', /pub const PERMISSIONS: &\[PermissionDeclaration\]/, 'declares `PERMISSIONS`'],
  ['src/permissions/templates.rs', /pub const TEMPLATES: &\[RoleTemplateDeclaration\]/, 'declares `TEMPLATES`'],
];

LAYOUTS.module.folders = SRC_FOLDERS;
LAYOUTS.module.declarations = REQUIRED;

/** The name and kind of a manifest. */
function describe(file) {
  const text = fs.readFileSync(file, 'utf8');
  return {
    name: text.match(/^name\s*=\s*"([^"]+)"/m)?.[1] ?? null,
    kind: text.match(/\[package\.metadata\.kampus\][^[]*?kind\s*=\s*"([^"]+)"/s)?.[1] ?? null,
  };
}

/** Every crate of the repository, with its folder. */
function crates(root) {
  const dir = path.join(root, 'crates');
  const found = [];
  if (!fs.existsSync(dir)) return found;
  for (const name of fs.readdirSync(dir)) {
    const manifest = path.join(dir, name, 'Cargo.toml');
    if (!fs.existsSync(manifest)) continue;
    found.push({ ...describe(manifest), dir: path.join('crates', name) });
    const adapters = path.join(dir, name, 'adapters');
    if (!fs.existsSync(adapters)) continue;
    for (const adapter of fs.readdirSync(adapters)) {
      const inner = path.join(adapters, adapter, 'Cargo.toml');
      if (fs.existsSync(inner)) found.push({ ...describe(inner), dir: path.join('crates', name, 'adapters', adapter) });
    }
  }
  return found;
}

/** The modules of the build, in dependency order. */
export function listModules(root) {
  const architecture = fs.readFileSync(path.join(root, 'docs/03-design/architecture.md'), 'utf8');
  const order = [...architecture.matchAll(/^\| [^|]+ \| `(kampus-[a-z0-9-]+)` \|/gm)].map((m) => m[1]);
  const local = crates(root)
    .filter((c) => c.kind === 'module')
    .sort((a, b) => order.indexOf(a.name) - order.indexOf(b.name))
    .map((c) => ({ crate: c.name, source: `path = "../${path.basename(c.dir)}"` }));
  const external = path.join(root, 'crates', 'server', 'modules.toml');
  const declared = [];
  if (fs.existsSync(external)) {
    for (const block of fs.readFileSync(external, 'utf8').split(/^\[\[module\]\]\s*$/m).slice(1)) {
      const value = (key) => block.match(new RegExp(`^${key}\\s*=\\s*["']([^\\n]*)["']\\s*$`, 'm'))?.[1];
      if (value('crate') && value('source')) declared.push({ crate: value('crate'), source: value('source') });
    }
  }
  return [...local, ...declared];
}

/** Problems with the kind of every crate and the layout of module crates. */
function checkLayout(root) {
  const problems = [];
  for (const c of crates(root)) {
    const layout = LAYOUTS[c.kind];
    if (!layout) {
      problems.push(`${c.dir}/Cargo.toml: [package.metadata.kampus] kind must be one of ${KINDS.join(', ')}`);
      continue;
    }
    const base = path.join(root, c.dir);
    const where = `the ${c.kind} layout`;
    for (const entry of fs.readdirSync(base)) {
      if (!['Cargo.toml', 'README.md', 'src', 'tests', ...layout.root].includes(entry)) {
        problems.push(`${c.dir}/${entry}: not part of ${where}`);
      }
    }
    const src = path.join(base, 'src');
    for (const entry of fs.readdirSync(src)) {
      const folder = fs.statSync(path.join(src, entry)).isDirectory();
      const allowed = folder ? !layout.folders || layout.folders.includes(entry) : layout.files.includes(entry);
      if (!allowed) problems.push(`${c.dir}/src/${entry}: not part of ${where}`);
    }
    for (const folder of layout.required) {
      if (!fs.existsSync(path.join(src, folder))) problems.push(`${c.dir}/src/${folder}: missing from ${where}`);
    }
    for (const folder of walkFolders(src)) {
      const name = path.basename(folder);
      if (!/^[a-z][a-z0-9_]*$/.test(name)) problems.push(`${path.relative(root, folder)}: folders are snake case`);
      if (!fs.existsSync(path.join(folder, 'mod.rs'))) problems.push(`${path.relative(root, folder)}: a folder needs its mod.rs`);
    }
    for (const [file, pattern, what] of layout.declarations) {
      const full = path.join(base, file);
      if (!fs.existsSync(full)) problems.push(`${c.dir}/${file}: missing`);
      else if (!pattern.test(fs.readFileSync(full, 'utf8'))) problems.push(`${c.dir}/${file}: ${what}`);
    }
  }
  return problems;
}

/** Every folder under a directory, deepest last. */
function walkFolders(dir) {
  return fs.readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .flatMap((e) => [path.join(dir, e.name), ...walkFolders(path.join(dir, e.name))]);
}

/** The registry of the server. */
function rustBody(modules) {
  const entries = modules.map((m) => `&${m.crate.replace(/-/g, '_')}::MODULE`);
  return [
    '//! The modules of this build, in dependency order, written by `tools/repo-check --fix` from',
    '//! the module crates and `crates/server/modules.toml`. Do not edit by hand.',
    '',
    '/// Every module of this build.',
    `pub const MODULES: &[&kampus_module::Module] = &[${entries.join(', ')}];`,
    '',
  ].join('\n');
}

/** Checks the layout and the registry, writing the registry with `fix`. */
export function checkModules(root, { fix, author, since }) {
  const problems = checkLayout(root);
  const manifest = path.join(root, 'crates', 'server', 'Cargo.toml');
  if (!fs.existsSync(manifest)) return { problems, summary: 'no server' };
  const modules = listModules(root);

  const text = fs.readFileSync(manifest, 'utf8');
  const block = [START, ...modules.map((m) => `${m.crate} = { ${m.source} }`), END].join('\n');
  const current = text.includes(START) ? text.slice(text.indexOf(START), text.indexOf(END) + END.length) : null;
  if (current !== block) {
    if (fix) {
      const next = current === null ? text.replace(/^\[dependencies\]\n/m, `[dependencies]\n${block}\n`) : text.replace(current, block);
      fs.writeFileSync(manifest, next);
    } else {
      problems.push('crates/server/Cargo.toml: the module dependencies are out of date (run tools/repo-check with --fix)');
    }
  }

  const source = path.join(root, 'crates', 'server', 'src', 'wiring', 'registry.rs');
  const existing = fs.existsSync(source) ? fs.readFileSync(source, 'utf8') : null;
  const top = existing?.match(/^(?:\/\/.*\n)+\n/)?.[0]
    ?? header('slash', { pkg: 'kampus-server', author, since, year: new Date().getFullYear().toString() });
  const expected = top + rustBody(modules);
  if (existing !== expected) {
    if (fix) fs.writeFileSync(source, expected);
    else problems.push('crates/server/src/wiring/registry.rs: the module registry is out of date (run tools/repo-check with --fix)');
  }
  return { problems, summary: `${modules.length} module(s) registered in the standard layout` };
}
