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
 * The words of the administration for what the server declares: the name of a setting, of one of its options and of
 * the module that declares it, from the texts of that module, or what the server says when a language has none.
 */

import type { ScopeSetting } from "../../api/types";
import type { Translate } from "../../i18n/translator";

/** A text of a key, or a fallback when no language has it. */
function textOr(t: Translate, key: string, fallback: string): string {
  const text = t(key);
  return text === key ? fallback : text;
}

/** `settingName(t, setting)`: its name in the language of the reader, or its summary. */
export function settingName(t: Translate, setting: Pick<ScopeSetting, "key" | "summary">): string {
  return textOr(t, setting.key, setting.summary);
}

/** `settingHint(t, setting)`: what helps fill it, when its module says. */
export function settingHint(t: Translate, setting: Pick<ScopeSetting, "key">): string | undefined {
  const key = `${setting.key}.hint`;
  const text = t(key);
  return text === key ? undefined : text;
}

/** `optionName(t, "notify.messaging", "campus")`: the name of an option, or the option itself. */
export function optionName(t: Translate, key: string, option: string): string {
  return textOr(t, `${key}.${option}`, option);
}

/** `moduleName(t, "identity")`: the name of a module in the administration, or its code. */
export function moduleName(t: Translate, module: string): string {
  return textOr(t, `admin.module.${module}`, module);
}
