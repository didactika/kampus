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


/** What the pages of the shell read from the API. */

/** The account that is signed in. */
export interface Me {
  readonly id: string;
  readonly campus: string;
  readonly username: string;
  readonly email: string;
}

/** A category of courses. */
export interface Category {
  readonly id: string;
  readonly parent: string | null;
  readonly code: string | null;
  readonly name: string;
  readonly description: string | null;
}

/** Where a course is in its life. */
export type CourseState = "draft" | "published" | "archived" | "deletion_scheduled";

/** A course. */
export interface Course {
  readonly id: string;
  readonly category: string;
  readonly short_name: string;
  readonly full_name: string;
  readonly code: string | null;
  readonly summary: string | null;
  readonly language_code: string | null;
  readonly starts_at: string | null;
  readonly ends_at: string | null;
  readonly state: CourseState;
}

/** A section of a course. */
export interface Section {
  readonly id: string;
  readonly title: string | null;
}

/** A course module. */
export interface CourseModule {
  readonly id: string;
  readonly title: string;
  readonly type_code: string;
}

/** A grade of the caller. */
export interface Grade {
  readonly record: string;
  readonly final: string | null;
  readonly text: string | null;
}

/** A record of a course. */
export interface GradeRecord {
  readonly id: string;
  readonly title: string;
  readonly max: string;
}

/** What the course records hold. */
export interface Tree {
  readonly records: GradeRecord[];
}

/** The state of the caller on a course module. */
export interface ActivityState {
  readonly course_module: string;
  readonly state: string;
}

/** One thing a calendar shows. */
export interface CalendarItem {
  readonly kind: string;
  readonly title: string;
  readonly starts_at: string;
  readonly ends_at: string | null;
}

/** A notification of the inbox. */
export interface Notification {
  readonly id: string;
  readonly type: string;
  readonly payload: Record<string, unknown>;
  readonly created_at: string;
  readonly read_at: string | null;
}

/** A page of the inbox. */
export interface Inbox {
  readonly notifications: Notification[];
  readonly unread: number;
}

/** A widget in a region of a page, with its settings. */
export interface Placement {
  readonly region: string;
  readonly widget: string;
  readonly settings: Readonly<Record<string, unknown>>;
  readonly permission?: string | null;
}

/** A setting of a widget as its module declares it. */
export interface DeclaredSetting {
  readonly key: string;
  readonly kind:
    | { readonly type: "boolean" }
    | { readonly type: "choice"; readonly options: readonly string[] }
    | { readonly type: "integer"; readonly min: number; readonly max: number }
    | { readonly type: "text"; readonly max: number };
  readonly default: unknown;
}

/** A widget that may be placed on a page. */
export interface AvailableWidget {
  readonly code: string;
  readonly summary: string;
  readonly settings: readonly DeclaredSetting[];
}

/** The layout of a page kind in a scope, as the reader sees it. */
export interface Layout {
  readonly page_kind: string;
  readonly regions: readonly string[];
  readonly placements: readonly Placement[];
  readonly editable?: boolean;
  readonly widgets?: readonly AvailableWidget[];
}

/** An item of a navigation menu. */
export interface MenuItem {
  readonly label: string;
  readonly target: string;
  readonly children: readonly MenuItem[];
}

/** A navigation menu of a scope. */
export interface Menu {
  readonly kind: string;
  readonly items: readonly MenuItem[];
  readonly editable?: boolean;
}
