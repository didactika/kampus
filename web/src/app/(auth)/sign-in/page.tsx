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


/** The page where a person signs in: the brand and what Kampus offers on one side, the form on the other. */

import type { Metadata } from "next";
import { redirect } from "next/navigation";

import { AuthFrame } from "../../../features/session/AuthFrame";
import { currentAccount } from "../../../features/session/current-account";
import { SignInForm } from "../../../features/session/SignInForm";
import { requestI18n } from "../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("signin.title") };
}

/** Why a person is asked to sign in again, by the `changed` of the address. */
const NOTICES: Readonly<Record<string, string>> = { password: "signin.notice.password" };

/** Signs a person in; someone already signed in goes to the campus. */
export default async function SignInPage({ searchParams }: { readonly searchParams: Promise<Record<string, string | string[] | undefined>> }) {
  if (await currentAccount()) redirect("/");
  const { t } = await requestI18n();
  const changed = (await searchParams)["changed"];
  const notice = typeof changed === "string" && NOTICES[changed] ? t(NOTICES[changed]) : undefined;
  return (
    <AuthFrame>
      <SignInForm notice={notice} />
    </AuthFrame>
  );
}
