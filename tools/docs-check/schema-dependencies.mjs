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
