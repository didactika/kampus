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


/**
 * The help of each screen: the guide of the handbook that explains it, in the language of the reader. The address of
 * the handbook is configuration (`KAMPUS_HELP_URL`); without it the client offers no help link.
 */

/** The guide of each kind of page, by the start of its path; the first that matches wins. */
const TOPICS: readonly (readonly [RegExp, string])[] = [
  [/^\/notifications\/preferences/, "learners/notifications/"],
  [/^\/notifications/, "learners/notifications/"],
  [/^\/account/, "learners/your-account/"],
  [/^\/courses\/[^/]+/, "learners/joining-a-course/"],
  [/^\/courses/, "learners/joining-a-course/"],
  [/^\/(sign-in|recovery|invitation|sign-up|email)/, "learners/your-account/"],
  [/^\/admin/, "administrators/"],
];

/** The path of the guide of a page, relative to the handbook. */
export function helpTopic(path: string): string {
  return TOPICS.find(([pattern]) => pattern.test(path))?.[1] ?? "learners/";
}

/** The address of the guide of a page, or `null` when the campus has no handbook address. */
export function helpAddress(base: string | null | undefined, language: string, path: string): string | null {
  if (!base) return null;
  const root = base.endsWith("/") ? base : `${base}/`;
  const locale = language === "en" ? "" : `${language}/`;
  return `${root}${locale}${helpTopic(path)}`;
}
