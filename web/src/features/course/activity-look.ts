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


/** How activities look by their type and the state of the person on them. */

import type { StatusTone } from "@kampus/ui";

/** The tone of each state of an activity. */
export const STATE_TONES: Readonly<Record<string, StatusTone>> = {
  not_started: "neutral",
  in_progress: "info",
  done: "success",
  done_passed: "success",
  done_failed: "danger",
};

/** The icon of each type of activity; the module that declares a type adds its icon here. */
export const TYPE_ICONS: Readonly<Record<string, string>> = { "activities.page": "description", "activities.file": "attach_file" };

/** Whether a state counts as done. */
export function isDone(state: string | undefined): boolean {
  return state === "done" || state === "done_passed";
}
