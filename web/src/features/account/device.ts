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
// @package   web
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/** What a session was opened from, read from the client it reported, in a few words. */

/** The browsers told apart, the first that matches wins. */
const BROWSERS: readonly (readonly [RegExp, string])[] = [
  [/Edg\//, "Edge"],
  [/OPR\//, "Opera"],
  [/Firefox\//, "Firefox"],
  [/Chrome\//, "Chrome"],
  [/Safari\//, "Safari"],
];

/** The systems told apart, the first that matches wins. */
const SYSTEMS: readonly (readonly [RegExp, string])[] = [
  [/Android/, "Android"],
  [/iPhone|iPad|iOS/, "iOS"],
  [/Windows/, "Windows"],
  [/Mac OS X|Macintosh/, "macOS"],
  [/Linux|X11/, "Linux"],
];

/** The browser and system of a client, or the name of a program; nothing when unknown. */
export function deviceOf(client: string | null | undefined): { readonly browser: string; readonly system: string | null } | null {
  if (!client) return null;
  const browser = BROWSERS.find(([pattern]) => pattern.test(client))?.[1];
  const system = SYSTEMS.find(([pattern]) => pattern.test(client))?.[1] ?? null;
  if (browser) return { browser, system };
  const program = /^([\w.-]+)(?:\/([\d.]+))?/.exec(client.replace(/^Mozilla\/\S+\s*/, ""));
  if (!program) return null;
  return { browser: program[2] ? `${program[1]} ${program[2]}` : program[1]!, system };
}
