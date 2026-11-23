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
 * The home of a campus: the widgets the campus arranged, each rendered on the server for the person, or the default
 * home when it arranged none. Whoever may arrange it gets the editor.
 */

import type { Metadata } from "next";

import { optional } from "../../api/server";
import type { Layout } from "../../api/types";
import { ArrangedPage } from "../../features/layouts/ArrangedPage";
import { DASHBOARD, DEFAULT_DASHBOARD, STANDARD_GRID } from "../../features/dashboard/default-layout";
import { currentAccount } from "../../features/session/current-account";
import { requestI18n } from "../../i18n/server";
import { WIDGETS } from "../../plugins/widgets.generated";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("dashboard.title") };
}

/** The home of the person: their own arrangement, or the one of their campus until they make one. */
export default async function HomePage() {
  const account = (await currentAccount())!;
  const layout = await optional<Layout | null>(`/customise/scopes/${account.scope}/layouts/${DASHBOARD}`, null);
  const placements = layout?.placements.length ? layout.placements : DEFAULT_DASHBOARD;
  const widgets = await Promise.all(
    placements.map(async (placement) => {
      const plugin = WIDGETS[placement.widget];
      return { placement, node: plugin ? await plugin.render({ settings: placement.settings, context: { campus: account.campus }, size: { w: placement.w, h: placement.h } }) : null };
    }),
  );
  return (
    <ArrangedPage
      scope={account.scope}
      pageKind={DASHBOARD}
      grid={layout ? { columns: layout.grid.columns, row: layout.grid.row_pixels } : STANDARD_GRID}
      widgets={widgets}
      editable={layout?.editable ?? false}
      available={(layout?.widgets ?? []).filter((w) => w.code in WIDGETS)}
    />
  );
}
