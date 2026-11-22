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


/**
 * The icons of the page, by name. The design system and every UI module define the icons they use (inline SVG,
 * shipped with their code); a theme overrides any of them with an image of its own, by name, without touching code.
 * An override is an address of an image drawn as a mask in the colour of the text, so it follows the theme and runs
 * nothing, whatever the file holds.
 */

/** Where an icon comes from: SVG shipped with the code, or the address of an image a theme set. */
export type IconSource = { readonly svg: string } | { readonly url: string };

/** The icons shipped with the code. */
const defined = new Map<string, string>();

/** The icons a theme replaces. */
let overrides = new Map<string, string>();

/** Who is told when the icons change. */
const listeners = new Set<() => void>();

/** Tells every listener. */
function changed(): void {
  for (const listener of listeners) listener();
}

/** Defines icons shipped with code, by name; a name defined again keeps its first SVG. */
export function defineIcons(icons: Readonly<Record<string, string>>): void {
  for (const [name, svg] of Object.entries(icons)) {
    if (!defined.has(name)) defined.set(name, svg);
  }
  changed();
}

/** Replaces the overrides of the theme: each name with the address of its image; none puts back the defined icons. */
export function overrideIcons(icons: Readonly<Record<string, string>> = {}): void {
  overrides = new Map(Object.entries(icons).filter(([, url]) => /^(https?:\/\/|\/)/.test(url)));
  changed();
}

/** The source of an icon: the override of the theme, or the icon defined for it. */
export function iconOf(name: string): IconSource | undefined {
  const url = overrides.get(name);
  if (url) return { url };
  const svg = defined.get(name);
  return svg ? { svg } : undefined;
}

/** The names of every icon defined, for an editor of overrides. */
export function iconNames(): string[] {
  return [...defined.keys()].sort();
}

/** Calls back when the icons change; returns how to stop. */
export function onIconsChange(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
