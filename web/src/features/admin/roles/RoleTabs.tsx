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

"use client";

/** The tabs of roles and permissions: the matrix, the assignments of the campus and the explanation of a check. */

import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import Link from "next/link";

import { Icon } from "@kampus/ui";

import { useI18n } from "../../../i18n/I18nProvider";
import { ROLE_TABS, type RoleTab } from "./words";

/** The icon of each tab. */
const ICONS: Readonly<Record<RoleTab, string>> = { matrix: "grid_view", grants: "groups", explain: "help" };

/** `<RoleTabs tab="matrix" />` */
export function RoleTabs({ tab }: { readonly tab: RoleTab }) {
  const { t } = useI18n();
  return (
    <Tabs value={tab} sx={{ mb: 3 }} variant="scrollable" allowScrollButtonsMobile aria-label={t("admin.section.roles")}>
      {ROLE_TABS.map((name) => (
        <Tab
          key={name}
          value={name}
          component={Link}
          href={name === "matrix" ? "/admin/roles" : `/admin/roles?tab=${name}`}
          icon={<Icon name={ICONS[name]} size="small" />}
          iconPosition="start"
          label={t(`admin.roles.tab.${name}`)}
        />
      ))}
    </Tabs>
  );
}
