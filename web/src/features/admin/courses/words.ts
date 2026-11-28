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

/** The words of the courses of the administration: the tone of a state and the states a course may go to. */

/** The tone a state shows in. */
export function stateTone(state: string): "success" | "info" | "warning" | "danger" | "neutral" {
  switch (state) {
    case "published":
      return "success";
    case "draft":
      return "info";
    case "archived":
      return "neutral";
    case "deletion_scheduled":
      return "danger";
    default:
      return "neutral";
  }
}

/** The states a course in a state may go to, in the order the head of its page offers them. */
export function nextStates(state: string): readonly string[] {
  switch (state) {
    case "draft":
      return ["published"];
    case "published":
      return ["archived", "draft"];
    case "archived":
      return ["published"];
    default:
      return [];
  }
}
