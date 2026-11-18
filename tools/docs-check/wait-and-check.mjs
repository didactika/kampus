// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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
// @package   tools/docs-check
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

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
