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
// @package   web/src
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/** What a page needs to draw itself. */
export interface Context {
  root: HTMLElement;
  params: Record<string, string>;
}

/** A route: a path with `:name` segments and the page that draws it. */
export interface Route {
  path: string;
  page: (context: Context) => Promise<void>;
}

/** The parameters of a path against a pattern, or none when it does not match. */
export function match(pattern: string, path: string): Record<string, string> | null {
  const wanted = pattern.split("/").filter(Boolean);
  const given = path.split("/").filter(Boolean);
  if (wanted.length !== given.length) return null;
  const params: Record<string, string> = {};
  for (const [i, part] of wanted.entries()) {
    const value = given[i] ?? "";
    if (part.startsWith(":")) params[part.slice(1)] = decodeURIComponent(value);
    else if (part !== value) return null;
  }
  return params;
}

/** Moves to a path without reloading the page. */
export function navigate(path: string): void {
  history.pushState({}, "", path);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

/** Draws the page of the current path, and again when the path changes. */
export function start(root: HTMLElement, routes: Route[], notFound: Route["page"]): void {
  const draw = async (): Promise<void> => {
    for (const route of routes) {
      const params = match(route.path, location.pathname);
      if (params) {
        await route.page({ root, params });
        root.focus();
        return;
      }
    }
    await notFound({ root, params: {} });
  };
  window.addEventListener("popstate", () => void draw());
  document.addEventListener("click", (event) => {
    const link = (event.target as HTMLElement).closest<HTMLAnchorElement>("a[data-link]");
    if (link && !event.metaKey && !event.ctrlKey && event.button === 0) {
      event.preventDefault();
      navigate(link.getAttribute("href") ?? "/");
    }
  });
  void draw();
}
