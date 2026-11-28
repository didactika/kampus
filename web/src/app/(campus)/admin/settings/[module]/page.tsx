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
 * The settings of a module, at the level asked with `?level=` (the campus by default, or the platform), among those the
 * person may change there.
 */

import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { optional } from "../../../../../api/server";
import type { ScopeSetting } from "../../../../../api/types";
import { type Level, ModuleSettings } from "../../../../../features/admin/settings/ModuleSettings";
import { currentAccount } from "../../../../../features/session/current-account";
import { requestI18n } from "../../../../../i18n/server";

/** The props of the page. */
interface ModuleProps {
  readonly params: Promise<{ readonly module: string }>;
  readonly searchParams: Promise<Record<string, string | string[] | undefined>>;
}

/** The title of the page, in the language of the request. */
export async function generateMetadata({ params }: ModuleProps): Promise<Metadata> {
  const { t } = await requestI18n();
  const { module } = await params;
  return { title: `${t(`admin.module.${decodeURIComponent(module)}`)} · ${t("admin.title")}` };
}

/** A scope of the path of the campus. */
interface Step {
  readonly id: string;
  readonly kind: string;
}

/** The settings of a module at a level. */
export default async function ModulePage({ params, searchParams }: ModuleProps) {
  const module = decodeURIComponent((await params).module);
  const asked = (await searchParams)["level"];
  const campus = (await currentAccount())!.campus;
  const path = await optional<Step[]>(`/access/scopes/${campus}/path`, []);
  const platform = path.find((s) => s.kind === "platform")?.id ?? null;
  const [atCampus, atPlatform] = await Promise.all([
    optional<ScopeSetting[]>(`/settings/scopes/${campus}`, []),
    platform ? optional<ScopeSetting[]>(`/settings/scopes/${platform}`, []) : Promise.resolve([] as ScopeSetting[]),
  ]);
  const own = (list: readonly ScopeSetting[]) => list.filter((s) => s.module === module);
  const levels: Level["kind"][] = [...(own(atCampus).length ? (["campus"] as const) : []), ...(own(atPlatform).length ? (["platform"] as const) : [])];
  if (!levels.length) notFound();
  const kind: Level["kind"] = asked === "platform" && levels.includes("platform") ? "platform" : levels[0]!;
  const level: Level = { kind, scope: kind === "platform" ? platform! : campus };
  return <ModuleSettings key={kind} module={module} level={level} levels={levels} settings={own(kind === "platform" ? atPlatform : atCampus)} />;
}
