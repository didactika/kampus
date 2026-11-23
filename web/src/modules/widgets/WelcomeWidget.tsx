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


/** The welcome of the home: a greeting in the colours of the brand with the figures of the person. */

import { optional } from "../../api/server";
import type { Inbox } from "../../api/types";
import { catalogueOf } from "../../features/courses/catalogue";
import { Welcome } from "../../features/home/Welcome";
import { currentAccount } from "../../features/session/current-account";
import { text, type WidgetProps } from "./widget";

/** The fewest rows that hold the figures under the greeting. */
const ROWS_FOR_FIGURES = 4;

/** `customise.welcome`: the greeting, and the figures of the reader when it sets them and is tall enough. */
export async function WelcomeWidget({ settings, context, size }: WidgetProps) {
  const account = await currentAccount();
  const figures = settings["figures"] !== false && size.h >= ROWS_FOR_FIGURES;
  const [catalogue, inbox] = figures
    ? await Promise.all([catalogueOf(context.campus), optional<Inbox | null>("/notify/inbox?limit=1", null)])
    : [null, null];
  return (
    <Welcome
      name={account?.username ?? ""}
      heading={text(settings, "heading")}
      figures={
        catalogue
          ? { courses: catalogue.courses.length, categories: catalogue.categories.length, unread: inbox ? inbox.unread : null }
          : null
      }
    />
  );
}
