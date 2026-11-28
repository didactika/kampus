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

/** The words of the people of the administration: the name of an account, of its status and the tone it shows in. */

import type { Me } from "../../../api/types";
import type { Translate } from "../../../i18n/translator";

/** The statuses an account may be in, in the order filters offer them. */
export const STATUSES = ["active", "invited", "pending_confirmation", "suspended"] as const;

/** The changes of status an administrator makes. */
export type Transition = "suspended" | "active" | "deletion_scheduled" | "deletion_cancelled";

/** `accountName(account)`: how an account is named: its username, else its email. */
export function accountName(account: Pick<Me, "username" | "email" | "id">): string {
  return account.username ?? account.email ?? account.id;
}

/** `initials(account)`: the first letter of its name. */
export function initials(account: Pick<Me, "username" | "email" | "id">): string {
  return accountName(account).slice(0, 1).toUpperCase();
}

/** `statusName(t, "active")` */
export function statusName(t: Translate, status: string): string {
  return t(`admin.people.status.${status}`);
}

/** The tone of a status. */
export function statusTone(status: string): "success" | "info" | "warning" | "danger" | "neutral" {
  switch (status) {
    case "active":
      return "success";
    case "invited":
    case "pending_confirmation":
      return "info";
    case "suspended":
      return "warning";
    case "deleted":
      return "danger";
    default:
      return "neutral";
  }
}

/** The changes an account in a status can take, in the order a menu offers them. */
export function transitionsOf(status: string, deletionDue: boolean): readonly Transition[] {
  if (deletionDue) return ["deletion_cancelled"];
  switch (status) {
    case "active":
      return ["suspended"];
    case "invited":
    case "pending_confirmation":
      return ["active", "suspended"];
    case "suspended":
      return ["active", "deletion_scheduled"];
    default:
      return [];
  }
}

/** The tabs of an account, by the value of `?tab=`. */
export const ACCOUNT_TABS = ["profile", "roles", "references"] as const;

/** A tab of an account. */
export type AccountTab = (typeof ACCOUNT_TABS)[number];
