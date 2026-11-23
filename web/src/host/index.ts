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
 * The host API of the web client for plugins, on the server: what a plugin may use of the client, and nothing else.
 * A plugin imports the design system (`@kampus/ui`), the types of the API (`@kampus/sdk`) and this; in the browser,
 * `@kampus/web/host/client`. Whatever is not exported here is the inside of the client and may change at any time.
 */

export type { PageContext, Widget, WidgetPlugin, WidgetProps, WidgetSize } from "./widget";
export { fitting, heightOf, number, text } from "./widget";
export { translated } from "../features/editor/translated";
export { EditableRichText, EditableText } from "../features/editor/EditInPlace";

export { optional } from "../api/server";
export type { Inbox } from "../api/types";
export { requestI18n } from "../i18n/server";
export { currentAccount } from "../features/session/current-account";

export { catalogueOf } from "../features/courses/catalogue";
export { CourseCatalogue } from "../features/courses/CourseCatalogue";
export { CourseOutline } from "../features/course/CourseOutline";
export { CalendarCard, GradesCard, ProgressCard, SummaryCard } from "../features/course/CourseCards";
export { DAYS_AHEAD } from "../features/course/course-data";
export { isDone } from "../features/course/activity-look";
