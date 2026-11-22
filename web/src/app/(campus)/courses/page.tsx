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


/** Every course of the campus: the catalogue as a page, with what the person asks kept in the address. */

import type { Metadata } from "next";

import { catalogueOf } from "../../../features/courses/catalogue";
import { CourseCatalogue } from "../../../features/courses/CourseCatalogue";
import { currentAccount } from "../../../features/session/current-account";
import { PageHeader } from "../../../features/shell/PageHeader";
import { requestI18n } from "../../../i18n/server";

/** The title of the page, in the language of the request. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  return { title: t("courses.title") };
}

/** The props of the page. */
interface CoursesProps {
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

/** The catalogue of the campus of the person. */
export default async function CoursesPage({ searchParams }: CoursesProps) {
  const account = (await currentAccount())!;
  const [{ t }, catalogue, search] = await Promise.all([requestI18n(), catalogueOf(account.campus), searchParams]);
  return (
    <>
      <PageHeader title={t("courses.title")} lead={t("courses.page.lead")} />
      <CourseCatalogue catalogue={catalogue} initial={paramsOf(search)} titleless />
    </>
  );
}
