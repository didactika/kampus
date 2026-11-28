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

/** The words of access in the administration: the name of a role, of a value and of the rule of a decision. */

import type { RoleView } from "../../../api/types";
import type { Translate } from "../../../i18n/translator";

/** A value a role gives a permission, or none. */
export type Choice = "allow" | "deny" | "forbid" | "none";

/** The values offered for a permission of a role, in order. */
export const CHOICES: readonly Choice[] = ["allow", "deny", "forbid", "none"];

/** The icon and the tone of a value. */
export const LOOK: Readonly<Record<Choice, { readonly icon: string; readonly color: string; readonly bg: string }>> = {
  allow: { icon: "check", color: "success.main", bg: "color-mix(in srgb, var(--k-palette-success-main) 14%, transparent)" },
  deny: { icon: "close", color: "text.secondary", bg: "action.hover" },
  forbid: { icon: "block", color: "error.main", bg: "color-mix(in srgb, var(--k-palette-error-main) 12%, transparent)" },
  none: { icon: "remove", color: "text.disabled", bg: "transparent" },
};

/** `roleName(t, role)`: the name of a role of the campus in the language of the reader, or its short name. */
export function roleName(t: Translate, role: Pick<RoleView, "short_name">): string {
  const key = `access.role.${role.short_name}`;
  const text = t(key);
  return text === key ? role.short_name : text;
}

/** `valueName(t, "allow")` */
export function valueName(t: Translate, value: Choice): string {
  return t(`access.value.${value}`);
}

/** `ruleText(t, "no_allow")`: why a check decided as it did. */
export function ruleText(t: Translate, rule: string): string {
  const key = `access.rule.${rule}`;
  const text = t(key);
  return text === key ? rule : text;
}
