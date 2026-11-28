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

/** The head of the account and its tabs, kept in the address so each one can be linked. */

import Avatar from "@mui/material/Avatar";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";

import { Icon } from "@kampus/ui";
import Link from "next/link";
import type { ReactNode } from "react";

import type { Me, Profile } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { PageHeader } from "../shell/PageHeader";
import { initialsOf, nameOf } from "./names";
import type { AccountTab } from "./tabs";

/** The props of `AccountTabs`. */
export interface AccountTabsProps {
  readonly account: Me;
  readonly profile: Profile;
  readonly tab: AccountTab;
  readonly children: ReactNode;
}

/** `<AccountTabs account={…} profile={…} tab="profile">…</AccountTabs>` */
export function AccountTabs({ account, profile, tab, children }: AccountTabsProps) {
  const { t } = useI18n();
  return (
    <>
      <PageHeader
        title={nameOf(profile, account)}
        lead={t("account.lead")}
        leading={
          <Avatar sx={{ inlineSize: 72, blockSize: 72, fontSize: "1.5rem", fontWeight: 700, backgroundImage: "var(--k-palette-gradient-primary)", color: "primary.contrastText" }}>
            {initialsOf(profile, account)}
          </Avatar>
        }
      />
      <Tabs value={tab} sx={{ mb: 3 }} aria-label={t("account.title")}>
        <Tab value="profile" component={Link} href="/account" icon={<Icon name="person" size="small" />} iconPosition="start" label={t("account.tab.profile")} />
        <Tab value="security" component={Link} href="/account?tab=security" icon={<Icon name="lock" size="small" />} iconPosition="start" label={t("account.tab.security")} />
      </Tabs>
      {children}
    </>
  );
}
