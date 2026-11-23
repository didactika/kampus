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
 * A setting that takes one text per language: read in the language of the reader, or in the first language it has
 * when it has none in theirs; a plain text is the same in every language.
 */

/** The texts of a value, by language; a plain text counts as the text of the language given. */
function byLanguage(value: unknown, language: string): Record<string, string> {
  if (typeof value === "string") return value ? { [language]: value } : {};
  const texts: Record<string, string> = {};
  if (value && typeof value === "object") {
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) if (typeof v === "string") texts[k] = v;
  }
  return texts;
}

/** The text of a value in a language, or in the first language it has. */
export function translated(value: unknown, language: string): string {
  if (typeof value === "string") return value;
  const texts = byLanguage(value, language);
  const own = texts[language] ?? texts[language.split("-")[0] ?? ""];
  if (own && own.trim()) return own;
  return Object.values(texts).find((t) => t.trim()) ?? "";
}

/** The text of a value in exactly one language, empty when it has none there. */
export function textIn(value: unknown, language: string, fallback: string): string {
  if (typeof value === "string") return language === fallback ? value : "";
  return byLanguage(value, language)[language] ?? "";
}

/** A value with the text of one language changed, kept per language; a plain text becomes the fallback's. */
export function withText(value: unknown, language: string, text: string, fallback: string): Readonly<Record<string, string>> {
  return { ...byLanguage(value, fallback), [language]: text };
}
