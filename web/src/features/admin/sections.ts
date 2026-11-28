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
 * The map of the administration: its categories and the pages of each, in the order they are shown. The settings have
 * one page per module that declares any, so a module or a plugin that adds settings adds its page.
 */

/** A page of the administration. */
export interface AdminPage {
  readonly href: string;
  /** The key of its name. */
  readonly label: string;
  readonly icon: string;
}

/** A category of pages. */
export interface AdminCategory {
  readonly id: string;
  /** The key of its name. */
  readonly label: string;
  readonly pages: readonly AdminPage[];
}

/** The icon of the settings of each module; others take the one of settings. */
export const MODULE_ICONS: Readonly<Record<string, string>> = {
  identity: "shield_person",
  courses: "school",
  notify: "notifications",
  progress: "trending_up",
  files: "attach_file",
  privacy: "verified",
};

/** The page of the settings of a module. */
export function settingsHref(module: string): string {
  return `/admin/settings/${encodeURIComponent(module)}`;
}

/** `adminMap(["identity", "notify"])`: the categories, with a page of settings for each module given. */
export function adminMap(modules: readonly string[]): readonly AdminCategory[] {
  return [
    {
      id: "people",
      label: "admin.category.people",
      pages: [{ href: "/admin/people", label: "admin.page.people", icon: "groups" }],
    },
    {
      id: "courses",
      label: "admin.category.courses",
      pages: [{ href: "/admin/courses", label: "admin.page.courses", icon: "school" }],
    },
    {
      id: "access",
      label: "admin.category.access",
      pages: [
        { href: "/admin/roles", label: "admin.page.roles", icon: "shield" },
        { href: "/admin/roles/grants", label: "admin.page.grants", icon: "groups" },
        { href: "/admin/roles/check", label: "admin.page.check", icon: "help" },
        { href: "/admin/roles/compare", label: "admin.page.compare", icon: "grid_view" },
      ],
    },
    {
      id: "settings",
      label: "admin.category.settings",
      pages: modules.map((module) => ({ href: settingsHref(module), label: `admin.module.${module}`, icon: MODULE_ICONS[module] ?? "tune" })),
    },
  ];
}

/** The page an address belongs to: the longest page address it starts with. */
export function pageOf(map: readonly AdminCategory[], path: string): { readonly category: AdminCategory; readonly page: AdminPage } | null {
  let best: { category: AdminCategory; page: AdminPage } | null = null;
  for (const category of map) {
    for (const page of category.pages) {
      const within = path === page.href || path.startsWith(`${page.href}/`);
      if (within && (!best || page.href.length > best.page.href.length)) best = { category, page };
    }
  }
  return best;
}
