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

/** The widget plugins of this build, by code. Written by `scripts/collect-plugins.ts`; do not edit. */

import type { WidgetPlugin } from "../host";
import { widget as widget_catalogue } from "@kampus/widget-catalogue";
import { widget as widget_course_content } from "@kampus/widget-course-content";
import { widget as widget_course_grades } from "@kampus/widget-course-grades";
import { widget as widget_course_progress } from "@kampus/widget-course-progress";
import { widget as widget_course_summary } from "@kampus/widget-course-summary";
import { widget as widget_notifications } from "@kampus/widget-notifications";
import { widget as widget_text } from "@kampus/widget-text";
import { widget as widget_upcoming } from "@kampus/widget-upcoming";
import { widget as widget_welcome } from "@kampus/widget-welcome";

/** Every widget the client renders; a placement of any other is skipped, so a page never breaks. */
export const WIDGETS: Readonly<Record<string, WidgetPlugin>> = {
  [widget_catalogue.code]: widget_catalogue,
  [widget_course_content.code]: widget_course_content,
  [widget_course_grades.code]: widget_course_grades,
  [widget_course_progress.code]: widget_course_progress,
  [widget_course_summary.code]: widget_course_summary,
  [widget_notifications.code]: widget_notifications,
  [widget_text.code]: widget_text,
  [widget_upcoming.code]: widget_upcoming,
  [widget_welcome.code]: widget_welcome,
};
