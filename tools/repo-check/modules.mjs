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
const KINDS = ['foundation', 'port', 'adapter', 'type', 'module', 'server'];

const SRC_FOLDERS = [
  'commands',
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

/** The start of a component's `version.toml`: its table and its version. */
const COMPONENT_VERSION = /^\[component\]\s*\nversion = "/m;

/**
 * The layout of each kind of crate: what may sit at its root besides `Cargo.toml`, `README.md`, `src` and
 * `tests`; the only files at the top of `src/`; the folders allowed there (`null`: any, named by concern);
 * the folders it must have; and the declarations it must make. Every folder under `src/` has a `mod.rs`.
 */
const LAYOUTS = {
  foundation: { root: [], files: ['lib.rs'], folders: null, required: [], declarations: [] },
  port: { root: ['adapters'], files: ['lib.rs'], folders: null, required: ['port', 'registry', 'conformance'], declarations: [] },
  adapter: {
    root: ['assets', 'version.toml'],
    files: ['lib.rs'],
    folders: null,
    required: ['store'],
    declarations: [
      ['src/lib.rs', /pub const ADAPTER: /, 'declares `pub const ADAPTER`'],
      ['version.toml', COMPONENT_VERSION, 'starts its [component] table with the version'],
    ],
  },
  // A plugin type: the contract its plugins build on, with the plugins of the repository in plugins/<name>.
  type: {
    root: ['plugins', 'web', 'assets', 'version.toml'],
    files: ['lib.rs'],
    folders: null,
    required: ['declaration', 'conformance'],
    declarations: [
      ['src/lib.rs', /pub const TYPE: &str = /, 'declares `pub const TYPE`, the prefix of its plugins'],
      ['version.toml', COMPONENT_VERSION, 'starts its [component] table with the version'],
    ],
  },
  server: { root: ['adapters.toml', 'modules.toml'], files: ['lib.rs', 'main.rs'], folders: null, required: ['commands', 'config', 'http', 'wiring'], declarations: [] },
  module: { root: ['web', 'lang', 'assets', 'version.toml'], files: ['lib.rs', 'module.rs'], folders: null, required: ['db', 'permissions'], declarations: null },
};

/** Files a module must have, and what each must declare. */
const REQUIRED = [
  ['version.toml', COMPONENT_VERSION, 'starts its [component] table with the version'],
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
    for (const group of fs.readdirSync(path.join(dir, name), { withFileTypes: true })) {
      if (!group.isDirectory() || ['src', 'tests', 'web', 'lang', 'assets'].includes(group.name)) continue;
      for (const child of fs.readdirSync(path.join(dir, name, group.name))) {
        const inner = path.join(dir, name, group.name, child, 'Cargo.toml');
        if (!fs.existsSync(inner)) continue;
        found.push({ ...describe(inner), dir: path.join('crates', name, group.name, child), owner: name, group: group.name, folder: child });
      }
    }
  }
  return found;
}

/** The modules of the build, in dependency order. */
export function listModules(root) {
  const architecture = fs.readFileSync(path.join(root, 'docs/03-design/architecture.md'), 'utf8');
  const order = [...architecture.matchAll(/^\| [^|]+ \| `(kampus-[a-z0-9-]+)` \|/gm)].map((m) => m[1]);
  // Core modules in the order of the architecture, then the plugins of the repository by name: a plugin needs
  // the module that defines its type, so it comes after every core module.
  const rank = (c) => (c.owner ? order.length : order.indexOf(c.name));
  const local = crates(root)
    .filter((c) => c.kind === 'module')
    .sort((a, b) => rank(a) - rank(b) || a.name.localeCompare(b.name))
    .map((c) => ({ crate: c.name, source: `path = "${path.relative(path.join('crates', 'server'), c.dir).split(path.sep).join('/')}"` }));
  const external = path.join(root, 'crates', 'server', 'modules.toml');
  const declared = [];
  if (fs.existsSync(external)) {
    for (const block of fs.readFileSync(external, 'utf8').split(/^\[\[module\]\]\s*$/m).slice(1)) {
      const value = (key) => block.match(new RegExp(`^${key}\\s*=\\s*["']([^\\n]*)["']\\s*$`, 'm'))?.[1];
      if (value('crate') && value('source')) declared.push({ crate: value('crate'), source: value('source'), external: true });
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
    const hosts = crates(root).filter((o) => o.owner === path.basename(c.dir) && !c.owner).map((o) => o.group);
    if (c.owner) {
      // Adapters live in the folder of their port, and plugins in plugins/ of the crate of their type.
      const type = c.group === 'adapters' || c.group === 'plugins' ? c.owner : c.group;
      const expected = `kampus-${type}-${c.folder.replace(/_/g, '-')}`;
      if (c.name !== expected) problems.push(`${c.dir}: a component of type ${type} in this folder is the package ${expected}, not ${c.name}`);
      if (c.group === 'adapters' && c.kind !== 'adapter') problems.push(`${c.dir}: crates in adapters/ are of kind adapter`);
      if (c.group !== 'adapters' && c.kind !== 'module') problems.push(`${c.dir}: plugins are of kind module`);
    }
    for (const entry of fs.readdirSync(base)) {
      if (!['Cargo.toml', 'README.md', 'src', 'tests', ...layout.root, ...hosts].includes(entry)) {
        problems.push(`${c.dir}/${entry}: not part of ${where}`);
      }
    }
    const src = path.join(base, 'src');
    if (!fs.existsSync(src)) {
      problems.push(`${c.dir}/src: missing`);
      continue;
    }
    for (const entry of fs.readdirSync(src)) {
      const folder = fs.statSync(path.join(src, entry)).isDirectory();
      const allowed = folder ? !layout.folders || layout.folders.includes(entry) : layout.files.includes(entry);
      if (!allowed) problems.push(`${c.dir}/src/${entry}: not part of ${where}`);
    }
    for (const folder of layout.required) {
      if (!fs.existsSync(path.join(src, folder))) problems.push(`${c.dir}/src/${folder}: missing from ${where}`);
    }
    for (const file of walkFiles(src).filter((f) => f.endsWith('.rs'))) {
      if (/^\s*#\[cfg\(test\)\]/m.test(fs.readFileSync(file, 'utf8'))) {
        problems.push(`${path.relative(root, file)}: tests live in tests/, never in the code`);
      }
    }
    const tests = path.join(base, 'tests');
    if (fs.existsSync(tests)) {
      for (const entry of fs.readdirSync(tests, { withFileTypes: true })) {
        const where = `${c.dir}/tests/${entry.name}`;
        if (!entry.isDirectory() || !TEST_KINDS.includes(entry.name)) {
          problems.push(`${where}: tests/ holds only the folders ${TEST_KINDS.join(', ')}`);
        } else if (entry.name !== 'support' && !fs.existsSync(path.join(tests, entry.name, 'main.rs'))) {
          problems.push(`${where}: needs its main.rs, which declares one module per topic`);
        }
      }
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

/**
 * A constant array as rustfmt writes it: on one line when it fits in 110 columns, otherwise one entry
 * per line.
 */
export function rustList(declaration, entries) {
  const line = `${declaration} &[${entries.join(', ')}];`;
  if (line.length <= 110) return line;
  return [`${declaration} &[`, ...entries.map((e) => `    ${e},`), '];'].join('\n');
}

/** The kinds of test a crate may have, one test binary each. */
const TEST_KINDS = ['unit', 'integration', 'conformance', 'support'];

/** Every file under a directory. */
function walkFiles(dir) {
  return fs.readdirSync(dir, { withFileTypes: true })
    .flatMap((e) => (e.isDirectory() ? walkFiles(path.join(dir, e.name)) : [path.join(dir, e.name)]));
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
    rustList('pub const MODULES: &[&kampus_module::Module] =', entries),
    '',
    '/// The packages of the modules that come from outside the repository.',
    `pub const EXTERNAL: &[&str] = &[${modules.filter((m) => m.external).map((m) => JSON.stringify(m.crate)).join(', ')}];`,
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
