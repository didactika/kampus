// This file is part of Kampus - Knowledge Academic Modular Platform for Universal Study.
//
// Copyright (c) 2026 Didactika. All rights reserved.
// Proprietary and confidential: see the LICENSE file at the root of the repository.
// SPDX-License-Identifier: LicenseRef-Didactika-Proprietary
//
// @package   tools/docs-check
// @author    hector-ae21
// @since     0.1.0

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
