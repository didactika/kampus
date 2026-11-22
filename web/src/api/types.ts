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
  readonly name: string;
}

/** A course. */
export interface Course {
  readonly id: string;
  readonly short_name: string;
  readonly full_name: string;
  readonly summary: string | null;
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
