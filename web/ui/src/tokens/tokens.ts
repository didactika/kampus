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
 * W3C design tokens as CSS custom properties. The default theme is four documents: the reference palettes, the
 * system tokens, and the colour roles of the light and the dark scheme. Only the system tokens and the roles become
 * properties, so a theme of a scope, which sets the same properties, replaces the whole look.
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

/** The documents of a theme. */
export interface ThemeDocuments {
  /** The palettes the roles are taken from. */
  readonly reference: TokenNode;
  /** The tokens the same in both schemes. */
  readonly system: TokenNode;
  /** The colour roles of the light scheme. */
  readonly light: TokenNode;
  /** The colour roles of the dark scheme. */
  readonly dark: TokenNode;
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

/** The custom property of a token: `color.primary` is `--k-color-primary`. */
export function property(path: string): string {
  return `--k-${path.replaceAll(".", "-")}`;
}

/** The CSS value of a token: a reference to a token that is a property too stays a reference, so it follows it. */
function cssValue(token: Token, properties: Set<string>): string {
  if (token.alias && properties.has(token.alias)) return `var(${property(token.alias)})`;
  if (Array.isArray(token.value)) return token.value.map((v) => (/\s/.test(String(v)) ? `"${v}"` : String(v))).join(", ");
  return String(token.value);
}

/** Tokens as declarations of custom properties, each line indented. */
export function declarations(tokens: Token[], indent = "  ", properties = new Set<string>()): string {
  return tokens.map((token) => `${indent}${property(token.path)}: ${cssValue(token, properties)};`).join("\n");
}

/** The colour roles of a scheme document, resolved against the palettes. */
export function roles(reference: TokenNode, scheme: TokenNode): Token[] {
  const own = paths(scheme);
  return flatten(reference, scheme).filter((token) => own.has(token.path));
}

/**
 * The stylesheet of a theme: the system tokens and the light roles on the root, the dark roles when the system asks
 * for them and the page does not force light, and on `[data-theme="dark"]`.
 */
export function stylesheet({ reference, system, light, dark }: ThemeDocuments): string {
  const own = flatten(system);
  const names = new Set(own.map((token) => token.path));
  return [
    `:root {\n${declarations(own, "  ", names)}\n${declarations(roles(reference, light))}\n  color-scheme: light dark;\n}`,
    `@media (prefers-color-scheme: dark) {\n  :root:not([data-theme="light"]) {\n${declarations(roles(reference, dark), "    ")}\n  }\n}`,
    `:root[data-theme="dark"] {\n${declarations(roles(reference, dark))}\n  color-scheme: dark;\n}`,
    `:root[data-theme="light"] {\n  color-scheme: light;\n}`,
  ].join("\n\n");
}
