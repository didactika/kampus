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


/**
 * The widgets the shell knows how to show, by the code their module declares. Each is a server component that reads
 * what it needs for the person of the request; a widget with no entry here is skipped, so a page never breaks.
 */

import { CalendarWidget } from "./CalendarWidget";
import { CatalogueWidget } from "./CatalogueWidget";
import { GradesWidget } from "./GradesWidget";
import { InboxWidget } from "./InboxWidget";
import { OutlineWidget } from "./OutlineWidget";
import { ProgressWidget } from "./ProgressWidget";
import { SummaryWidget } from "./SummaryWidget";
import { TextWidget } from "./TextWidget";
import { WelcomeWidget } from "./WelcomeWidget";
import type { Widget } from "./widget";

/** The widgets, by code. */
export const WIDGETS: Readonly<Record<string, Widget>> = {
  "customise.welcome": WelcomeWidget,
  "customise.text": TextWidget,
  "courses.catalogue": CatalogueWidget,
  "courses.outline": OutlineWidget,
  "courses.summary": SummaryWidget,
  "progress.course": ProgressWidget,
  "records.grades": GradesWidget,
  "calendar.upcoming": CalendarWidget,
  "notify.inbox": InboxWidget,
};
