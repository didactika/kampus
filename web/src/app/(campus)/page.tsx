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
// @package   web/ui
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/** The home of a campus: a welcome with the figures of the person, and the catalogue of courses. */

import type { Metadata } from "next";

import { optional } from "../../api/server";
import type { Inbox } from "../../api/types";
import { catalogueOf } from "../../features/courses/catalogue";
import { CourseCatalogue } from "../../features/courses/CourseCatalogue";
import { Welcome } from "../../features/home/Welcome";
import { currentAccount } from "../../features/session/current-account";
import { requestI18n } from "../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("courses.title") };
}

/** The props of the home. */
interface HomeProps {
  readonly searchParams: Promise<Record<string, string | string[] | undefined>>;
}

/** The parameters of the address as text. */
function paramsOf(search: Record<string, string | string[] | undefined>): string {
  const params = new URLSearchParams();
  for (const [name, value] of Object.entries(search)) {
    for (const one of Array.isArray(value) ? value : value === undefined ? [] : [value]) params.append(name, one);
  }
  return params.toString();
}

/** The home of the campus of the person. */
export default async function HomePage({ searchParams }: HomeProps) {
  const account = (await currentAccount())!;
  const [catalogue, inbox, search] = await Promise.all([
    catalogueOf(account.campus),
    optional<Inbox | null>("/notify/inbox?limit=1", null),
    searchParams,
  ]);
  return (
    <>
      <Welcome
        name={account.username}
        courses={catalogue.courses.length}
        categories={catalogue.categories.length}
        unread={inbox ? inbox.unread : null}
      />
      <CourseCatalogue catalogue={catalogue} initial={paramsOf(search)} />
    </>
  );
}
