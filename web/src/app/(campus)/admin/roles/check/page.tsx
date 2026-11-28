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

/** Why a person may or may not use a permission in the campus. */

import type { Metadata } from "next";

import { optional } from "../../../../../api/server";
import type { PermissionInfo, RoleView } from "../../../../../api/types";
import { Explain } from "../../../../../features/admin/roles/Explain";
import { EmptyState } from "../../../../../features/shell/EmptyState";
import { PageHeader } from "../../../../../features/shell/PageHeader";
import { currentAccount } from "../../../../../features/session/current-account";
import { requestI18n } from "../../../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: `${t("admin.page.check")} · ${t("admin.title")}` };
}

/** The page, read on the server. */
export default async function Page() {
  const campus = (await currentAccount())!.campus;
  const { t } = await requestI18n();
  const [roles, permissions] = await Promise.all([
    optional<RoleView[] | null>(`/access/scopes/${campus}/roles`, null),
    optional<PermissionInfo[]>(`/access/scopes/${campus}/permissions`, []),
  ]);
  const header = <PageHeader title={t("admin.page.check")} lead={t("admin.page.check.lead")} />;
  if (!roles) {
    return (
      <>
        {header}
        <EmptyState icon="shield" message={t("admin.roles.empty")} />
      </>
    );
  }
  return (
    <>
      {header}
      <Explain campus={campus} roles={roles} permissions={permissions} />
    </>
  );
}
