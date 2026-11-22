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
 * The rules a new password follows, as the person reads them: the ones the client can check as they type (no blank
 * characters, both copies equal) and the messages of every rule the server enforces.
 */

import { ApiError } from "../../api/problem";
import type { Translate } from "../../i18n/translator";

/** Whether a password holds a space or another blank character, which it never may. */
export function hasBlank(password: string): boolean {
  return /\s/u.test(password);
}

/** The message of each rule the server names when it rejects a new password. */
function ruleMessage(rule: string, failure: ApiError, t: Translate): string {
  switch (rule) {
    case "min_length":
      return t("password.rule.min_length", { n: failure.params["min_length"] ?? "" });
    case "max_length":
      return t("password.rule.max_length");
    case "whitespace":
      return t("password.rule.whitespace");
    case "contains_name":
      return t("password.rule.contains_name");
    default:
      return t("password.rule.other");
  }
}

/** What is wrong with a new password the server rejected, or `undefined` when the failure is of another kind. */
export function rejectedPassword(failure: unknown, t: Translate): string | undefined {
  if (!(failure instanceof ApiError) || failure.code !== "identity.password_rejected") return undefined;
  const rules = failure.rulesOf("password");
  return (rules.length > 0 ? rules : ["other"]).map((rule) => ruleMessage(rule, failure, t)).join(" ");
}
