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
 * What the shell reads from the API, by the names its pages use: each an alias of a schema the server publishes,
 * generated in `@kampus/sdk`, so a field the client reads is a field the server sends.
 */

import type { Schema } from "@kampus/sdk";

/** The account that is signed in. */
export type Me = Schema<"Account">;
/** A category of courses. */
export type Category = Schema<"Category">;
/** The profile of the person: names, language and time zone. */
export type Profile = Schema<"Profile">;
/** An open session of the person. */
export type SessionSummary = Schema<"SessionSummary">;
export type SecondFactor = Schema<"SecondFactor">;
/** A course. */
export type Course = Schema<"Course">;
/** A course with its owners, as one course is read. */
export type CourseWithOwners = Schema<"WithOwners">;
/** Where a course is in its life. */
export type CourseState = Schema<"CourseState">;
/** A section of a course. */
export type Section = Schema<"Section">;
/** A course module. */
export type CourseModule = Schema<"Module">;
/** A grade of the person. */
export type Grade = Schema<"Grade">;
/** What the records of a course hold. */
export type Tree = Schema<"Tree">;
/** The state of the person on a course module. */
export type ActivityState = Schema<"ActivityState">;
/** One thing a calendar shows. */
export type CalendarItem = Schema<"Item">;
/** A page of the inbox. */
export type Inbox = Schema<"Inbox">;
/** A notification of the inbox. */
export type Notification = Schema<"Notification">;
/** How the person wants a type of notification on a channel. */
export type Preference = Schema<"Preference">;
/** The layout of a page kind in a scope, as the reader sees it. */
export type Layout = Schema<"Layout">;
/** A widget in a region of a page, with its settings. */
export type Placement = Schema<"Placement">;
/** A widget that may be placed on a page. */
export type AvailableWidget = Schema<"AvailableWidget">;
/** A navigation menu of a scope. */
export type Menu = Schema<"Menu">;
/** An item of a navigation menu. */
export type MenuItem = Schema<"MenuItem">;

/** A setting of a widget as its module declares it; the API sends it as free JSON. */
export interface DeclaredSetting {
  readonly key: string;
  readonly kind:
    | { readonly type: "boolean" }
    | { readonly type: "choice"; readonly options: readonly string[] }
    | { readonly type: "integer"; readonly min: number; readonly max: number }
    | { readonly type: "text"; readonly max: number }
    | { readonly type: "rich_text"; readonly max: number };
  readonly default: unknown;
  /** Whether it may not be left empty. */
  readonly required?: boolean;
  /** Whether a text takes one value per language. */
  readonly translatable?: boolean;
}

/** A setting of a scope, as its administration reads and changes it. */
export type ScopeSetting = Schema<"ScopeSetting">;

/** The type and bounds of a setting. */
export type StoredKind = Schema<"StoredKind">;

/** A role of a campus with its values. */
export type RoleView = Schema<"RoleView">;

/** A declared permission. */
export type PermissionInfo = Schema<"PermissionInfo">;

/** A role granted in a scope. */
export type GrantView = Schema<"GrantView">;

/** The decision of a check, with its explanation. */
export type Decision = Schema<"Decision">;
