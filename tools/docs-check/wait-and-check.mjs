/**
 * Waits for the PlantUML server of the local compose project, then runs the documentation checks.
 */
const url = process.env.PLANTUML_URL ?? 'http://plantuml:8080';

for (let attempt = 0; attempt < 60; attempt++) {
  const ready = await fetch(`${url}/`).then((r) => r.ok).catch(() => false);
  if (ready) break;
  await new Promise((resolve) => setTimeout(resolve, 1000));
}

await import('./check.mjs');
