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
 * Choosing the language of a request, and the texts and formatters of a language. The texts come from the strings
 * folders of the shell and of every UI module (see `scripts/collect-strings.ts`); a text a language lacks is the
 * English one, and a key with no text at all shows as itself.
 */

import { BUNDLES } from "./strings.generated";

/** The languages the client speaks: every language with texts in a strings folder. */
export const LANGUAGES = Object.keys(BUNDLES);

/** The languages written right to left. */
const RIGHT_TO_LEFT = new Set(["ar", "fa", "he", "ur"]);

/** The language of a request from its `Accept-Language`: the first the shell speaks, English otherwise. */
export function languageOf(acceptLanguage: string | null | undefined): string {
  const asked = (acceptLanguage ?? "")
    .split(",")
    .map((part) => part.trim().split(";")[0]!.toLowerCase().slice(0, 2))
    .filter(Boolean);
  return asked.find((code) => code in BUNDLES) ?? "en";
}

/** The direction of a language. */
export function directionOf(language: string): "ltr" | "rtl" {
  return RIGHT_TO_LEFT.has(language) ? "rtl" : "ltr";
}

/** Gives a message of a language with `{name}` placeholders filled in. */
export type Translate = (key: string, values?: Readonly<Record<string, string | number>>) => string;

/**
 * The texts of a language: a missing one falls back to English, then to its key. When the values carry a count `n`
 * and the key has plural forms (`<key>.one`, `<key>.other`…, as `Intl.PluralRules` names them), the form of that count
 * is used.
 */
export function translator(language: string): Translate {
  const rules = new Intl.PluralRules(language);
  const lookup = (key: string) => BUNDLES[language]?.[key] ?? BUNDLES["en"]?.[key];
  return (key, values = {}) => {
    const count = values["n"];
    const plural = typeof count === "number" ? lookup(`${key}.${rules.select(count)}`) ?? lookup(`${key}.other`) : undefined;
    const text = plural ?? lookup(key) ?? key;
    return text.replace(/\{(\w+)\}/g, (_, name: string) => String(values[name] ?? ""));
  };
}

/** A date and time in a language. */
export function formatDate(language: string, iso: string): string {
  return new Intl.DateTimeFormat(language, { dateStyle: "medium", timeStyle: "short" }).format(new Date(iso));
}

/** A day in a language. */
export function formatDay(language: string, iso: string): string {
  return new Intl.DateTimeFormat(language, { dateStyle: "medium" }).format(new Date(iso));
}

/** How long ago a moment was, in a language ("2 hours ago"), or its date and time after a week. */
export function formatRelative(language: string, iso: string, now = Date.now()): string {
  const seconds = Math.round((new Date(iso).getTime() - now) / 1000);
  const steps: [Intl.RelativeTimeFormatUnit, number][] = [["second", 60], ["minute", 60], ["hour", 24], ["day", 7]];
  let value = seconds;
  for (const [unit, size] of steps) {
    if (Math.abs(value) < size) return new Intl.RelativeTimeFormat(language, { numeric: "auto" }).format(value, unit);
    value = Math.round(value / size);
  }
  return formatDate(language, iso);
}

/** A number in a language. */
export function formatNumber(language: string, value: string | number): string {
  return new Intl.NumberFormat(language, { maximumFractionDigits: 5 }).format(Number(value));
}
