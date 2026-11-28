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

/** The accounts of the campus, the first page read on the server. */

import type { Metadata } from "next";

import { optional } from "../../../../api/server";
import type { Me } from "../../../../api/types";
import { AccountsList } from "../../../../features/admin/people/AccountsList";
import { EmptyState } from "../../../../features/shell/EmptyState";
import { currentAccount } from "../../../../features/session/current-account";
import { requestI18n } from "../../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: `${t("admin.page.people")} · ${t("admin.title")}` };
}

/** The accounts of the campus of the person. */
export default async function PeoplePage() {
  const campus = (await currentAccount())!.campus;
  const { t } = await requestI18n();
  const first = await optional<{ readonly items: readonly Me[]; readonly next: string | null } | null>(`/identity/campuses/${campus}/accounts?limit=25`, null);
  if (!first) return <EmptyState icon="groups" message={t("admin.people.denied")} />;
  return <AccountsList campus={campus} first={first} />;
}
