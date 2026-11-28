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
 * How the administration writes the value of a setting: on or off, the name of an option, a number with its unit, or a
 * text; and the unit a number counts in, from the end of its key (`_days`, `_hours`, `_minutes`, `_seconds`).
 */

import type { ScopeSetting } from "../../../api/types";
import type { Translate } from "../../../i18n/translator";
import { optionName } from "../names";

/** The units a number of a setting may count in. */
const UNITS = ["days", "hours", "minutes", "seconds"] as const;

/** A unit. */
export type Unit = (typeof UNITS)[number];

/** `unitOf("identity.invitation_days")`: the unit of a number, if its key says one. */
export function unitOf(key: string): Unit | null {
  return UNITS.find((unit) => key.endsWith(`_${unit}`)) ?? null;
}

/** `formatValue(t, setting, 7)`: a value as a person reads it. */
export function formatValue(t: Translate, setting: Pick<ScopeSetting, "key" | "kind">, value: unknown): string {
  const kind = setting.kind;
  if (kind.type === "boolean") return value === true ? t("admin.on") : t("admin.off");
  if (kind.type === "choice") return optionName(t, setting.key, String(value));
  if (kind.type === "integer") {
    const unit = unitOf(setting.key);
    return typeof value === "number" && unit ? t(`admin.unit.${unit}`, { n: value }) : String(value ?? "");
  }
  const text = typeof value === "string" ? value.trim() : "";
  return text ? t("admin.quoted", { text: text.length > 60 ? `${text.slice(0, 60)}…` : text }) : t("admin.empty.value");
}

/** Whether two values of a setting are the same. */
export function same(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** Why a value of a setting cannot be saved, if it cannot. */
export function problemOf(setting: Pick<ScopeSetting, "kind">, value: unknown): "range" | "length" | null {
  const kind = setting.kind;
  if (kind.type === "integer") {
    return typeof value === "number" && Number.isInteger(value) && value >= kind.min && value <= kind.max ? null : "range";
  }
  if (kind.type === "text" || kind.type === "rich_text") {
    return typeof value === "string" && [...value].length <= kind.max ? null : "length";
  }
  return null;
}
