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


/** The account of the person: its profile, or its security with `?tab=security`. */

import type { Metadata } from "next";

import { optional, serverApi } from "../../../api/server";
import type { Profile, SecondFactor, SessionSummary } from "../../../api/types";
import { AccountTabs } from "../../../features/account/AccountTabs";
import { ProfileForm } from "../../../features/account/ProfileForm";
import { SecurityPanel } from "../../../features/account/SecurityPanel";
import { ACCOUNT_TABS } from "../../../features/account/tabs";
import { currentAccount } from "../../../features/session/current-account";
import { requestI18n } from "../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("account.title") };
}

/** The props of the page. */
interface AccountProps {
  readonly searchParams: Promise<Record<string, string | string[] | undefined>>;
}

/** The account of the person, read on the server. */
export default async function AccountPage({ searchParams }: AccountProps) {
  const asked = (await searchParams)["tab"];
  const tab = ACCOUNT_TABS.find((t) => t === asked) ?? "profile";
  const [account, profile] = [(await currentAccount())!, await serverApi<Profile>("/users/me/profile")];
  return (
    <AccountTabs account={account} profile={profile} tab={tab}>
      {tab === "profile" ? (
        <ProfileForm account={account} profile={profile} />
      ) : (
        <SecurityPanel
          sessions={await optional<SessionSummary[]>("/identity/me/sessions", [])}
          factors={await optional<SecondFactor[]>("/identity/me/second-factors", [])}
        />
      )}
    </AccountTabs>
  );
}
