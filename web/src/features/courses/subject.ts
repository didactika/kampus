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
// @package   web/ui
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/** The icon of the subject of a course, guessed from its names and its category, so its cover says what it is. */

/** Words of a subject and its icon, the first that matches wins. */
const SUBJECTS: readonly (readonly [RegExp, string])[] = [
  [/phys|f[ií]sic|chem|qu[ií]m|bio|scien|cienc/i, "science"],
  [/math|matem|mat\d|calcul|c[aá]lcul|algeb|statist|estad/i, "calculate"],
  [/hist|geog|social|world|mund/i, "public"],
  [/art|design|dise[nñ]o|music|m[uú]sic|draw|dibuj/i, "palette"],
  [/program|code|c[oó]dig|comput|inform|software|data|datos/i, "code"],
  [/lang|idiom|engl|ingl|span|espa|liter|read|lect/i, "translate"],
];

/** The icon of a course: of its subject when it can be told, a book otherwise. */
export function subjectIcon(...names: readonly (string | null | undefined)[]): string {
  const text = names.filter(Boolean).join(" ");
  return SUBJECTS.find(([words]) => words.test(text))?.[1] ?? "auto_stories";
}
