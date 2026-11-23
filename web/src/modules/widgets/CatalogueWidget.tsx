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


/** The catalogue of courses as a widget of a page, with a way to the page of every course. */

import { catalogueOf } from "../../features/courses/catalogue";
import { CourseCatalogue } from "../../features/courses/CourseCatalogue";
import { requestI18n } from "../../i18n/server";
import { fitting, number, text, type WidgetProps } from "./widget";

/** The sizes of page a widget sets, as "5, 10, 200"; none when empty or unreadable. */
function sizesOf(text: string | undefined): number[] | undefined {
  const sizes = (text ?? "").split(",").map((part) => Number(part.trim())).filter((size) => Number.isInteger(size) && size > 0);
  return sizes.length ? sizes : undefined;
}

/** The height of the heading, the tools and the pager of the catalogue, of a row of cards and of a row of the list. */
const CHROME = 240;
const CARD_ROW = 330;
const LIST_ROW = 72;

/** The columns of a grid of twelve that one card takes at least. */
const CARD_COLUMNS = 4;

/**
 * `courses.catalogue`: its first page holds as many courses as its size shows, as cards or as a list, up to the size
 * it sets.
 */
export async function CatalogueWidget({ settings, context, size }: WidgetProps) {
  const [{ t }, catalogue] = await Promise.all([requestI18n(), catalogueOf(context.campus)]);
  const list = text(settings, "view") === "list";
  const fits = list ? fitting(size, CHROME, LIST_ROW) : Math.max(1, Math.floor(size.w / CARD_COLUMNS)) * fitting(size, CHROME, CARD_ROW);
  return (
    <CourseCatalogue
      catalogue={catalogue}
      inAddress={false}
      heading={text(settings, "heading")}
      view={list ? "list" : "grid"}
      size={Math.min(number(settings, "size", 8), fits)}
      sizes={sizesOf(text(settings, "sizes"))}
      more={{ href: "/courses", label: t("catalogue.all") }}
    />
  );
}
