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
 * W3C design tokens: Didactika's reference palettes and the colour roles of the light and the dark scheme, read and
 * resolved so the MUI theme and the contrast checks use them.
 */

/** A group or a token of a W3C design tokens document. */
export interface TokenNode {
  readonly [key: string]: unknown;
}

/** One token, flattened: its path joined with dots, its resolved value and the token it refers to, if any. */
export interface Token {
  /** The path of the token, such as `color.primary`. */
  readonly path: string;
  /** Its type, inherited from its group when it does not say. */
  readonly type: string | undefined;
  /** Its value, with references to other tokens resolved. */
  readonly value: unknown;
  /** The path of the token its value refers to. */
  readonly alias: string | undefined;
}

/** Whether a node is a token rather than a group. */
function isToken(node: TokenNode): boolean {
  return Object.prototype.hasOwnProperty.call(node, "$value");
}

/** The reference a value makes to another token, such as `palette.teal.700` for `{palette.teal.700}`. */
function referenceOf(value: unknown): string | undefined {
  return typeof value === "string" ? /^\{([^}]+)\}$/.exec(value)?.[1] : undefined;
}

/** Every token of a document with its raw value, keyed by path. */
function collect(node: TokenNode, path: string[], type: string | undefined, out: Map<string, Token>): void {
  const own = typeof node["$type"] === "string" ? (node["$type"] as string) : type;
  if (isToken(node)) {
    const value = node["$value"];
    out.set(path.join("."), { path: path.join("."), type: own, value, alias: referenceOf(value) });
    return;
  }
  for (const [key, child] of Object.entries(node)) {
    if (!key.startsWith("$") && child !== null && typeof child === "object") {
      collect(child as TokenNode, [...path, key], own, out);
    }
  }
}

/** The paths of the tokens a document holds. */
export function paths(document: TokenNode): Set<string> {
  const out = new Map<string, Token>();
  collect(document, [], undefined, out);
  return new Set(out.keys());
}

/** The value of a token with `{path}` references resolved, refusing cycles. */
function resolve(path: string, raw: Map<string, Token>, seen: Set<string>): unknown {
  const token = raw.get(path);
  if (!token) throw new Error(`unknown token ${path}`);
  if (seen.has(path)) throw new Error(`token ${path} refers to itself`);
  if (!token.alias) return token.value;
  return resolve(token.alias, raw, new Set([...seen, path]));
}

/** The tokens of documents laid one on another, flattened and resolved. */
export function flatten(...documents: TokenNode[]): Token[] {
  const raw = new Map<string, Token>();
  for (const document of documents) collect(document, [], undefined, raw);
  return [...raw.values()].map((token) => ({ ...token, value: resolve(token.path, raw, new Set()) }));
}

/** The colour roles of a scheme document, resolved against the palettes. */
export function roles(reference: TokenNode, scheme: TokenNode): Token[] {
  const own = paths(scheme);
  return flatten(reference, scheme).filter((token) => own.has(token.path));
}
