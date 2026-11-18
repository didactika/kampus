/**
 * Documentation checks for Kampus.
 *
 * - Every relative link in a Markdown file points to an existing file or folder.
 * - Every PlantUML source has its rendered SVG next to it.
 * - Every PlantUML source renders without errors on the PlantUML server, when one is given.
 * - No foreign key of a data model points to a module later in the architecture's ranking.
 *
 * Usage: node tools/docs-check/check.mjs [--plantuml-url http://plantuml:8080]
 */
import fs from 'node:fs';
import path from 'node:path';
import { checkSchemaDependencies } from './schema-dependencies.mjs';

const args = process.argv.slice(2);
const urlIndex = args.indexOf('--plantuml-url');
const plantumlUrl = urlIndex >= 0 ? args[urlIndex + 1] : process.env.PLANTUML_URL;
const root = process.cwd();
const ignored = new Set(['.git', 'node_modules', 'target']);

/** Lists the files under a folder, recursively. */
function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return ignored.has(entry.name) ? [] : walk(full);
    return [full];
  });
}

const files = walk(root);
const errors = [];

for (const file of files.filter((f) => f.endsWith('.md'))) {
  const text = fs.readFileSync(file, 'utf8').replace(/```[\s\S]*?```/g, '');
  for (const match of text.matchAll(/\]\(([^)\s]+)\)/g)) {
    const target = match[1].split('#')[0];
    if (!target || /^[a-z]+:/i.test(target)) continue;
    const resolved = path.resolve(path.dirname(file), decodeURIComponent(target));
    if (!resolved.startsWith(root)) continue;
    if (!fs.existsSync(resolved)) errors.push(`${path.relative(root, file)}: broken link ${match[1]}`);
  }
}

const sources = files.filter((f) => f.endsWith('.puml'));
for (const source of sources) {
  const svg = source.replace(/\.puml$/, '.svg');
  if (!fs.existsSync(svg)) errors.push(`${path.relative(root, source)}: missing rendered ${path.basename(svg)}`);
}

if (plantumlUrl) {
  for (const source of sources) {
    const body = fs.readFileSync(source, 'utf8');
    const response = await fetch(`${plantumlUrl.replace(/\/$/, '')}/svg`, { method: 'POST', body });
    const svg = await response.text();
    if (!response.ok || /Syntax Error|An error has occured/i.test(svg)) {
      errors.push(`${path.relative(root, source)}: PlantUML render failed (${response.status})`);
    }
  }
} else {
  console.log('No PlantUML server given: rendering not checked.');
}

errors.push(...checkSchemaDependencies(root));

const markdownCount = files.filter((f) => f.endsWith('.md')).length;
if (errors.length) {
  console.error(errors.join('\n'));
  console.error(`${errors.length} problem(s) found.`);
  process.exit(1);
}
console.log(`Documentation OK: ${markdownCount} Markdown files, ${sources.length} diagrams.`);
