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
// @package   web/sdk
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/** The routes and schemas of the API. Written by `scripts/generate.ts` from `openapi.json`; do not edit. */

export type paths = {
    "/activities/campuses/{campus}/course-modules/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The course module a system knows by an identifier in a campus */
        get: operations["get_activities_campuses_campus_course_modules_by_reference_system_external_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A course module available to the caller, with its version as ETag */
        get: operations["get_activities_course_modules_id"];
        /** Replace a course module */
        put: operations["put_activities_course_modules_id"];
        post?: never;
        /** Put a course module in the recycle bin (query `permanent`) */
        delete: operations["delete_activities_course_modules_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/conditions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The access conditions of a course module */
        get: operations["get_activities_course_modules_id_conditions"];
        /** Replace the access conditions of a course module; null removes them */
        put: operations["put_activities_course_modules_id_conditions"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/copy": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Copy a course module to a section */
        post: operations["post_activities_course_modules_id_copy"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/effective-values/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The settings of a course module as they apply to a person */
        get: operations["get_activities_course_modules_id_effective_values_account"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/exceptions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The exceptions of a course module */
        get: operations["get_activities_course_modules_id_exceptions"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/exceptions/accounts/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give a person an exception to the settings of a course module */
        put: operations["put_activities_course_modules_id_exceptions_accounts_account"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/exceptions/groups/{group}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give a group an exception to the settings of a course module */
        put: operations["put_activities_course_modules_id_exceptions_groups_group"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/file": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The file of a material */
        get: operations["get_activities_course_modules_id_file"];
        /** Make an uploaded file the file of a material */
        put: operations["put_activities_course_modules_id_file"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/move": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Move a course module to another section and/or place */
        post: operations["post_activities_course_modules_id_move"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The identifiers systems give a course module */
        get: operations["get_activities_course_modules_id_references"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/references/{system}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give a course module the identifier a system uses for it */
        put: operations["put_activities_course_modules_id_references_system"];
        post?: never;
        /** Remove the identifier a system gave a course module */
        delete: operations["delete_activities_course_modules_id_references_system"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/course-modules/{id}/restore": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Restore a course module from the recycle bin */
        post: operations["post_activities_course_modules_id_restore"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/courses/{id}/bin": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The recycle bin of a course */
        get: operations["get_activities_courses_id_bin"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/exceptions/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Remove an exception */
        delete: operations["delete_activities_exceptions_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/sections/{id}/conditions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The access conditions of a section */
        get: operations["get_activities_sections_id_conditions"];
        /** Replace the access conditions of a section; null removes them */
        put: operations["put_activities_sections_id_conditions"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/sections/{id}/course-modules": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The course modules of a section, as the caller may see them */
        get: operations["get_activities_sections_id_course_modules"];
        put?: never;
        /** Add a course module of a type to a section */
        post: operations["post_activities_sections_id_course_modules"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/sections/{id}/course-modules/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Update the course module a system knows by an identifier; 201 when it was created in the section */
        put: operations["put_activities_sections_id_course_modules_by_reference_system_external_id"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/activities/types": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The types of activity and material this build provides */
        get: operations["get_activities_types"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/courses/{id}/calendar": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The calendar of a course in a window */
        get: operations["get_calendar_courses_id_calendar"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/entries": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The entries and dates the caller sees in a window (query `from`, `to`) */
        get: operations["get_calendar_entries"];
        put?: never;
        /** Add an entry to a calendar */
        post: operations["post_calendar_entries"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/entries/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** An entry, with its version as ETag */
        get: operations["get_calendar_entries_id"];
        /** Replace an entry */
        put: operations["put_calendar_entries_id"];
        post?: never;
        /** Delete an entry */
        delete: operations["delete_calendar_entries_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/entries/{id}/occurrences/{at}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Move or cancel one occurrence of a repeating entry */
        put: operations["put_calendar_entries_id_occurrences_at"];
        post?: never;
        /** Restore one occurrence of a repeating entry */
        delete: operations["delete_calendar_entries_id_occurrences_at"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/feed/{secret}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The calendar of a feed, as iCalendar */
        get: operations["get_calendar_feed_secret"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/feeds": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The calendar feeds of the caller */
        get: operations["get_calendar_feeds"];
        put?: never;
        /** Make a calendar feed; its secret address is shown once */
        post: operations["post_calendar_feeds"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/feeds/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Revoke a calendar feed */
        delete: operations["delete_calendar_feeds_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/scopes/{id}/subscriptions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The external calendars a scope subscribes to */
        get: operations["get_calendar_scopes_id_subscriptions"];
        put?: never;
        /** Subscribe a scope to an external calendar */
        post: operations["post_calendar_scopes_id_subscriptions"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/subscriptions/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Remove a subscription with its events */
        delete: operations["delete_calendar_subscriptions_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/calendar/subscriptions/{id}/refresh": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Read a subscription again now */
        post: operations["post_calendar_subscriptions_id_refresh"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/{kind}/{id}/references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The identifiers systems give a record (kind: categories, courses, sections, groups, group-sets) */
        get: operations["get_courses_kind_id_references"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/{kind}/{id}/references/{system}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give a record the identifier a system uses for it */
        put: operations["put_courses_kind_id_references_system"];
        post?: never;
        /** Remove the identifier a system gave a record */
        delete: operations["delete_courses_kind_id_references_system"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/campuses/{campus}/{kind}/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The record of a kind a system knows by an identifier in a campus */
        get: operations["get_courses_campuses_campus_kind_by_reference_system_external_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/campuses/{campus}/categories": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Categories under a parent (query `parent`, the top when absent), as the caller sees them */
        get: operations["get_courses_campuses_campus_categories"];
        put?: never;
        /** Create a category under a parent and at a place */
        post: operations["post_courses_campuses_campus_categories"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/campuses/{campus}/categories/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Update the category a system knows by an identifier; 201 when it was created last under its parent */
        put: operations["put_courses_campuses_campus_categories_by_reference_system_external_id"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/categories/{category}/courses/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Update the course a system knows by an identifier; 201 when it was created last in the category */
        put: operations["put_courses_categories_category_courses_by_reference_system_external_id"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/categories/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A category the caller sees, with its version as ETag */
        get: operations["get_courses_categories_id"];
        /** Replace the name, code, description and visibility of a category */
        put: operations["put_courses_categories_id"];
        post?: never;
        /** Delete an empty category */
        delete: operations["delete_courses_categories_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/categories/{id}/courses": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Courses of a category the caller sees */
        get: operations["get_courses_categories_id_courses"];
        put?: never;
        /** Create a course as a draft, with its first section */
        post: operations["post_courses_categories_id_courses"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/categories/{id}/move": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Move a category under another parent and/or between two siblings */
        post: operations["post_courses_categories_id_move"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A course with its owners, with its version as ETag */
        get: operations["get_courses_courses_id"];
        /** Replace the names, code, summary, layout, dates and visibility of a course */
        put: operations["put_courses_courses_id"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/fields": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The custom fields that apply to a course, with the values the caller sees */
        get: operations["get_courses_courses_id_fields"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/fields/{code}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Set the value of a custom field of a course, or clear it with null */
        put: operations["put_courses_courses_id_fields_code"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/group-sets": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The group sets of a course with their groups */
        get: operations["get_courses_courses_id_group_sets"];
        put?: never;
        /** Create a group set */
        post: operations["post_courses_courses_id_group_sets"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/groups": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The groups of a course the caller may see */
        get: operations["get_courses_courses_id_groups"];
        put?: never;
        /** Create a group, optionally limited to a section */
        post: operations["post_courses_courses_id_groups"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/move": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Move a course to another category and/or another place */
        post: operations["post_courses_courses_id_move"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/sections": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The sections of a course the caller sees, depth first */
        get: operations["get_courses_courses_id_sections"];
        put?: never;
        /** Add a section under a parent and at a place among its siblings */
        post: operations["post_courses_courses_id_sections"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/courses/{id}/state": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Publish, take back to draft, archive, unarchive or schedule deletion */
        put: operations["put_courses_courses_id_state"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/group-sets/{id}/groups/{group}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Put a group of the same course in a set */
        put: operations["put_courses_group_sets_id_groups_group"];
        post?: never;
        /** Take a group out of a set */
        delete: operations["delete_courses_group_sets_id_groups_group"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/groups/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A group the caller sees, with its version as ETag */
        get: operations["get_courses_groups_id"];
        /** Replace a group */
        put: operations["put_courses_groups_id"];
        post?: never;
        /** Delete a group */
        delete: operations["delete_courses_groups_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/groups/{id}/members": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The members of a group with their origin */
        get: operations["get_courses_groups_id_members"];
        put?: never;
        /** Add a participant to a group by hand */
        post: operations["post_courses_groups_id_members"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/groups/{id}/members/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Remove a manual membership */
        delete: operations["delete_courses_groups_id_members_account"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/sections/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A section the caller sees, with its version as ETag */
        get: operations["get_courses_sections_id"];
        /** Replace the title, summary and visibility of a section */
        put: operations["put_courses_sections_id"];
        post?: never;
        /** Delete a section with the sections in it, recoverably */
        delete: operations["delete_courses_sections_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/sections/{id}/move": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Move a section with its subtree under another parent and/or place */
        post: operations["post_courses_sections_id_move"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/courses/sections/{id}/restore": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Restore a deleted section with the sections deleted with it */
        post: operations["post_courses_sections_id_restore"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/fields/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A custom field */
        get: operations["get_customise_fields_id"];
        put?: never;
        post?: never;
        /** Delete a custom field with its values */
        delete: operations["delete_customise_fields_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/imports/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** An import with its differences */
        get: operations["get_customise_imports_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/imports/{id}/apply": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Apply a previewed import */
        post: operations["post_customise_imports_id_apply"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/rules/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A rule, with its version as ETag */
        get: operations["get_customise_rules_id"];
        /** Replace a rule */
        put: operations["put_customise_rules_id"];
        post?: never;
        /** Delete a rule */
        delete: operations["delete_customise_rules_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/rules/{id}/runs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The last runs of a rule */
        get: operations["get_customise_rules_id_runs"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/fields": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The custom fields of a kind of entity in a scope (query `kind`) */
        get: operations["get_customise_scopes_scope_fields"];
        put?: never;
        /** Create a custom field in a scope */
        post: operations["post_customise_scopes_scope_fields"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/flags": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The features of a scope, on or off */
        get: operations["get_customise_scopes_scope_flags"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/flags/{code}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A feature of a scope */
        get: operations["get_customise_scopes_scope_flags_code"];
        /** Turn a feature of a scope on or off, and lock it */
        put: operations["put_customise_scopes_scope_flags_code"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/imports": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Preview the import of a package into a scope */
        post: operations["post_customise_scopes_scope_imports"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/layouts/{page_kind}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The layout of a kind of page in a scope */
        get: operations["get_customise_scopes_scope_layouts_page_kind"];
        /** Place widgets on the grid of a kind of page */
        put: operations["put_customise_scopes_scope_layouts_page_kind"];
        post?: never;
        /** Go back to the layout the scope inherits */
        delete: operations["delete_customise_scopes_scope_layouts_page_kind"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/menus/{kind}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A navigation menu of a scope */
        get: operations["get_customise_scopes_scope_menus_kind"];
        /** Replace the items of a navigation menu */
        put: operations["put_customise_scopes_scope_menus_kind"];
        post?: never;
        /** Go back to the menu the scope inherits */
        delete: operations["delete_customise_scopes_scope_menus_kind"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/packages": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Export the customisation of a scope as a package */
        post: operations["post_customise_scopes_scope_packages"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/rules": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The rules of a scope */
        get: operations["get_customise_scopes_scope_rules"];
        put?: never;
        /** Create a rule in a scope */
        post: operations["post_customise_scopes_scope_rules"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/terms/{code}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A term of a scope in a language (query `language`) */
        get: operations["get_customise_scopes_scope_terms_code"];
        /** Rename a term in a scope */
        put: operations["put_customise_scopes_scope_terms_code"];
        post?: never;
        /** Take back the name a scope gave a term (query `language`) */
        delete: operations["delete_customise_scopes_scope_terms_code"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/theme": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The theme of a scope */
        get: operations["get_customise_scopes_scope_theme"];
        /** Set the tokens of the theme of a scope */
        put: operations["put_customise_scopes_scope_theme"];
        post?: never;
        /** Go back to the theme the scope inherits */
        delete: operations["delete_customise_scopes_scope_theme"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/customise/scopes/{scope}/theme.css": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The theme of a scope as CSS custom properties */
        get: operations["get_customise_scopes_scope_theme_css"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/campuses/{campus}/enrolments/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The enrolment a system knows by an identifier in a campus */
        get: operations["get_enrol_campuses_campus_enrolments_by_reference_system_external_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/courses/{id}/enrolment-instances": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The enrolment instances of a course */
        get: operations["get_enrol_courses_id_enrolment_instances"];
        put?: never;
        /** Add an enrolment instance to a course */
        post: operations["post_enrol_courses_id_enrolment_instances"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/courses/{id}/enrolment-requests": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The pending enrolment requests of a course */
        get: operations["get_enrol_courses_id_enrolment_requests"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/courses/{id}/participants": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The enrolments of a course (query `active`) */
        get: operations["get_enrol_courses_id_participants"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-instances/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** An enrolment instance, with its version as ETag */
        get: operations["get_enrol_enrolment_instances_id"];
        /** Replace the settings of an enrolment instance */
        put: operations["put_enrol_enrolment_instances_id"];
        post?: never;
        /** Delete an enrolment instance with its enrolments */
        delete: operations["delete_enrol_enrolment_instances_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-instances/{id}/enrolments": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Enrol people by hand, with an answer per person */
        post: operations["post_enrol_enrolment_instances_id_enrolments"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-instances/{id}/enrolments/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Update the enrolment a system knows by an identifier; 201 when it was created in the instance */
        put: operations["put_enrol_enrolment_instances_id_enrolments_by_reference_system_external_id"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-instances/{id}/keys": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Set or clear the enrolment key of a self instance */
        put: operations["put_enrol_enrolment_instances_id_keys"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-instances/{id}/self": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Enrol the caller; 202 when it becomes a request */
        post: operations["post_enrol_enrolment_instances_id_self"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-instances/{id}/sync": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Synchronise an instance with its source now */
        post: operations["post_enrol_enrolment_instances_id_sync"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-requests/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Withdraw the caller's own enrolment request */
        delete: operations["delete_enrol_enrolment_requests_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolment-requests/{id}/decision": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Approve or reject an enrolment request */
        put: operations["put_enrol_enrolment_requests_id_decision"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolments/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** An enrolment, with its version as ETag */
        get: operations["get_enrol_enrolments_id"];
        /** Change the period or status of an enrolment */
        put: operations["put_enrol_enrolments_id"];
        post?: never;
        /** Remove an enrolment */
        delete: operations["delete_enrol_enrolments_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolments/{id}/references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The identifiers systems give an enrolment */
        get: operations["get_enrol_enrolments_id_references"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/enrol/enrolments/{id}/references/{system}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give an enrolment the identifier a system uses for it */
        put: operations["put_enrol_enrolments_id_references_system"];
        post?: never;
        /** Remove the identifier a system gave an enrolment */
        delete: operations["delete_enrol_enrolments_id_references_system"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/content/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The content behind a signed link */
        get: operations["get_files_content_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/files/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A file the caller may read */
        get: operations["get_files_files_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/files/{id}/conversions/{kind}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A conversion of a file, such as its thumbnail; 202 while it is queued */
        get: operations["get_files_files_id_conversions_kind"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/files/{id}/url": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** A signed link that reads the file for a few minutes */
        post: operations["post_files_files_id_url"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/people/{id}/quota": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** How much the files of a person may take, and take */
        get: operations["get_files_people_id_quota"];
        /** Set the limit of a person; no limit with null */
        put: operations["put_files_people_id_quota"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/scopes/{scope}/quota": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** How much the files of a scope may take, and take */
        get: operations["get_files_scopes_scope_quota"];
        /** Set the limit of a scope; no limit with null */
        put: operations["put_files_scopes_scope_quota"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/uploads": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Open an upload into an area of a scope */
        post: operations["post_files_uploads"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/files/uploads/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Send the content of an open upload */
        put: operations["put_files_uploads_id"];
        /** Complete an upload into a file */
        post: operations["post_files_uploads_id"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/accounts/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** An account, with its version as ETag */
        get: operations["get_identity_accounts_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/accounts/{id}/references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The identifiers systems give an account */
        get: operations["get_identity_accounts_id_references"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/accounts/{id}/references/{system}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give an account the identifier a system uses for it */
        put: operations["put_identity_accounts_id_references_system"];
        post?: never;
        /** Remove the identifier a system gave an account */
        delete: operations["delete_identity_accounts_id_references_system"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/accounts/{id}/tokens": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Create an API token for an account; shown this once */
        post: operations["post_identity_accounts_id_tokens"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/accounts/{id}/transitions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Suspend, reactivate or delete an account */
        post: operations["post_identity_accounts_id_transitions"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/confirm": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Confirm a sign-up with its link and the password chosen */
        post: operations["post_identity_auth_confirm"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/email/confirm": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Confirm a new email with its link */
        post: operations["post_identity_auth_email_confirm"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/invitations/accept": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Accept an invitation with its link and the password chosen */
        post: operations["post_identity_auth_invitations_accept"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/oidc/{provider}/callback": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Finish a sign-in with what the provider sent back */
        get: operations["get_identity_auth_oidc_provider_callback"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/oidc/{provider}/start": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Send the browser to the provider to sign in */
        get: operations["get_identity_auth_oidc_provider_start"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/recovery": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Send a recovery link; the answer is the same whether the email has an account */
        post: operations["post_identity_auth_recovery"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/recovery/complete": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Complete a recovery with a new password */
        post: operations["post_identity_auth_recovery_complete"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/recovery/needs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** What completing a recovery needs besides its link */
        post: operations["post_identity_auth_recovery_needs"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/second-factor": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Complete a sign-in with a second factor */
        post: operations["post_identity_auth_second_factor"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/second-factor/email-code": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Send a code by email for a pending sign-in */
        post: operations["post_identity_auth_second_factor_email_code"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/sign-in": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Sign in with a credential; a second factor may follow */
        post: operations["post_identity_auth_sign_in"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/sign-out": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Close the session of the cookie */
        post: operations["post_identity_auth_sign_out"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/sign-up": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Sign up with an email; the answer is the same whether it has an account */
        post: operations["post_identity_auth_sign_up"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/auth/step-up": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Prove the password again to raise the session for sensitive changes */
        post: operations["post_identity_auth_step_up"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/campuses/{campus}/accounts": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Invite a person to a campus */
        post: operations["post_identity_campuses_campus_accounts"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/campuses/{campus}/accounts/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The account a system knows by an identifier in a campus */
        get: operations["get_identity_campuses_campus_accounts_by_reference_system_external_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/campuses/{campus}/identity-providers": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The identity providers of a campus */
        get: operations["get_identity_campuses_campus_identity_providers"];
        put?: never;
        /** Add an identity provider to a campus */
        post: operations["post_identity_campuses_campus_identity_providers"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/campuses/{campus}/service-accounts": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Create a service account for an integration */
        post: operations["post_identity_campuses_campus_service_accounts"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The account of the caller, with its version as ETag */
        get: operations["get_identity_me"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/email": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Change the email; a link to the new one confirms it */
        post: operations["post_identity_me_email"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/identity-links": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Link an identity a provider vouched for to the account */
        post: operations["post_identity_me_identity_links"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/password": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Change the password, proving the current one */
        post: operations["post_identity_me_password"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/recovery-codes": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** A new set of recovery codes, replacing the old ones */
        post: operations["post_identity_me_recovery_codes"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/second-factors": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The second factors of the caller */
        get: operations["get_identity_me_second_factors"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/second-factors/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Remove a second factor */
        delete: operations["delete_identity_me_second_factors_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/second-factors/email": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Add codes by email as a second factor */
        post: operations["post_identity_me_second_factors_email"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/second-factors/totp": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** A new secret for an authenticator app, to confirm */
        post: operations["post_identity_me_second_factors_totp"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/second-factors/totp/confirm": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Add an authenticator app with a code it shows */
        post: operations["post_identity_me_second_factors_totp_confirm"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/sessions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The open sessions of the caller */
        get: operations["get_identity_me_sessions"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/sessions/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Close a session of the caller */
        delete: operations["delete_identity_me_sessions_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/tokens": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The API tokens of the caller */
        get: operations["get_identity_me_tokens"];
        put?: never;
        /** Create an API token; shown this once */
        post: operations["post_identity_me_tokens"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/me/tokens/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Revoke an API token of the caller */
        delete: operations["delete_identity_me_tokens_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/identity/tokens/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Revoke an API token of any account */
        delete: operations["delete_identity_tokens_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/blocks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The people the caller blocked */
        get: operations["get_notify_blocks"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/blocks/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Block a person from messaging the caller */
        put: operations["put_notify_blocks_account"];
        post?: never;
        /** Unblock a person */
        delete: operations["delete_notify_blocks_account"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/campuses/{id}/defaults": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The channels a campus turns on by default */
        get: operations["get_notify_campuses_id_defaults"];
        /** Set whether a channel of a type is on by default in a campus */
        put: operations["put_notify_campuses_id_defaults"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/campuses/{id}/templates": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The templates a campus wrote for its notifications */
        get: operations["get_notify_campuses_id_templates"];
        /** Write the template of a type, channel and locale */
        put: operations["put_notify_campuses_id_templates"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/campuses/{id}/templates/{type_code}/{channel}/{locale}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Go back to the template a module ships */
        delete: operations["delete_notify_campuses_id_templates_type_code_channel_locale"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/conversations": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A page of the conversations of the caller (query `limit`, `after`) */
        get: operations["get_notify_conversations"];
        put?: never;
        /** Open a conversation with a person or a group */
        post: operations["post_notify_conversations"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/conversations/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A conversation of the caller */
        get: operations["get_notify_conversations_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/conversations/{id}/membership": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Leave a conversation */
        delete: operations["delete_notify_conversations_id_membership"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/conversations/{id}/messages": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A page of the messages of a conversation, newest first */
        get: operations["get_notify_conversations_id_messages"];
        put?: never;
        /** Send a message to a conversation */
        post: operations["post_notify_conversations_id_messages"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/conversations/{id}/mute": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Mute or unmute a conversation */
        put: operations["put_notify_conversations_id_mute"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/conversations/{id}/read": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Mark a conversation read up to its last message */
        post: operations["post_notify_conversations_id_read"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/inbox": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A page of the notifications of the caller (query `limit`, `after`) */
        get: operations["get_notify_inbox"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/inbox/{id}/read": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Mark a notification read */
        post: operations["post_notify_inbox_id_read"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/inbox/read-all": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Mark every notification of the caller read */
        post: operations["post_notify_inbox_read_all"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/messages/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Delete a message the caller sent */
        delete: operations["delete_notify_messages_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/preferences": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** How the caller wants each type of notification */
        get: operations["get_notify_preferences"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/preferences/{type_code}/{channel}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Turn a channel of a type of notification on or off */
        put: operations["put_notify_preferences_type_code_channel"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/push-key": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The public key push subscriptions are made with */
        get: operations["get_notify_push_key"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/push-subscriptions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The push subscriptions of the caller */
        get: operations["get_notify_push_subscriptions"];
        put?: never;
        /** Register a push subscription of a browser */
        post: operations["post_notify_push_subscriptions"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/notify/push-subscriptions/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Remove a push subscription */
        delete: operations["delete_notify_push_subscriptions_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/holds": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The legal holds */
        get: operations["get_privacy_holds"];
        put?: never;
        /** Place a legal hold that keeps data from deletion */
        post: operations["post_privacy_holds"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/holds/{id}/release": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Release a legal hold */
        post: operations["post_privacy_holds_id_release"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/policies": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The published policies */
        get: operations["get_privacy_policies"];
        put?: never;
        /** Publish a version of a policy */
        post: operations["post_privacy_policies"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/policies/{code}/acceptance": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Accept the current version of a policy */
        post: operations["post_privacy_policies_code_acceptance"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/policies/pending": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The policies the caller has yet to accept */
        get: operations["get_privacy_policies_pending"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/requests": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The privacy requests of the caller, or of everyone for an officer */
        get: operations["get_privacy_requests"];
        put?: never;
        /** Ask for an export or the deletion of personal data; it runs in the background */
        post: operations["post_privacy_requests"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/requests/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A privacy request */
        get: operations["get_privacy_requests_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/requests/{id}/decision": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Approve or reject a deletion */
        post: operations["post_privacy_requests_id_decision"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/requests/{id}/package": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The package of a finished export */
        get: operations["get_privacy_requests_id_package"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/requests/awaiting": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The deletions that wait for an approval */
        get: operations["get_privacy_requests_awaiting"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/retention": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** How long each category of personal data is kept */
        get: operations["get_privacy_retention"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/privacy/retention/{category}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Set how long a category of personal data is kept */
        put: operations["put_privacy_retention_category"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/exceptions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The decisions teachers took on the progress of people */
        get: operations["get_progress_course_modules_id_exceptions"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/exceptions/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Decide the state of a person in a course module */
        put: operations["put_progress_course_modules_id_exceptions_account"];
        post?: never;
        /** Clear the decision on a person */
        delete: operations["delete_progress_course_modules_id_exceptions_account"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/mark": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Mark a course module done by hand */
        put: operations["put_progress_course_modules_id_mark"];
        post?: never;
        /** Take back a mark by hand */
        delete: operations["delete_progress_course_modules_id_mark"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/rules": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The kinds of rule a course module requires to be complete */
        get: operations["get_progress_course_modules_id_rules"];
        /** Replace the kinds of rule a course module requires */
        put: operations["put_progress_course_modules_id_rules"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/states/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Where a person stands in a course module */
        get: operations["get_progress_course_modules_id_states_account"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/type-rules": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The rules of its type a course module requires, and those its type offers */
        get: operations["get_progress_course_modules_id_type_rules"];
        /** Replace the rules of its type a course module requires */
        put: operations["put_progress_course_modules_id_type_rules"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/course-modules/{id}/views": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Record that the caller viewed a course module */
        post: operations["post_progress_course_modules_id_views"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/course-progress": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The progress of every participant in a course */
        get: operations["get_progress_courses_id_course_progress"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/course-progress/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The progress of a person in a course */
        get: operations["get_progress_courses_id_course_progress_account"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/criteria": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** What completes a course */
        get: operations["get_progress_courses_id_criteria"];
        /** Replace what completes a course */
        put: operations["put_progress_courses_id_criteria"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/my-course-progress": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The progress of the caller in a course */
        get: operations["get_progress_courses_id_my_course_progress"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/my-progress": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Where the caller stands in every course module of a course */
        get: operations["get_progress_courses_id_my_progress"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/prerequisites": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The courses that must be complete before a course */
        get: operations["get_progress_courses_id_prerequisites"];
        /** Replace the prerequisites of a course */
        put: operations["put_progress_courses_id_prerequisites"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/courses/{id}/states/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Where a person stands in every course module of a course */
        get: operations["get_progress_courses_id_states_account"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/progress/criteria/{id}/approvals/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Approve a person on a criterion that needs a person */
        put: operations["put_progress_criteria_id_approvals_account"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/exports": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The exports of grades of a course */
        get: operations["get_records_courses_id_exports"];
        put?: never;
        /** Ask for an export of the grades of a course */
        post: operations["post_records_courses_id_exports"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/grader-report": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A page of the grader report of a course (query `limit`, `after`) */
        get: operations["get_records_courses_id_grader_report"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/grades/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The grades of a person in a course */
        get: operations["get_records_courses_id_grades_account"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/letters": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The letters that apply to a course, if any */
        get: operations["get_records_courses_id_letters"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/my-grades": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The grades of the caller in a course */
        get: operations["get_records_courses_id_my_grades"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/records": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The tree of categories and grade records of a course */
        get: operations["get_records_courses_id_records"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/records/categories": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Add a category to the records of a course */
        post: operations["post_records_courses_id_records_categories"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/records/items": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Add a grade record to a course */
        post: operations["post_records_courses_id_records_items"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/courses/{id}/scales": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The scales a course may use */
        get: operations["get_records_courses_id_scales"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/exports/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** An export of grades */
        get: operations["get_records_exports_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/record-categories/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Replace a category of records */
        put: operations["put_records_record_categories_id"];
        post?: never;
        /** Delete a category of records */
        delete: operations["delete_records_record_categories_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A grade record, with its version as ETag */
        get: operations["get_records_records_id"];
        /** Replace a grade record */
        put: operations["put_records_records_id"];
        post?: never;
        /** Delete a grade record */
        delete: operations["delete_records_records_id"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/formula": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The formula of a calculated record, if any */
        get: operations["get_records_records_id_formula"];
        /** Set the formula of a record */
        put: operations["put_records_records_id_formula"];
        post?: never;
        /** Remove the formula of a record */
        delete: operations["delete_records_records_id_formula"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/grades": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Set grades of a record, with what changed */
        put: operations["put_records_records_id_grades"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/grades/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The grade of a person in a record, with its history */
        get: operations["get_records_records_id_grades_account"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/grades/{account}/excluded": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Leave a grade out of the aggregations */
        put: operations["put_records_records_id_grades_account_excluded"];
        post?: never;
        /** Count a grade in the aggregations again */
        delete: operations["delete_records_records_id_grades_account_excluded"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/grades/{account}/hidden": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Hide a grade from its person until a date */
        put: operations["put_records_records_id_grades_account_hidden"];
        post?: never;
        /** Show a hidden grade */
        delete: operations["delete_records_records_id_grades_account_hidden"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/grades/{account}/lock": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Lock a grade */
        put: operations["put_records_records_id_grades_account_lock"];
        post?: never;
        /** Unlock a grade */
        delete: operations["delete_records_records_id_grades_account_lock"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/grades/{account}/override": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Override the grade of a person */
        put: operations["put_records_records_id_grades_account_override"];
        post?: never;
        /** Clear the override of a grade */
        delete: operations["delete_records_records_id_grades_account_override"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/records/{id}/lock": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Lock a grade record against changes */
        put: operations["put_records_records_id_lock"];
        post?: never;
        /** Unlock a grade record */
        delete: operations["delete_records_records_id_lock"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/scopes/{id}/letters": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The letters defined in a scope, if any */
        get: operations["get_records_scopes_id_letters"];
        put?: never;
        /** Define the letters of a scope */
        post: operations["post_records_scopes_id_letters"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/records/scopes/{id}/scales": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The scales defined in a scope */
        get: operations["get_records_scopes_id_scales"];
        put?: never;
        /** Define a scale in a scope */
        post: operations["post_records_scopes_id_scales"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/accounts/{id}/picture": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The file of the picture of an account */
        get: operations["get_users_accounts_id_picture"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/accounts/{id}/profile": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The profile of an account, with its version as ETag */
        get: operations["get_users_accounts_id_profile"];
        /** Replace the profile of an account */
        put: operations["put_users_accounts_id_profile"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/campuses/{campus}/system-groups/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The group a system knows by an identifier in a campus */
        get: operations["get_users_campuses_campus_system_groups_by_reference_system_external_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/me/picture": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Set the picture of the caller from an uploaded file */
        put: operations["put_users_me_picture"];
        post?: never;
        /** Remove the picture of the caller */
        delete: operations["delete_users_me_picture"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/me/preferences": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The preferences of the caller, with their defaults */
        get: operations["get_users_me_preferences"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        /** Set some preferences of the caller; null clears one */
        patch: operations["patch_users_me_preferences"];
        trace?: never;
    };
    "/users/me/profile": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The profile of the caller, with its version as ETag */
        get: operations["get_users_me_profile"];
        /** Replace the profile of the caller */
        put: operations["put_users_me_profile"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/scopes/{scope}/system-groups": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Create a system group in a scope */
        post: operations["post_users_scopes_scope_system_groups"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/scopes/{scope}/system-groups/by-reference/{system}/{external_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** The group a system knows by an identifier; 201 when it was created in the scope */
        put: operations["put_users_scopes_scope_system_groups_by_reference_system_external_id"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/system-groups/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** A system group */
        get: operations["get_users_system_groups_id"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/system-groups/{id}/apply-rule": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Apply the rule of a group to its members now */
        post: operations["post_users_system_groups_id_apply_rule"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/system-groups/{id}/members": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The members of a system group */
        get: operations["get_users_system_groups_id_members"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/system-groups/{id}/members/{account}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Add a member by hand */
        put: operations["put_users_system_groups_id_members_account"];
        post?: never;
        /** Remove a member */
        delete: operations["delete_users_system_groups_id_members_account"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/system-groups/{id}/references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** The identifiers systems give a group */
        get: operations["get_users_system_groups_id_references"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/users/system-groups/{id}/references/{system}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Give a group the identifier a system uses for it */
        put: operations["put_users_system_groups_id_references_system"];
        post?: never;
        /** Remove the identifier a system gave a group */
        delete: operations["delete_users_system_groups_id_references_system"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
};
export type webhooks = Record<string, never>;
export type components = {
    schemas: {
        /** @description An account as read. */
        Account: {
            /**
             * Format: uuid
             * @description The campus it belongs to.
             */
            campus: string;
            /**
             * Format: date-time
             * @description When a scheduled deletion runs.
             */
            deletion_due_at: string | null;
            /** @description The email, lower-case; absent on a tombstone and optional for services. */
            email: string | null;
            /**
             * Format: date-time
             * @description When the email was proven.
             */
            email_verified_at: string | null;
            /**
             * Format: uuid
             * @description The account, which is also its principal.
             */
            id: string;
            /** @description Person or service. */
            kind: components["schemas"]["AccountKind"];
            /**
             * Format: date-time
             * @description When the account last signed in.
             */
            last_sign_in_at: string | null;
            /** @description A new email waiting for its confirmation. */
            pending_email: string | null;
            /** @description The pseudonym of a tombstone. */
            pseudonym: string | null;
            /**
             * Format: uuid
             * @description The user scope of the account.
             */
            scope: string;
            /** @description The status. */
            status: components["schemas"]["AccountStatus"];
            /** @description Why the status was last changed. */
            status_reason: string | null;
            /** @description The username, lower-case; absent on a tombstone. */
            username: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description Who an account is for. */
        AccountKind: "person" | "service";
        /** @description The status of an account, a state machine. */
        AccountStatus: "invited" | "pending_confirmation" | "active" | "suspended" | "deleted";
        /** @description The state of a person on a course module. */
        ActivityState: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /**
             * Format: date-time
             * @description Since when.
             */
            at: string;
            /**
             * Format: uuid
             * @description The course module.
             */
            course_module: string;
            /** @description Where it came from. */
            source: components["schemas"]["Source"];
            /** @description Their state. */
            state: components["schemas"]["State"];
            /**
             * Format: int32
             * @description The version of the row.
             */
            version: number;
        };
        /** @description A type of activity or material, as registered. */
        ActivityType: {
            /** @description Its code. */
            code: string;
            /** @description Whether course modules of it may be added. */
            enabled: boolean;
            /** @description What it supports. */
            features: string[];
            /** @description `activity` or `material`. */
            kind: string;
            /** @description What it is. */
            summary: string;
        };
        /** @description An added second factor. */
        Added: {
            /**
             * Format: uuid
             * @description The factor.
             */
            id: string;
            /** @description A new set of recovery codes, shown this once, when the account had none. */
            recovery_codes: string[] | null;
        };
        /** @description How many members a group has after its rule was applied. */
        Applied: {
            /**
             * Format: uint
             * @description The members.
             */
            members: number;
        };
        /** @description A widget that may be placed on a page kind. */
        AvailableWidget: {
            /** @description Its code. */
            code: string;
            /** @description Its icon, by the name the design system gives it. */
            icon: string;
            /** @description Whether every page holds it, at one size and never removed. */
            required: boolean;
            /** @description Its settings as declared: key, kind and default. */
            settings: unknown[];
            /** @description Whether a page holds it once at most. */
            single: boolean;
            /** @description The sizes it takes on the page kind. */
            sizes: components["schemas"]["WidgetSizes"];
            /** @description What it shows. */
            summary: string;
        };
        /** @description One band of a letter set: from this percentage up, the grade is this letter. */
        Band: {
            /** @description The letter. */
            letter: string;
            /** @description The lowest percentage of the band. */
            min_percent: string;
        };
        /** @description A course module in the recycle bin. */
        BinItem: {
            /**
             * Format: date-time
             * @description When it was deleted.
             */
            deleted_at: string;
            /** @description Why removing it failed, which keeps it in the bin until someone retries or restores it. */
            error: string | null;
            /**
             * Format: date-time
             * @description When it is removed for good.
             */
            expires_at: string;
            /**
             * Format: uuid
             * @description The course module.
             */
            module: string;
            /** @description Its title. */
            title: string;
            /** @description Its type. */
            type: string;
        };
        /** @description What a campus sets for a type on a channel. */
        CampusDefault: {
            /** @description The channel. */
            channel: string;
            /** @description Whether people receive it. */
            enabled: boolean;
            /**
             * @description Whether people may change it.
             * @default false
             */
            locked: boolean;
            /** @description The type. */
            type: string;
        };
        /** @description A course category. */
        Category: {
            /**
             * Format: uuid
             * @description Its campus.
             */
            campus: string;
            /** @description Its code, unique in the campus when set. */
            code: string | null;
            /** @description Its description. */
            description: string | null;
            /**
             * Format: uuid
             * @description The category.
             */
            id: string;
            /** @description Its name. */
            name: string;
            /**
             * Format: uuid
             * @description Its parent; the campus itself when absent.
             */
            parent: string | null;
            /**
             * Format: uuid
             * @description Its scope.
             */
            scope: string;
            /** @description Its place among its siblings. */
            sort_key: string;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Its own visibility. */
            visibility: components["schemas"]["Visibility"];
        };
        /** @description A node of the tree of records of a course, which aggregates its children. */
        Category2: {
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /**
             * Format: int32
             * @description How many of the lowest grades it drops.
             */
            drop_lowest: number;
            /** @description Whether an empty grade is left out instead of counted as zero. */
            exclude_empty: boolean;
            /**
             * Format: uuid
             * @description The category.
             */
            id: string;
            /**
             * Format: int32
             * @description How many of the highest grades it keeps; zero keeps all.
             */
            keep_highest: number;
            /** @description How it aggregates. */
            method: components["schemas"]["Method2"];
            /** @description Its name. */
            name: string;
            /**
             * Format: uuid
             * @description Its parent; none for the root of the course.
             */
            parent: string | null;
            /** @description Its place among its siblings. */
            sort_key: string;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description A choice of a custom field. */
        Choice: {
            /** @description Its code, stable while its label changes. */
            code: string;
            /** @description What it says. */
            label: string;
        };
        /** @description A comparison of a field of the payload of the event. */
        Comparison: {
            /** @description A field the event type declares. */
            field: string;
            /** @description How it is compared. */
            op: components["schemas"]["Operator2"];
            /**
             * @description What it is compared with; a list for `in`, nothing for `exists`.
             * @default null
             */
            value: unknown;
        };
        /** @description When a rule runs: a tree of comparisons. */
        Condition: {
            all: components["schemas"]["Condition"][];
        } | {
            any: components["schemas"]["Condition"][];
        } | {
            not: components["schemas"]["Condition"];
        } | {
            compare: components["schemas"]["Comparison"];
        };
        /** @description A node of a tree of access conditions. */
        ConditionNode: {
            /** @description Set on a leaf whose kind is unknown or that lost what it depends on: it never holds. */
            broken?: boolean;
            /** @description Its children, for `all`, `any` and `not`. */
            children?: components["schemas"]["ConditionNode"][];
            /** @description The kind of a leaf, such as `activities.date`. */
            kind?: string | null;
            /** @description How it combines its children, or `leaf`. */
            operator: components["schemas"]["Operator"];
            /** @description The parameters of a leaf, as its kind takes them. */
            params?: unknown;
        };
        /** @description The access conditions of a course module or a section. */
        Conditions: {
            /**
             * @description Whether people who do not meet them still see it listed, without its content.
             * @default true
             */
            show_when_unmet: boolean;
            /** @description The tree. */
            tree: components["schemas"]["ConditionNode"];
        };
        /** @description A conversation between people: two, a group someone gathered, or the members of a group of a course. */
        Conversation: {
            /**
             * Format: uuid
             * @description The group of a course it belongs to.
             */
            course_group: string | null;
            /**
             * Format: uuid
             * @description The conversation.
             */
            id: string;
            /** @description `direct`, `group` or `course_group`. */
            kind: string;
            /**
             * Format: date-time
             * @description When its last message was sent, if any.
             */
            last_message_at: string | null;
            /** @description Who takes part now. */
            members: string[];
            /** @description Whether the person turned its notifications off. */
            muted: boolean;
            /** @description Its title, for a group. */
            title: string | null;
            /**
             * Format: int64
             * @description How many messages the person has not read.
             */
            unread: number;
        };
        /** @description Where a conversion of a file stands. */
        Conversion: {
            /** @description Why it failed. */
            error: string | null;
            /**
             * Format: date-time
             * @description When the link stops working.
             */
            expires_at: string | null;
            /** @description What it converts into. */
            kind: string;
            /** @description The media type of the result, once done. */
            media_type: string | null;
            /** @description `queued`, `done` or `failed`. */
            state: string;
            /** @description A link that reads the result for a few minutes, once done. */
            url: string | null;
        };
        /** @description A course. */
        Course: {
            /**
             * Format: uuid
             * @description Its campus.
             */
            campus: string;
            /**
             * Format: uuid
             * @description Its category.
             */
            category: string;
            /** @description Its code, unique in the campus when set. */
            code: string | null;
            /**
             * Format: date-time
             * @description When a scheduled deletion runs.
             */
            deletion_due_at: string | null;
            /**
             * Format: date-time
             * @description When it ends.
             */
            ends_at: string | null;
            /** @description Its full name. */
            full_name: string;
            /**
             * Format: uuid
             * @description The course.
             */
            id: string;
            /** @description The language it is forced to, when set. */
            language_code: string | null;
            /** @description The layout plugin that presents it. */
            layout: string;
            /**
             * Format: uuid
             * @description Its scope.
             */
            scope: string;
            /** @description Its short name, unique in the campus. */
            short_name: string;
            /** @description Its place in its category. */
            sort_key: string;
            /**
             * Format: date-time
             * @description When it starts.
             */
            starts_at: string | null;
            /** @description Its lifecycle state. */
            state: components["schemas"]["CourseState"];
            /** @description Its summary. */
            summary: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Its own visibility. */
            visibility: components["schemas"]["Visibility"];
        };
        /** @description What a course requires and how. */
        CourseCriteria: {
            /**
             * @description `all` of the criteria, or `any` of them.
             * @default all
             */
            aggregation: string;
            /** @description The criteria, in order. */
            criteria: components["schemas"]["Criterion"][];
            /**
             * @description Whether a completion stays when a criterion stops being met.
             * @default false
             */
            final: boolean;
        };
        /** @description Where a person stands in a course. */
        CourseProgress: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /**
             * Format: date-time
             * @description When they completed it.
             */
            completed_at: string | null;
            /**
             * Format: uuid
             * @description The course.
             */
            course: string;
            /** @description Each criterion with whether they meet it. */
            criteria: components["schemas"]["CriterionState"][];
            /** @description `in_progress` or `completed`; `not_tracked` for someone progress does not follow there. */
            state: string;
        };
        /** @description The lifecycle state of a course. */
        CourseState: "draft" | "published" | "archived" | "deletion_scheduled";
        /** @description One thing a course requires. */
        Criterion: {
            /** @constant */
            kind: "activity";
            /**
             * Format: uuid
             * @description The course module.
             */
            module: string;
            /** @description `done` or `passed`. */
            required: string;
        } | {
            /** @constant */
            kind: "grade";
            /** @description The share, from 0 to 100. */
            min_percent: string;
            /**
             * Format: uuid
             * @description The grade record.
             */
            record: string;
        } | {
            /** @constant */
            kind: "date";
            /**
             * Format: date-time
             * @description From when it holds.
             */
            on_or_after: string;
        } | {
            /**
             * Format: int32
             * @description The days.
             */
            enrolled_days: number;
            /** @constant */
            kind: "duration";
        } | {
            /** @constant */
            kind: "role_approval";
            /**
             * Format: uuid
             * @description The role.
             */
            role: string;
        } | {
            /**
             * Format: uuid
             * @description The other course.
             */
            course: string;
            /** @constant */
            kind: "course";
        };
        /** @description A criterion with whether a person meets it. */
        CriterionState: {
            /**
             * Format: uuid
             * @description The criterion.
             */
            id: string;
            /** @description Whether the person meets it now. */
            met: boolean;
        } & ({
            /** @constant */
            kind: "activity";
            /**
             * Format: uuid
             * @description The course module.
             */
            module: string;
            /** @description `done` or `passed`. */
            required: string;
        } | {
            /** @constant */
            kind: "grade";
            /** @description The share, from 0 to 100. */
            min_percent: string;
            /**
             * Format: uuid
             * @description The grade record.
             */
            record: string;
        } | {
            /** @constant */
            kind: "date";
            /**
             * Format: date-time
             * @description From when it holds.
             */
            on_or_after: string;
        } | {
            /**
             * Format: int32
             * @description The days.
             */
            enrolled_days: number;
            /** @constant */
            kind: "duration";
        } | {
            /** @constant */
            kind: "role_approval";
            /**
             * Format: uuid
             * @description The role.
             */
            role: string;
        } | {
            /**
             * Format: uuid
             * @description The other course.
             */
            course: string;
            /** @constant */
            kind: "course";
        });
        /** @description A grade with the entries that made it. */
        Detail: {
            /** @description Its history, oldest first. */
            entries: components["schemas"]["Entry2"][];
            /** @description The grade. */
            grade: components["schemas"]["Grade"];
        };
        /** @description What an import would do to one customisation of the scope. */
        Difference: {
            /** @description What the scope will set. */
            after: unknown;
            /** @description What the scope sets now; nothing when the import adds it. */
            before: unknown;
            /** @description What it is, such as the code of a term and its language. */
            key: string;
            /** @description The kind of customisation. */
            kind: components["schemas"]["PackageKind"];
        };
        /** @description The answer for one person enrolled by hand: the enrolment, or the code of why it failed. */
        Enrolled: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /**
             * Format: uuid
             * @description The enrolment, when it was made.
             */
            enrolment?: string | null;
            /** @description The code of the problem, when it failed. */
            error?: string | null;
        };
        /** @description A person's enrolment through an instance. */
        Enrolment: {
            /**
             * Format: uuid
             * @description The account.
             */
            account: string;
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /**
             * Format: uuid
             * @description Who created it, when a person did.
             */
            created_by: string | null;
            /**
             * Format: date-time
             * @description When it ends; open when absent.
             */
            ends_at: string | null;
            /**
             * Format: uuid
             * @description The enrolment.
             */
            id: string;
            /**
             * Format: uuid
             * @description Its instance.
             */
            instance: string;
            /** @description Why, as told by who made the last change. */
            reason: string | null;
            /**
             * Format: date-time
             * @description When it starts.
             */
            starts_at: string;
            /** @description Its status. */
            status: components["schemas"]["Status"];
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description An entry of a scope, or a personal one. */
        Entry: {
            /** @description Whether it lasts whole days, from midnight to midnight in its time zone. */
            all_day: boolean;
            /** @description What it is about. */
            description: string | null;
            /**
             * Format: date-time
             * @description When it ends.
             */
            ends_at: string;
            /**
             * Format: uuid
             * @description The group of the course it is for; only its members and who manages the calendar see it.
             */
            group: string | null;
            /**
             * Format: uuid
             * @description The entry.
             */
            id: string;
            /**
             * Format: uuid
             * @description The person a personal entry belongs to.
             */
            owner: string | null;
            /** @description Its recurrence rule, RFC 5545 (`FREQ=WEEKLY;BYDAY=MO`), for an entry that repeats. */
            recurrence: string | null;
            /**
             * Format: uuid
             * @description The scope it belongs to: a campus, a category or a course; none for a personal entry.
             */
            scope: string | null;
            /**
             * Format: date-time
             * @description When it starts.
             */
            starts_at: string;
            /** @description The time zone its recurrence and all-day dates follow. */
            time_zone: string;
            /** @description Its title. */
            title: string;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description An immutable change to the grade of a person. */
        Entry2: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /**
             * Format: uuid
             * @description Who made it, when a person did.
             */
            actor: string | null;
            /**
             * Format: date-time
             * @description When.
             */
            at: string;
            /** @description What the grader said about it. */
            feedback: string | null;
            /**
             * Format: uuid
             * @description The entry.
             */
            id: string;
            /** @description The grade it set. */
            raw: string | null;
            /**
             * Format: uuid
             * @description The record.
             */
            record: string;
            /** @description Where it came from. */
            source: components["schemas"]["Source2"];
            /** @description The text it set. */
            text: string | null;
        };
        /** @description The kind of an error, which decides how every client handles it. */
        ErrorKind: "not_found" | "unauthenticated" | "permission_denied" | "validation" | "conflict" | "precondition_failed" | "rate_limited" | "unavailable" | "internal";
        /** @description An exception of a course module. */
        Exception: {
            /**
             * Format: int32
             * @description How many attempts a person has.
             * @default null
             */
            attempts: number | null;
            /**
             * Format: date-time
             * @description When it closes.
             * @default null
             */
            closes_at: string | null;
            /**
             * Format: date-time
             * @description The last moment a late one is accepted.
             * @default null
             */
            cutoff_at: string | null;
            /**
             * Format: date-time
             * @description When it is due.
             * @default null
             */
            due_at: string | null;
            /**
             * Format: uuid
             * @description The exception.
             */
            id: string;
            /**
             * Format: uuid
             * @description The course module.
             */
            module: string;
            /**
             * Format: date-time
             * @description When it opens.
             * @default null
             */
            opens_at: string | null;
            /** @description Why it was granted. */
            reason: string | null;
            /** @description Whom it is for. */
            target: components["schemas"]["Target"];
            /**
             * Format: int32
             * @description The time a person has once started, in seconds.
             * @default null
             */
            time_limit_seconds: number | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /**
         * @description A teacher's decision about the state of a person on a course module, which wins over the rules until it
         *     is removed.
         */
        Exception2: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /**
             * Format: date-time
             * @description When.
             */
            at: string;
            /**
             * Format: uuid
             * @description Who decided it.
             */
            author: string | null;
            /**
             * Format: uuid
             * @description The course module.
             */
            course_module: string;
            /** @description Why. */
            reason: string;
            /** @description The state it sets. */
            state: components["schemas"]["State"];
        };
        /** @description What happens to an enrolment when its period ends. */
        ExpiryAction: "suspend" | "remove" | "keep_without_role";
        /** @description An export of the grades of a course, as a file for the person who asked. */
        Export: {
            /**
             * Format: date-time
             * @description When it was made.
             */
            completed_at: string | null;
            /**
             * Format: uuid
             * @description The course.
             */
            course: string;
            /**
             * Format: date-time
             * @description When it was asked for.
             */
            created_at: string;
            /** @description Why it failed. */
            error: string | null;
            /**
             * Format: date-time
             * @description When the file goes.
             */
            expires_at: string | null;
            /**
             * Format: uuid
             * @description The file, while it is available.
             */
            file: string | null;
            /**
             * Format: uuid
             * @description The export.
             */
            id: string;
            /** @description Whether it also has the people who no longer take part. */
            include_inactive: boolean;
            /** @description `pending`, `completed`, `failed` or `expired`. */
            state: string;
        };
        /** @description A package as exported, with the checksum of its contents. */
        Exported: {
            /** @description The SHA-256 of the package as written, in hexadecimal. */
            checksum: string;
            /** @description The package. */
            package: components["schemas"]["Package"];
        };
        /** @description The identifier a system gives a record. */
        ExternalReference: {
            /** @description The identifier the system gives the record. */
            external_id: string;
            /** @description The system, such as `sis` or `lms`: lowercase letters, digits, `_`, `-` and `.`. */
            system: string;
        };
        /** @description The kind of a second factor. */
        FactorKind: "totp" | "passkey" | "email";
        /** @description A personal feed: a secret address a calendar application reads what its owner sees from. */
        Feed: {
            /**
             * Format: date-time
             * @description When it was made.
             */
            created_at: string;
            /**
             * Format: uuid
             * @description The feed.
             */
            id: string;
            /**
             * Format: date-time
             * @description When an application last read it.
             */
            last_used_at: string | null;
            /** @description What its owner calls it. */
            name: string;
            /**
             * Format: date-time
             * @description When it was revoked.
             */
            revoked_at: string | null;
        };
        /** @description A custom field of a kind of entity, defined in a scope. */
        Field: {
            /** @description Its choices, in their order, for choice fields. */
            choices: components["schemas"]["Choice"][];
            /** @description Its code, unique for its kind and scope. */
            code: string;
            /**
             * Format: uuid
             * @description The field.
             */
            id: string;
            /** @description The kind of entity it belongs to. */
            kind: string;
            /**
             * Format: int32
             * @description The longest text, for text fields.
             */
            max_length: number | null;
            /** @description Its name. */
            name: string;
            /** @description Whether every entity has a value. */
            required: boolean;
            /**
             * Format: uuid
             * @description The scope that defines it; it applies there and below.
             */
            scope: string;
            /** @description Its type. */
            type: components["schemas"]["FieldType"];
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Who sees its values. */
            visibility: components["schemas"]["FieldVisibility"];
        };
        /** @description A custom field with its value for one entity. */
        FieldEntry: {
            /** @description The field. */
            field: components["schemas"]["Field"];
            /**
             * @description Its value: a string, a number, a date, a boolean, a choice code, a list of choice codes, or an
             *     account; null when absent.
             */
            value: unknown;
        };
        /** @description One failing field of a validation error: which field, and which rule it breaks. */
        FieldError: {
            /** @description The field, as the caller named it (`limit`, `profile.time_zone`). */
            field: string;
            /** @description The rule it breaks (`required`, `range`, `format`). */
            rule: string;
        };
        /** @description The type of a custom field. */
        FieldType: "text" | "integer" | "date" | "boolean" | "choice" | "choices" | "account";
        /** @description Who sees the value of a custom field. */
        FieldVisibility: "public" | "private";
        /** @description A file: content stored once, used under a name in an area. */
        File: {
            /** @description The area it belongs to. */
            area: string;
            /** @description Whether a record owns it; until then only its uploader reads it. */
            attached: boolean;
            /**
             * Format: date-time
             * @description When it was uploaded.
             */
            created_at: string;
            /**
             * Format: uuid
             * @description The file.
             */
            id: string;
            /** @description Its media type, read from its content. */
            media_type: string;
            /** @description Its name. */
            name: string;
            /**
             * Format: uuid
             * @description The scope it was uploaded for.
             */
            scope: string;
            /**
             * Format: int64
             * @description Its size in bytes.
             */
            size: number;
            /**
             * Format: uuid
             * @description Who uploaded it, while their account exists.
             */
            uploader: string | null;
        };
        /** @description Whether a feature is on in a scope. */
        Flag: {
            /** @description The feature. */
            code: string;
            /** @description Whether it is on. */
            enabled: boolean;
            /** @description Where the value came from. */
            origin: components["schemas"]["Origin"];
        };
        /** @description The formula of a record. */
        Formula: {
            /** @description The records it reads. */
            depends_on: string[];
            /** @description The expression, with the records it reads written `[[<record id>]]`. */
            expression: string;
            /**
             * Format: uuid
             * @description The record it computes.
             */
            record: string;
        };
        /** @description The record a system knows by an identifier. */
        Found: {
            /**
             * Format: uuid
             * @description Its id.
             */
            id: string;
            /** @description Its kind. */
            kind: components["schemas"]["RecordKind"];
        };
        /** @description The current grade of a person on a record. */
        Grade: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /** @description Whether it is left out of the totals. */
            excluded: boolean;
            /** @description What counts: the override when there is one, otherwise the raw grade after the multiplier and offset. */
            final: string | null;
            /**
             * Format: date-time
             * @description Hidden until then.
             */
            hidden_until: string | null;
            /**
             * Format: date-time
             * @description Locked since then.
             */
            locked_at: string | null;
            /** @description Whether a teacher overrode it. */
            overridden: boolean;
            /** @description What was graded, as given. */
            raw: string | null;
            /**
             * Format: uuid
             * @description The record.
             */
            record: string;
            /** @description The text of a text grade. */
            text: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description What happened to a grade a source wrote. */
        GradeChange: {
            /** @description Whether it changed anything; the same grade sent twice changes nothing. */
            changed: boolean;
            /** @description The grade after it. */
            grade: components["schemas"]["Grade"];
        };
        /** @description What a grade is made of. */
        GradeType: "value" | "scale" | "text";
        /** @description A group of a course. */
        Group: {
            /** @description Its code, unique in the course when set. */
            code: string | null;
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /** @description Its description. */
            description: string | null;
            /**
             * Format: uuid
             * @description The group.
             */
            id: string;
            /** @description Its name, unique in the course. */
            name: string;
            /**
             * Format: uuid
             * @description The section it is limited to.
             */
            section: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Who sees it. */
            visibility: components["schemas"]["GroupVisibility"];
        };
        /** @description How the members of a system group are chosen. */
        GroupKind: "Manual" | {
            Rule: components["schemas"]["GroupRule"];
        };
        /** @description The rule of a system group: the people of its campus whose profile field equals a value. */
        GroupRule: {
            /** @description The value it must equal. */
            equals: string;
            /** @description The field compared. */
            field: components["schemas"]["RuleField"];
        };
        /** @description A group set of a course. */
        GroupSet: {
            /** @description Its code. */
            code: string | null;
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /** @description Its groups. */
            groups: string[];
            /**
             * Format: uuid
             * @description The set.
             */
            id: string;
            /** @description Its name, unique in the course. */
            name: string;
        };
        /** @description Who sees a group and its members. */
        GroupVisibility: "everyone" | "members" | "own_only" | "nobody";
        /** @description A suspension of deletion for an account, or for a whole campus. */
        Hold: {
            /**
             * Format: uuid
             * @description The account it protects; none for a hold on the whole campus.
             */
            account: string | null;
            /**
             * Format: date-time
             * @description When it was placed.
             */
            created_at: string;
            /**
             * Format: uuid
             * @description The hold.
             */
            id: string;
            /** @description Why deletion waits. */
            reason: string;
            /**
             * Format: date-time
             * @description When it was released by hand.
             */
            released_at: string | null;
            /**
             * Format: date-time
             * @description When it ends by itself, when it does.
             */
            until: string | null;
        };
        /** @description The answer that names what an operation created or found by its identifier: `{ "id": ... }`. */
        Identified: {
            /**
             * Format: uuid
             * @description The identifier.
             */
            id: string;
        };
        /** @description The answer that names what an operation created or found by its identifier: `{ "id": ... }`. */
        Identified2: {
            /**
             * Format: uuid
             * @description The identifier.
             */
            id: string;
        };
        /** @description The answer that names what an operation created or found by its identifier: `{ "id": ... }`. */
        Identified3: {
            /**
             * Format: uuid
             * @description The identifier.
             */
            id: string;
        };
        /** @description An import of a package into a scope. */
        Import: {
            /**
             * Format: date-time
             * @description When it was applied, if it was.
             */
            applied_at: string | null;
            /** @description The checksum of the package. */
            checksum: string;
            /** @description What it adds or changes; what the scope already sets the same way is left out. */
            differences: components["schemas"]["Difference"][];
            /**
             * Format: uuid
             * @description The import.
             */
            id: string;
            /**
             * Format: uuid
             * @description The scope it goes into.
             */
            scope: string;
        };
        /** @description A page of the inbox. */
        Inbox: {
            /**
             * Format: uuid
             * @description Where the next page starts, when there is one.
             */
            next: string | null;
            /** @description The notifications, newest first. */
            notifications: components["schemas"]["Notification"][];
            /**
             * Format: int64
             * @description How many of all of them the person has not read.
             */
            unread: number;
        };
        /** @description A method configured in a course. */
        Instance: {
            /**
             * Format: int32
             * @description How many enrolments it holds at most.
             */
            capacity: number | null;
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /**
             * Format: int32
             * @description How many days an enrolment lasts when it gives no end; open when absent.
             */
            default_days: number | null;
            /** @description Whether it gives participation. */
            enabled: boolean;
            /** @description What happens when an enrolment ends. */
            expiry_action: components["schemas"]["ExpiryAction"];
            /** @description Whether joining needs a key. */
            has_key: boolean;
            /**
             * Format: uuid
             * @description The instance.
             */
            id: string;
            /** @description Its method. */
            method: components["schemas"]["Method"];
            /** @description Its name, unique in the course. */
            name: string;
            /**
             * Format: date-time
             * @description When people may start joining.
             */
            open_from: string | null;
            /**
             * Format: date-time
             * @description When people may no longer join.
             */
            open_until: string | null;
            /** @description Whether joining waits for an approval. */
            requires_approval: boolean;
            /**
             * Format: uuid
             * @description The role its enrolments give.
             */
            role: string;
            /**
             * Format: uuid
             * @description The system group it follows, for that method.
             */
            source: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description One thing a calendar shows. */
        Item: {
            /** @description Whether it lasts whole days. */
            all_day: boolean;
            /**
             * Format: uuid
             * @description The course module, for the date of an activity.
             */
            course_module: string | null;
            /**
             * Format: date-time
             * @description When it ends, for an entry.
             */
            ends_at: string | null;
            /**
             * Format: uuid
             * @description The entry, for an entry.
             */
            entry: string | null;
            /** @description What it is. */
            kind: components["schemas"]["ItemKind"];
            /**
             * Format: date-time
             * @description For an occurrence of a recurring entry, when it was due to start by the rule.
             */
            occurrence: string | null;
            /**
             * Format: date-time
             * @description When it starts, or happens.
             */
            starts_at: string;
            /** @description Its title: the title of the entry or of the activity. */
            title: string;
        };
        /** @description One part of a request: a data category and where it stands. */
        Item2: {
            /** @description The data category. */
            category: string;
            /** @description Why it failed. */
            error: string | null;
            /** @description `pending`, `completed` or `failed`. */
            state: string;
        };
        /** @description What an item of a calendar is. */
        ItemKind: "entry" | "opens" | "due" | "closes" | "cutoff" | "imported";
        /** @description The outcome of enrolling oneself. */
        Joined: {
            /**
             * Format: uuid
             * @description The enrolment.
             */
            enrolment: string;
            /** @constant */
            outcome: "enrolled";
        } | {
            /** @constant */
            outcome: "requested";
            /**
             * Format: uuid
             * @description The request.
             */
            request: string;
        };
        /** @description The layout of a page kind in a scope. */
        Layout: {
            /** @description Whether the reader may change it here: they may arrange the page and no scope above locks it. */
            editable: boolean;
            /** @description The grid its widgets are placed on. */
            grid: components["schemas"]["PageGrid"];
            /** @description Whether the reader may lock widgets here for the scopes below: never on a page a person arranges for themselves. */
            lockable: boolean;
            /** @description Where the layout came from. */
            origin: components["schemas"]["Origin"];
            /** @description The page kind. */
            page_kind: string;
            /**
             * @description The widgets the reader sees, in reading order (by row, then by column); the page shows its own arrangement
             *     when the layout comes from its declaration.
             */
            placements: components["schemas"]["Placement"][];
            /** @description The widgets the reader may place on the page kind, with their sizes and settings; only when it is editable. */
            widgets: components["schemas"]["AvailableWidget"][];
        };
        /** @description Bands that turn a percentage into a letter. */
        LetterSet: {
            /** @description Its bands, from the highest percentage down. */
            bands: components["schemas"]["Band"][];
            /**
             * Format: uuid
             * @description The version of the set.
             */
            id: string;
            /**
             * Format: uuid
             * @description The scope it is defined in; it applies there and below.
             */
            scope: string;
            /**
             * Format: int32
             * @description Its version.
             */
            version: number;
        };
        /** @description How a provider identity that is not linked yet finds its account. */
        LinkingPolicy: "subject_only" | "verified_email" | "confirm";
        /** @description The file of a material. */
        MaterialFile: {
            /**
             * Format: uuid
             * @description The file.
             */
            file: string;
        };
        /** @description A member of a group, once per origin. */
        Member: {
            /**
             * Format: uuid
             * @description The account.
             */
            account: string;
            /** @description Where the membership comes from. */
            origin: components["schemas"]["MemberOrigin"];
        };
        /** @description A member of a system group, once per origin. */
        Member2: {
            /**
             * Format: uuid
             * @description The account.
             */
            account: string;
            /** @description Where the membership comes from. */
            origin: components["schemas"]["Origin2"];
        };
        /** @description Where a group membership comes from; a module removes only what it added. */
        MemberOrigin: {
            /** @constant */
            kind: "manual";
        } | {
            /** @constant */
            kind: "module";
            module: string;
        };
        /** @description A navigation menu in a scope. */
        Menu: {
            /** @description Whether the reader may change it here: they hold `customise.menu.manage` and no scope above locks it. */
            editable: boolean;
            /** @description The items the reader sees, in order. */
            items: components["schemas"]["MenuItem"][];
            /** @description The kind of menu. */
            kind: string;
            /** @description Where the menu came from. */
            origin: components["schemas"]["Origin"];
        };
        /** @description An item of a navigation menu, with the items under it. */
        MenuItem: {
            /**
             * @description The items under it, in order.
             * @default []
             */
            children: components["schemas"]["MenuItem"][];
            /** @description What it says. */
            label: string;
            /**
             * @description The permission a person needs to see it, and the items under it; everyone when absent.
             * @default null
             */
            permission: string | null;
            /** @description A path of the site (`/…`) or an `http`/`https` address. */
            target: string;
        };
        /** @description One message of a conversation. */
        Message: {
            /**
             * Format: uuid
             * @description Who wrote it.
             */
            author: string;
            /** @description What it says, in sanitised rich text. */
            body: string;
            /**
             * Format: uuid
             * @description Its conversation.
             */
            conversation: string;
            /** @description The files attached to it, in order. */
            files: string[];
            /**
             * Format: uuid
             * @description The message.
             */
            id: string;
            /**
             * Format: date-time
             * @description When it was sent.
             */
            sent_at: string;
        };
        /** @description The methods this module provides. */
        Method: "manual" | "self" | "system_group";
        /** @description How a category aggregates its children. */
        Method2: "mean" | "weighted_mean" | "simple_weighted_mean" | "natural" | "median" | "lowest" | "highest" | "mode";
        /** @description How strongly a session was authenticated. */
        MfaLevel: "none" | "second_factor";
        /** @description A course module: one activity or material placed in a section, with a scope of its own under it. */
        Module: {
            /**
             * @description Whether the person who asked meets its access conditions; when not, what the course module holds is
             *     left out.
             */
            available: boolean;
            /**
             * Format: uuid
             * @description The course module.
             */
            id: string;
            /** @description Whether it is listed on the course page; separate from whether people may open it. */
            on_course_page: boolean;
            /**
             * Format: uuid
             * @description Its scope, under the scope of its section.
             */
            scope: string;
            /**
             * Format: uuid
             * @description Its section.
             */
            section: string;
            /** @description What its type keeps for it, each of its declared kind. */
            settings: {
                [key: string]: unknown;
            };
            /** @description Its place among the course modules of its section. */
            sort_key: string;
            /** @description Where it stands. */
            state: components["schemas"]["ModuleState"];
            /** @description Its title. */
            title: string;
            /** @description Its type, such as `activities.page`. */
            type: string;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Whether it is shown, hidden or shown for a while. */
            visibility: components["schemas"]["Visibility"];
        };
        /** @description Where a course module stands. */
        ModuleState: "active" | "in_bin";
        /** @description A new feed, with its secret address, shown once. */
        NewFeedAnswer: {
            /** @description The feed. */
            feed: components["schemas"]["Feed"];
            /** @description Its secret. */
            secret: string;
            /** @description The address of its calendar. */
            url: string;
        };
        /** @description A created token; the token itself is shown this once. */
        NewToken: {
            /**
             * Format: date-time
             * @description When it expires.
             */
            expires_at: string;
            /**
             * Format: uuid
             * @description The token.
             */
            id: string;
            /** @description The prefix that names it. */
            prefix: string;
            /** @description The secret to give to the integration. */
            token: string;
        };
        /** @description One notification to one person. */
        Notification: {
            /**
             * Format: date-time
             * @description When it was created.
             */
            created_at: string;
            /**
             * Format: uuid
             * @description The notification.
             */
            id: string;
            /**
             * @description What it is about: the objects it links to, which are checked against the permissions of the person
             *     when they open it.
             */
            payload: unknown;
            /**
             * Format: date-time
             * @description When the person read it.
             */
            read_at: string | null;
            /**
             * Format: uuid
             * @description Who it is for.
             */
            recipient: string;
            /** @description Its type. */
            type: string;
        };
        /** @description A rule the type of a course module offers. */
        OfferedRule: {
            /** @description Its code. */
            code: string;
            /** @description What it asks for. */
            summary: string;
        };
        /** @description How the children of a node of a tree of conditions combine. */
        Operator: "all" | "any" | "not" | "leaf";
        /** @description How a field of the payload is compared. */
        Operator2: "eq" | "ne" | "in" | "exists";
        /** @description Where a resolved customisation came from. */
        Origin: {
            /** @constant */
            from: "declared";
        } | {
            /** @constant */
            from: "scope";
            /** @description Whether that scope locks it for the scopes below. */
            locked: boolean;
            /**
             * Format: uuid
             * @description The scope.
             */
            scope: string;
        };
        /** @description Where a membership comes from; a rule or a sync removes only the memberships it made. */
        Origin2: "manual" | "rule" | "sync";
        /** @description A package of customisations. */
        Package: {
            /**
             * @description The custom fields.
             * @default []
             */
            fields: components["schemas"]["PackageField"][];
            /**
             * @description The values of feature flags.
             * @default []
             */
            flags: components["schemas"]["PackageFlag"][];
            /**
             * Format: uint16
             * @description The format it is written in.
             */
            format: number;
            /**
             * @description The layouts.
             * @default []
             */
            layouts: components["schemas"]["PackageLayout"][];
            /**
             * @description The menus.
             * @default []
             */
            menus: components["schemas"]["PackageMenu"][];
            /**
             * @description The rules, which the importer owns once applied.
             * @default []
             */
            rules: components["schemas"]["RuleSpec"][];
            /**
             * @description The words of terms.
             * @default []
             */
            terms: components["schemas"]["PackageTerm"][];
            /**
             * @description The theme.
             * @default null
             */
            theme: components["schemas"]["PackageTheme"] | null;
        };
        /** @description A custom field a scope defines. */
        PackageField: {
            /**
             * @description Its choices, for choice fields.
             * @default []
             */
            choices: components["schemas"]["Choice"][];
            /** @description Its code. */
            code: string;
            /** @description The kind of entity it belongs to. */
            kind: string;
            /**
             * Format: int32
             * @description The longest text, for text fields.
             * @default null
             */
            max_length: number | null;
            /** @description Its name. */
            name: string;
            /**
             * @description Whether every entity has a value.
             * @default false
             */
            required: boolean;
            /** @description Its type. */
            type: components["schemas"]["FieldType"];
            /**
             * @description Who sees its values.
             * @default public
             */
            visibility: components["schemas"]["FieldVisibility"];
        };
        /** @description The value a scope gives a feature flag. */
        PackageFlag: {
            /** @description The flag. */
            code: string;
            /** @description Whether the feature is on. */
            enabled: boolean;
            /**
             * @description Whether the scopes below keep it.
             * @default false
             */
            locked: boolean;
        };
        /** @description A kind of customisation a package holds. */
        PackageKind: "terms" | "flags" | "fields" | "layouts" | "menus" | "theme" | "rules";
        /** @description The layout a scope gives a page kind. */
        PackageLayout: {
            /**
             * @description Whether the scopes below keep it.
             * @default false
             */
            locked: boolean;
            /** @description The page kind. */
            page_kind: string;
            /** @description Its widgets, in order. */
            placements: components["schemas"]["Placement"][];
        };
        /** @description The menu a scope gives a kind of menu. */
        PackageMenu: {
            /** @description Its items, in order. */
            items: components["schemas"]["MenuItem"][];
            /** @description The kind of menu. */
            kind: string;
            /**
             * @description Whether the scopes below keep it.
             * @default false
             */
            locked: boolean;
        };
        /** @description The word a scope gives a term in a language. */
        PackageTerm: {
            /** @description The term. */
            code: string;
            /** @description The language. */
            language: string;
            /**
             * @description Whether the scopes below keep it.
             * @default false
             */
            locked: boolean;
            /** @description The word for several. */
            plural: string;
            /** @description The word for one. */
            singular: string;
        };
        /** @description The tokens a scope sets in its theme. */
        PackageTheme: {
            /**
             * @description Whether the scopes below keep them.
             * @default false
             */
            locked: boolean;
            /** @description The tokens, by name. */
            tokens: {
                [key: string]: unknown;
            };
        };
        /** @description One page of a list, with the cursor of the next page when there is one. */
        Page: {
            /** @description The items of the page. */
            items: components["schemas"]["Conversation"][];
            /** @description The cursor of the next page, absent on the last page. */
            next: string | null;
        };
        /** @description One page of a list, with the cursor of the next page when there is one. */
        Page2: {
            /** @description The items of the page. */
            items: components["schemas"]["Message"][];
            /** @description The cursor of the next page, absent on the last page. */
            next: string | null;
        };
        /** @description The grid of a page kind. */
        PageGrid: {
            /**
             * Format: uint16
             * @description How many columns it has.
             */
            columns: number;
            /** @description The height of a row, by name: `compact`, `regular` or `tall`. */
            row: components["schemas"]["RowHeight"];
            /**
             * Format: uint16
             * @description The height of a row, in CSS pixels.
             */
            row_pixels: number;
        };
        /** @description The picture of an account. */
        PictureFile: {
            /**
             * Format: uuid
             * @description The file of the picture.
             */
            file: string;
        };
        /** @description A widget on the grid of a layout: the rectangle of cells it takes, and its settings. */
        Placement: {
            /**
             * @description Whether its height is the height of its content rather than its rows, as its widget declares on the page kind;
             *     given by the server and ignored when a layout is written.
             * @default false
             */
            grows: boolean;
            /**
             * Format: uint16
             * @description How many rows it takes, within the sizes the widget declares for the page kind.
             */
            h: number;
            /**
             * @description Whether a scope above locked it on this page, so it shows as that scope placed it; given by the server and
             *     ignored when a layout is written.
             * @default false
             */
            held: boolean;
            /**
             * @description Whether the scope it is placed in locks it for the scopes below: there it shows as placed here, and it cannot
             *     be moved, resized, edited or removed.
             * @default false
             */
            locked: boolean;
            /**
             * @description The permission a person needs to see it; everyone who sees the page when absent.
             * @default null
             */
            permission: string | null;
            /**
             * @description Whether the person pinned it where it is; otherwise it rises to the widgets above it, so no row is left empty.
             * @default false
             */
            pinned: boolean;
            /**
             * @description Its settings, each of the kind the widget declares; the defaults fill what is left out.
             * @default {}
             */
            settings: {
                [key: string]: unknown;
            };
            /**
             * Format: uint16
             * @description How many columns it takes, within the sizes the widget declares for the page kind.
             */
            w: number;
            /** @description The widget, by code. */
            widget: string;
            /**
             * Format: uint16
             * @description The first column it takes, from 0.
             */
            x: number;
            /**
             * Format: uint16
             * @description The first row it takes, from 0.
             */
            y: number;
        };
        /** @description A policy of a campus, as of its latest version. */
        Policy: {
            /**
             * Format: date-time
             * @description When the person accepted that version, when they did.
             */
            accepted_at: string | null;
            /** @description What it says in that version. */
            body: string;
            /** @description What people refer to it by, unique in the campus. */
            code: string;
            /**
             * Format: uuid
             * @description The policy.
             */
            id: string;
            /**
             * Format: date-time
             * @description When that version was published.
             */
            published_at: string;
            /** @description Its title. */
            title: string;
            /**
             * Format: int32
             * @description Its latest version.
             */
            version: number;
        };
        /** @description What a person receives of a type of notification through a channel. */
        Preference: {
            /** @description The channel. */
            channel: string;
            /** @description Whether the person receives it. */
            enabled: boolean;
            /** @description Whether the campus fixed it, so the person cannot change it. */
            locked: boolean;
            /** @description Whether it cannot be turned off. */
            required: boolean;
            /** @description What it tells people. */
            summary: string;
            /** @description The type. */
            type: string;
        };
        /** @description The prerequisites around a course. */
        Prerequisites: {
            /**
             * Format: uuid
             * @description The course.
             */
            course: string;
            /** @description The courses that require it first. */
            required_by: string[];
            /** @description The courses it requires first. */
            requires: string[];
        };
        /** @description The problem details (RFC 9457) every error is answered with, as `application/problem+json`. */
        Problem: {
            /**
             * @description What went wrong, as a stable code a client can translate (`courses.code_taken`); `internal` for an
             *     internal error, which exposes nothing else.
             */
            code: string;
            /** @description Every failing field of a validation problem. */
            fields: components["schemas"]["FieldError"][];
            /** @description The values the message of the code is written with. */
            params: {
                [key: string]: string;
            };
            /** @description The id of the request, to find it in the logs. */
            request_id: string;
            /**
             * Format: uint16
             * @description The HTTP status.
             */
            status: number;
            /** @description The kind of problem, which decides how a client handles it. */
            title: components["schemas"]["ErrorKind"];
            /** @description The address of the kind of problem (`https://kampus.didactika.org/problems/not_found`). */
            type: string;
        };
        /** @description The person behind an account. */
        Profile: {
            /** @description How the person is shown; the given and family names when absent. */
            display_name: string | null;
            /** @description The family name. */
            family_name: string | null;
            /** @description The given name. */
            given_name: string | null;
            /** @description The language of the person, a BCP 47 tag; the campus language when absent. */
            language_code: string | null;
            /** @description The time zone of the person, an IANA name; the campus zone when absent. */
            time_zone: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`; 0 while the profile was never saved.
             */
            version: number;
        };
        /** @description A provider as listed on the sign-in page. */
        Provider: {
            /**
             * Format: uuid
             * @description The provider.
             */
            id: string;
            /** @description Its issuer. */
            issuer: string;
            /** @description Its linking policy. */
            linking_policy: components["schemas"]["LinkingPolicy"];
        };
        /** @description The public key of web push. */
        PushKey: {
            /** @description The key, in base64url. */
            public_key: string;
        };
        /** @description A browser or phone a person allowed to receive push notifications. */
        PushSubscription: {
            /** @description Its authentication secret, as the browser gives it. */
            auth: string;
            /** @description Where its push service receives messages for it. */
            endpoint: string;
            /**
             * Format: uuid
             * @description The subscription.
             * @default null
             */
            id: string | null;
            /** @description Its public key, as the browser gives it. */
            p256dh: string;
        };
        /** @description How much files may take and take. */
        Quota: {
            /**
             * Format: int64
             * @description The most, in bytes; no limit when absent.
             */
            limit_bytes: number | null;
            /**
             * Format: int64
             * @description What attached files take, in bytes.
             */
            used_bytes: number;
        };
        /** @description How many notifications were marked read. */
        ReadAll: {
            /**
             * Format: uint64
             * @description The notifications that changed.
             */
            changed: number;
        };
        /** @description One graded thing of a course. */
        Record: {
            /**
             * Format: uuid
             * @description The category that aggregates it; none for the course total.
             */
            category: string | null;
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /**
             * Format: uuid
             * @description The course module it grades, for an activity record.
             */
            course_module: string | null;
            /** @description Whether it counts on top of the maximum of its category. */
            extra_credit: boolean;
            /** @description What its grades are made of. */
            grade_type: components["schemas"]["GradeType"];
            /**
             * Format: date-time
             * @description Hidden until then.
             */
            hidden_until: string | null;
            /**
             * Format: uuid
             * @description The record.
             */
            id: string;
            /**
             * Format: int32
             * @description Which of the grades of its course module it is.
             */
            item_number: number | null;
            /** @description What it stands for. */
            kind: components["schemas"]["RecordKind2"];
            /**
             * Format: date-time
             * @description Locked since then.
             */
            locked_at: string | null;
            /** @description The highest grade. */
            max: string;
            /** @description The lowest grade. */
            min: string;
            /** @description What its grades are multiplied by. */
            multiplier: string;
            /** @description What is added to its grades after multiplying. */
            offset: string;
            /** @description The grade that passes. */
            pass_grade: string | null;
            /**
             * Format: uuid
             * @description The version of the scale its grades are items of, for a scale record.
             */
            scale: string | null;
            /** @description Its place among its siblings. */
            sort_key: string;
            /** @description Its title. */
            title: string;
            /**
             * Format: uuid
             * @description The category it totals, for a total.
             */
            total_of: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Its weight in the category. */
            weight: string;
        };
        /** @description A kind of record of courses that carries external references. */
        RecordKind: "category" | "course" | "section" | "group" | "group_set";
        /** @description What a grade record stands for. */
        RecordKind2: "activity" | "manual" | "category_total" | "course_total";
        /** @description A new set of recovery codes, shown this once. */
        RecoveryCodes: {
            /** @description The codes. */
            recovery_codes: string[];
        };
        /** @description What completing a recovery needs, so the client asks for it before the new password. */
        RecoveryNeeds: {
            /**
             * @description The kinds of second factor that may be given; empty when the link alone is enough. Recovery codes
             *     work whenever this is not empty; email codes never do, since the link came by email.
             */
            methods: components["schemas"]["FactorKind"][];
        };
        /** @description What a read changed. */
        Refreshed: {
            /**
             * Format: uint64
             * @description Events added.
             */
            added: number;
            /**
             * Format: uint64
             * @description Events changed.
             */
            changed: number;
            /**
             * Format: uint64
             * @description Events removed.
             */
            removed: number;
        };
        /** @description A page of the grader report. */
        Report: {
            /**
             * Format: uuid
             * @description Where the next page starts, when there is one.
             */
            next: string | null;
            /** @description The people of the page, in the order of their ids. */
            rows: components["schemas"]["ReportRow"][];
        };
        /** @description The grades of one person in a report. */
        ReportRow: {
            /**
             * Format: uuid
             * @description The person.
             */
            account: string;
            /**
             * @description Whether they take part in the course now; the grades of those who left stay, out of the report
             *     unless asked for.
             */
            active: boolean;
            /** @description Their grades, those the viewer may see. */
            grades: components["schemas"]["Grade"][];
        };
        /** @description A request to join a course through an instance. */
        Request: {
            /**
             * Format: uuid
             * @description The account asking.
             */
            account: string;
            /**
             * Format: date-time
             * @description When it was made.
             */
            created_at: string;
            /**
             * Format: uuid
             * @description The group of the key they gave, when any.
             */
            group: string | null;
            /**
             * Format: uuid
             * @description The request.
             */
            id: string;
            /**
             * Format: uuid
             * @description Its instance.
             */
            instance: string;
            /** @description What they wrote. */
            message: string | null;
            /** @description Its state. */
            state: components["schemas"]["RequestState"];
        };
        /** @description A request to export or to delete the personal data of an account. */
        Request2: {
            /**
             * Format: uuid
             * @description Whose data.
             */
            account: string;
            /**
             * Format: uuid
             * @description Who approved a deletion.
             */
            approved_by: string | null;
            /**
             * Format: date-time
             * @description When it was answered.
             */
            completed_at: string | null;
            /**
             * Format: date-time
             * @description When it was made.
             */
            created_at: string;
            /**
             * Format: date-time
             * @description When it must be answered by.
             */
            due_at: string;
            /**
             * Format: date-time
             * @description When the data stops being available.
             */
            expires_at: string | null;
            /**
             * Format: uuid
             * @description The request.
             */
            id: string;
            /** @description Its parts. */
            items: components["schemas"]["Item2"][];
            /** @description What it asks for. */
            kind: string;
            /** @description Why a deletion was asked for. */
            reason: string | null;
            /** @description `pending`, `awaiting_approval`, `held`, `completed`, `failed`, `expired` or `rejected`. */
            state: string;
        };
        /** @description The state of a request to join. */
        RequestState: "pending" | "approved" | "rejected" | "withdrawn";
        /** @description How a widget may be resized. */
        Resize: "none" | "horizontal" | "vertical" | "both";
        /** @description A kind of personal data and how long the campus keeps it after the account is deleted. */
        Retention: {
            /** @description The data category, `<module>.<what>`. */
            category: string;
            /**
             * Format: int32
             * @description Days it is kept after the account is deleted; none keeps it for ever.
             */
            keep_days: number | null;
            /** @description Why the data is kept. */
            purpose: string;
        };
        /** @description The height of a row of a grid, by name. */
        RowHeight: "compact" | "regular" | "tall";
        /** @description A rule of a scope; it reacts to the events of its scope and of the scopes below. */
        Rule: {
            /** @description What it does, in order. */
            actions: components["schemas"]["RuleAction"][];
            /** @description When it runs. */
            conditions: components["schemas"]["Condition"] | null;
            /** @description Whether it runs. */
            enabled: boolean;
            /**
             * Format: uuid
             * @description The rule.
             */
            id: string;
            /** @description Its name. */
            name: string;
            /**
             * Format: uuid
             * @description The principal it acts as: who wrote it last.
             */
            owner: string;
            /**
             * Format: uuid
             * @description Its scope.
             */
            scope: string;
            /** @description The event type that triggers it. */
            trigger: string;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
        };
        /** @description An action of a rule, with its parameters. */
        RuleAction: {
            /** @description The action a module declares, by code. */
            action: string;
            /**
             * @description Its parameters, each of its declared kind.
             * @default {}
             */
            params: {
                [key: string]: unknown;
            };
        };
        /** @description The profile field a rule of a system group compares. */
        RuleField: "given_name" | "family_name" | "language_code" | "time_zone";
        /** @description What a course module can require of a person. */
        RuleKind: "manual" | "view" | "submit" | "grade" | "pass";
        /** @description One evaluation of a rule for an event. */
        RuleRun: {
            /**
             * Format: int32
             * @description How many of its actions ran.
             */
            actions_done: number;
            /**
             * Format: date-time
             * @description When it was evaluated.
             */
            at: string;
            /** @description Why it failed. */
            error: string | null;
            /**
             * Format: uuid
             * @description The event.
             */
            event: string;
            /** @description `running`, `succeeded`, `failed`, or `skipped` when its conditions did not hold. */
            outcome: string;
        };
        /** @description A rule to write. */
        RuleSpec: {
            /** @description What it does, in order. */
            actions: components["schemas"]["RuleAction"][];
            /**
             * @description When it runs; always when absent.
             * @default null
             */
            conditions: components["schemas"]["Condition"] | null;
            /**
             * @description Whether it runs.
             * @default true
             */
            enabled: boolean;
            /** @description Its name. */
            name: string;
            /** @description The event type that triggers it. */
            trigger: string;
        };
        /**
         * @description An ordered list of items used instead of numbers; a new version is a new row, and a record keeps the
         *     version it was graded with.
         */
        Scale: {
            /** @description Its code, the same across versions. */
            code: string;
            /**
             * Format: uuid
             * @description The version of the scale.
             */
            id: string;
            /** @description Its items, from the lowest to the highest. */
            items: string[];
            /**
             * Format: uuid
             * @description The scope it is defined in; it is available there and below.
             */
            scope: string;
            /**
             * Format: int32
             * @description Its version.
             */
            version: number;
        };
        /** @description The colour roles of one scheme, light or dark, by name (`primary`, `on-primary-container`…). */
        Scheme: {
            [key: string]: string;
        };
        /** @description A second factor as listed to its owner; its secret never leaves identity. */
        SecondFactor: {
            /**
             * Format: date-time
             * @description When it was added.
             */
            created_at: string;
            /**
             * Format: uuid
             * @description The factor.
             */
            id: string;
            /** @description Its kind. */
            kind: components["schemas"]["FactorKind"];
            /** @description The name its owner gave it. */
            label: string;
            /**
             * Format: date-time
             * @description When it was last used.
             */
            last_used_at: string | null;
        };
        /** @description A section of a course. */
        Section: {
            /**
             * Format: uuid
             * @description Its course.
             */
            course: string;
            /**
             * Format: uuid
             * @description The section.
             */
            id: string;
            /**
             * Format: uuid
             * @description The section it is nested in.
             */
            parent: string | null;
            /**
             * Format: uuid
             * @description Its scope.
             */
            scope: string;
            /** @description Its place among its siblings. */
            sort_key: string;
            /** @description Its summary. */
            summary: string | null;
            /** @description Its title. */
            title: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Its own visibility. */
            visibility: components["schemas"]["Visibility"];
        };
        /** @description A session that authenticated a request. */
        Session: {
            /**
             * Format: uuid
             * @description Its account.
             */
            account: string;
            /**
             * Format: uuid
             * @description The campus of the account.
             */
            campus: string;
            /**
             * Format: date-time
             * @description When it ends at the latest.
             */
            expires_at: string;
            /**
             * Format: uuid
             * @description The session.
             */
            id: string;
            /** @description How it was authenticated. */
            mfa_level: components["schemas"]["MfaLevel"];
            /**
             * Format: date-time
             * @description The last strong authentication, for step-up.
             */
            strong_auth_at: string;
        };
        /** @description A session as listed to its owner. */
        SessionSummary: {
            /** @description The address it was opened from. */
            address: string | null;
            /** @description The client that opened it. */
            client: string | null;
            /**
             * Format: date-time
             * @description When it started.
             */
            created_at: string;
            /**
             * Format: date-time
             * @description When it ends at the latest.
             */
            expires_at: string;
            /**
             * Format: uuid
             * @description The session.
             */
            id: string;
            /**
             * Format: date-time
             * @description When it was last used.
             */
            last_seen_at: string;
        };
        /** @description A link that reads one file for a few minutes, without signing in. */
        SignedUrl: {
            /**
             * Format: date-time
             * @description When it stops working.
             */
            expires_at: string;
            /** @description The link. */
            url: string;
        };
        /** @description What a sign-in answers, by its `status`. */
        SignInAnswer: {
            /** @description The recovery codes of a first second factor, shown this once. */
            recovery_codes: string[] | null;
            /** @description The session opened. */
            session: components["schemas"]["Session"];
            /** @constant */
            status: "signed_in";
        } | {
            /** @description The challenge to complete with `/auth/second-factor`. */
            challenge: string;
            /** @description The kinds of second factor that may complete it. */
            methods: components["schemas"]["FactorKind"][];
            /** @constant */
            status: "second_factor_required";
        } | {
            /** @description The sealed identity to link with `/me/identity-links`. */
            link: string;
            /** @constant */
            status: "confirm_link";
        };
        /** @description A size on a grid, in cells. */
        Size: {
            /**
             * Format: uint16
             * @description Rows.
             */
            h: number;
            /**
             * Format: uint16
             * @description Columns.
             */
            w: number;
        };
        /** @description The heights a widget takes between two widths. */
        SizeBand: {
            /** @description The lowest and the highest it takes there, in rows. */
            heights: number[];
            /** @description The narrowest and the widest it applies to, in columns. */
            widths: number[];
        };
        /** @description Where a state came from. */
        Source: "automatic" | "exception";
        /** @description Where the change of a grade came from. */
        Source2: "activity" | "teacher" | "import" | "formula";
        /** @description The state of a person on a course module. */
        State: "not_started" | "in_progress" | "done" | "done_passed" | "done_failed";
        /** @description The status of an enrolment. */
        Status: "active" | "suspended";
        /** @description The session a step-up raised. */
        SteppedUp: {
            /** @description The session. */
            session: components["schemas"]["Session"];
        };
        /** @description A calendar of another system shown in a scope, read again every so often. */
        Subscription: {
            /**
             * Format: uuid
             * @description The subscription.
             */
            id: string;
            /** @description Why the last read failed. */
            last_error: string | null;
            /**
             * Format: date-time
             * @description When it was last read.
             */
            last_refreshed_at: string | null;
            /**
             * Format: int32
             * @description Every how many minutes it is read.
             */
            refresh_minutes: number;
            /**
             * Format: uuid
             * @description The scope it is shown in.
             */
            scope: string;
            /** @description What it is called. */
            title: string;
            /** @description Where it is read from. */
            url: string;
        };
        /** @description What a sync of an instance with its source did. */
        Synced: {
            /**
             * Format: uint
             * @description Enrolments added.
             */
            added: number;
            /**
             * Format: uint
             * @description Enrolments removed.
             */
            removed: number;
        };
        /** @description A system group. */
        SystemGroup: {
            /** @description Its code, unique in its scope. */
            code: string;
            /**
             * Format: uuid
             * @description The group.
             */
            id: string;
            /** @description How its members are chosen. */
            kind: components["schemas"]["GroupKind"];
            /** @description Its name. */
            name: string;
            /**
             * Format: uuid
             * @description The campus or category it lives in.
             */
            scope: string;
        };
        /** @description Whom an exception is for. */
        Target: {
            /** Format: uuid */
            person: string;
        } | {
            /** Format: uuid */
            group: string;
        };
        /** @description A template of a campus for a type, a channel and a language. */
        Template: {
            /** @description The text, with placeholders taken from the notification. */
            body: string;
            /** @description The channel, `email` or `push`. */
            channel: string;
            /** @description The language, such as `es`. */
            locale: string;
            /** @description The subject, or the title of a push notification, with placeholders like `{{summary}}`. */
            subject: string;
            /** @description The type. */
            type: string;
        };
        /** @description The word for a term in a scope and a language. */
        Term: {
            /** @description The term. */
            code: string;
            /** @description The language of the forms. */
            language: string;
            /** @description Where the forms came from. */
            origin: components["schemas"]["Origin"];
            /** @description The word for several. */
            plural: string;
            /** @description The word for one. */
            singular: string;
        };
        /**
         * @description The theme of a scope: every token with its value and where it came from, and the colour roles grown
         *     from them.
         */
        Theme: {
            /** @description The colour roles of the dark scheme. */
            dark: components["schemas"]["Scheme"];
            /** @description The colour roles of the light scheme. */
            light: components["schemas"]["Scheme"];
            /** @description Where each token came from. */
            origins: {
                [key: string]: components["schemas"]["Origin"];
            };
            /** @description Every token, by name. */
            tokens: {
                [key: string]: unknown;
            };
        };
        /** @description A permission a token carries, in a scope and its descendants. */
        TokenScope: {
            /** @description The permission. */
            permission: string;
            /**
             * Format: uuid
             * @description The scope.
             */
            scope: string;
        };
        /** @description A token as listed. */
        TokenSummary: {
            /**
             * Format: date-time
             * @description When it was created.
             */
            created_at: string;
            /**
             * Format: date-time
             * @description When it expires.
             */
            expires_at: string;
            /**
             * Format: uuid
             * @description The token.
             */
            id: string;
            /**
             * Format: date-time
             * @description When it was last used.
             */
            last_used_at: string | null;
            /** @description Its name. */
            name: string;
            /** @description Its prefix. */
            prefix: string;
            /**
             * Format: date-time
             * @description When it was revoked.
             */
            revoked_at: string | null;
            /** @description What it may do. */
            scopes: components["schemas"]["TokenScope"][];
        };
        /** @description A new secret for an authenticator app, to show as a QR code. */
        TotpSetup: {
            /** @description The secret in base32. */
            secret: string;
            /** @description The `otpauth://` link. */
            uri: string;
        };
        /** @description The records of a course: its categories and its grade records, each in its place. */
        Tree: {
            /** @description The categories, the root first. */
            categories: components["schemas"]["Category2"][];
            /** @description The grade records. */
            records: components["schemas"]["Record"][];
        };
        /** @description The rules of its type a course module requires, and those its type offers. */
        TypeRulesAnswer: {
            /** @description The rules its type offers. */
            offered: components["schemas"]["OfferedRule"][];
            /** @description The codes of the rules it requires. */
            required: string[];
        };
        /** @description An upload: permission to send one file into an area, for a while. */
        Upload: {
            /** @description The area. */
            area: string;
            /**
             * Format: date-time
             * @description When it stops accepting content.
             */
            expires_at: string;
            /**
             * Format: uuid
             * @description The file it became, once completed.
             */
            file: string | null;
            /**
             * Format: uuid
             * @description The upload.
             */
            id: string;
            /**
             * Format: int64
             * @description The largest content it accepts, in bytes.
             */
            max_size: number;
            /** @description The name of the file. */
            name: string;
            /** @description Whether the content arrived. */
            received: boolean;
            /**
             * Format: uuid
             * @description The scope it is for.
             */
            scope: string;
        };
        /** @description The settings an exception may change; each left out keeps the value below it. */
        Values: {
            /**
             * Format: int32
             * @description How many attempts a person has.
             * @default null
             */
            attempts: number | null;
            /**
             * Format: date-time
             * @description When it closes.
             * @default null
             */
            closes_at: string | null;
            /**
             * Format: date-time
             * @description The last moment a late one is accepted.
             * @default null
             */
            cutoff_at: string | null;
            /**
             * Format: date-time
             * @description When it is due.
             * @default null
             */
            due_at: string | null;
            /**
             * Format: date-time
             * @description When it opens.
             * @default null
             */
            opens_at: string | null;
            /**
             * Format: int32
             * @description The time a person has once started, in seconds.
             * @default null
             */
            time_limit_seconds: number | null;
        };
        /**
         * @description Whether a node is shown. It is the node's own setting: what learners see also depends on every
         *     ancestor, and nothing is ever copied down.
         */
        Visibility: {
            /** @constant */
            mode: "shown";
        } | {
            /** @constant */
            mode: "hidden";
        } | {
            /**
             * Format: date-time
             * @description Shown from.
             */
            from: string | null;
            /** @constant */
            mode: "scheduled";
            /**
             * Format: date-time
             * @description Shown until.
             */
            until: string | null;
        };
        /** @description The sizes a widget takes on a page kind. */
        WidgetSizes: {
            /**
             * @description The heights it takes at some widths; every height between `min` and `max` at the widths no band holds.
             * @default []
             */
            bands: components["schemas"]["SizeBand"][];
            /** @description The one it takes when it is placed. */
            default: components["schemas"]["Size"];
            /**
             * @description Whether its height is the height of its content, however tall, rather than its rows.
             * @default false
             */
            grows: boolean;
            /** @description The largest. */
            max: components["schemas"]["Size"];
            /** @description The smallest. */
            min: components["schemas"]["Size"];
            /** @description How it may be resized between them. */
            resize: components["schemas"]["Resize"];
        };
        /** @description A course with the people who own it. */
        WithOwners: {
            /**
             * Format: uuid
             * @description Its campus.
             */
            campus: string;
            /**
             * Format: uuid
             * @description Its category.
             */
            category: string;
            /** @description Its code, unique in the campus when set. */
            code: string | null;
            /**
             * Format: date-time
             * @description When a scheduled deletion runs.
             */
            deletion_due_at: string | null;
            /**
             * Format: date-time
             * @description When it ends.
             */
            ends_at: string | null;
            /** @description Its full name. */
            full_name: string;
            /**
             * Format: uuid
             * @description The course.
             */
            id: string;
            /** @description The language it is forced to, when set. */
            language_code: string | null;
            /** @description The layout plugin that presents it. */
            layout: string;
            /** @description The outline it is presented with: the code of an outline plugin, as its settings resolve it. */
            outline: string;
            /** @description Its owners. */
            owners: string[];
            /**
             * Format: uuid
             * @description Its scope.
             */
            scope: string;
            /** @description Its short name, unique in the campus. */
            short_name: string;
            /** @description Its place in its category. */
            sort_key: string;
            /**
             * Format: date-time
             * @description When it starts.
             */
            starts_at: string | null;
            /** @description Its lifecycle state. */
            state: components["schemas"]["CourseState"];
            /** @description Its summary. */
            summary: string | null;
            /**
             * Format: int32
             * @description The version, for `If-Match`.
             */
            version: number;
            /** @description Its own visibility. */
            visibility: components["schemas"]["Visibility"];
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
};
export type $defs = Record<string, never>;
export interface operations {
    get_activities_campuses_campus_course_modules_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
                external_id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_course_modules_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_course_modules_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether it is listed on the course page.
                     * @default true
                     */
                    on_course_page?: boolean;
                    /**
                     * @description What its type keeps for it; the defaults fill what is left out.
                     * @default {}
                     */
                    settings?: {
                        [key: string]: unknown;
                    };
                    /** @description Its title. */
                    title: string;
                    /**
                     * @description Whether it is shown, hidden or shown for a while.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_activities_course_modules_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_course_modules_id_conditions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conditions"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_course_modules_id_conditions: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether people who do not meet them still see it listed, without its content.
                     * @default true
                     */
                    show_when_unmet?: boolean;
                    /** @description The tree. */
                    tree: {
                        /** @description Set on a leaf whose kind is unknown or that lost what it depends on: it never holds. */
                        broken?: boolean;
                        /** @description Its children, for `all`, `any` and `not`. */
                        children?: components["schemas"]["ConditionNode"][];
                        /** @description The kind of a leaf, such as `activities.date`. */
                        kind?: string | null;
                        /** @description How it combines its children, or `leaf`. */
                        operator: "all" | "any" | "not" | "leaf";
                        /** @description The parameters of a leaf, as its kind takes them. */
                        params?: unknown;
                    };
                } | null;
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conditions"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_activities_course_modules_id_copy: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Where among its course modules. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /**
                     * Format: uuid
                     * @description The section.
                     */
                    section: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_course_modules_id_effective_values_account: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Values"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_course_modules_id_exceptions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Exception"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_course_modules_id_exceptions_accounts_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description How many attempts a person has.
                     * @default null
                     */
                    attempts?: number | null;
                    /**
                     * Format: date-time
                     * @description When it closes.
                     * @default null
                     */
                    closes_at?: string | null;
                    /**
                     * Format: date-time
                     * @description The last moment a late one is accepted.
                     * @default null
                     */
                    cutoff_at?: string | null;
                    /**
                     * Format: date-time
                     * @description When it is due.
                     * @default null
                     */
                    due_at?: string | null;
                    /**
                     * Format: date-time
                     * @description When it opens.
                     * @default null
                     */
                    opens_at?: string | null;
                    /**
                     * @description Why it is granted.
                     * @default null
                     */
                    reason?: string | null;
                    /**
                     * Format: int32
                     * @description The time a person has once started, in seconds.
                     * @default null
                     */
                    time_limit_seconds?: number | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Exception"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_course_modules_id_exceptions_groups_group: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                group: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description How many attempts a person has.
                     * @default null
                     */
                    attempts?: number | null;
                    /**
                     * Format: date-time
                     * @description When it closes.
                     * @default null
                     */
                    closes_at?: string | null;
                    /**
                     * Format: date-time
                     * @description The last moment a late one is accepted.
                     * @default null
                     */
                    cutoff_at?: string | null;
                    /**
                     * Format: date-time
                     * @description When it is due.
                     * @default null
                     */
                    due_at?: string | null;
                    /**
                     * Format: date-time
                     * @description When it opens.
                     * @default null
                     */
                    opens_at?: string | null;
                    /**
                     * @description Why it is granted.
                     * @default null
                     */
                    reason?: string | null;
                    /**
                     * Format: int32
                     * @description The time a person has once started, in seconds.
                     * @default null
                     */
                    time_limit_seconds?: number | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Exception"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_course_modules_id_file: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MaterialFile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_course_modules_id_file: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description A file uploaded into `activities.material`.
                     */
                    file: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MaterialFile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_activities_course_modules_id_move: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Where among its course modules. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /**
                     * Format: uuid
                     * @description The section.
                     */
                    section: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_course_modules_id_references: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_course_modules_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The identifier. */
                    external_id: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_activities_course_modules_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_activities_course_modules_id_restore: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_courses_id_bin: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BinItem"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_activities_exceptions_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_sections_id_conditions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conditions"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_sections_id_conditions: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether people who do not meet them still see it listed, without its content.
                     * @default true
                     */
                    show_when_unmet?: boolean;
                    /** @description The tree. */
                    tree: {
                        /** @description Set on a leaf whose kind is unknown or that lost what it depends on: it never holds. */
                        broken?: boolean;
                        /** @description Its children, for `all`, `any` and `not`. */
                        children?: components["schemas"]["ConditionNode"][];
                        /** @description The kind of a leaf, such as `activities.date`. */
                        kind?: string | null;
                        /** @description How it combines its children, or `leaf`. */
                        operator: "all" | "any" | "not" | "leaf";
                        /** @description The parameters of a leaf, as its kind takes them. */
                        params?: unknown;
                    };
                } | null;
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conditions"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_sections_id_course_modules: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_activities_sections_id_course_modules: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether it is listed on the course page.
                     * @default true
                     */
                    on_course_page?: boolean;
                    /** @description Where among the course modules of the section. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /**
                     * @description What its type keeps for it; the defaults fill what is left out.
                     * @default {}
                     */
                    settings?: {
                        [key: string]: unknown;
                    };
                    /** @description Its title. */
                    title: string;
                    /** @description Its type. */
                    type: string;
                    /**
                     * @description Whether it is shown, hidden or shown for a while.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_activities_sections_id_course_modules_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                external_id: string;
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether it is listed on the course page.
                     * @default true
                     */
                    on_course_page?: boolean;
                    /** @description Where among the course modules of the section. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /**
                     * @description What its type keeps for it; the defaults fill what is left out.
                     * @default {}
                     */
                    settings?: {
                        [key: string]: unknown;
                    };
                    /** @description Its title. */
                    title: string;
                    /** @description Its type. */
                    type: string;
                    /**
                     * @description Whether it is shown, hidden or shown for a while.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Module"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_activities_types: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityType"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_calendar_courses_id_calendar: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Item"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_calendar_entries: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Item"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_calendar_entries: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether it lasts whole days.
                     * @default false
                     */
                    all_day?: boolean;
                    /**
                     * @description What it is about.
                     * @default null
                     */
                    description?: string | null;
                    /**
                     * Format: date-time
                     * @description When it ends.
                     */
                    ends_at: string;
                    /**
                     * Format: uuid
                     * @description The group of the course it is for.
                     * @default null
                     */
                    group?: string | null;
                    /**
                     * @description Its recurrence rule, RFC 5545, for an entry that repeats.
                     * @default null
                     */
                    recurrence?: string | null;
                    /**
                     * Format: uuid
                     * @description The scope it belongs to; a personal entry when left out.
                     * @default null
                     */
                    scope?: string | null;
                    /**
                     * Format: date-time
                     * @description When it starts.
                     */
                    starts_at: string;
                    /**
                     * @description The time zone its recurrence and all-day dates follow; UTC when left out.
                     * @default null
                     */
                    time_zone?: string | null;
                    /** @description Its title. */
                    title: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Entry"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_calendar_entries_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Entry"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_calendar_entries_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether it lasts whole days.
                     * @default false
                     */
                    all_day?: boolean;
                    /**
                     * @description What it is about.
                     * @default null
                     */
                    description?: string | null;
                    /**
                     * Format: date-time
                     * @description When it ends.
                     */
                    ends_at: string;
                    /**
                     * Format: uuid
                     * @description The group of the course it is for.
                     * @default null
                     */
                    group?: string | null;
                    /**
                     * @description Its recurrence rule, RFC 5545, for an entry that repeats.
                     * @default null
                     */
                    recurrence?: string | null;
                    /**
                     * Format: date-time
                     * @description When it starts.
                     */
                    starts_at: string;
                    /**
                     * @description The time zone its recurrence and all-day dates follow; UTC when left out.
                     * @default null
                     */
                    time_zone?: string | null;
                    /** @description Its title. */
                    title: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Entry"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_calendar_entries_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_calendar_entries_id_occurrences_at: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                at: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description The occurrence is cancelled.
                     * @default false
                     */
                    cancelled?: boolean;
                    /**
                     * Format: date-time
                     * @description Another end.
                     * @default null
                     */
                    ends_at?: string | null;
                    /**
                     * Format: date-time
                     * @description Another start.
                     * @default null
                     */
                    starts_at?: string | null;
                    /**
                     * @description Another title for it.
                     * @default null
                     */
                    title?: string | null;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_calendar_entries_id_occurrences_at: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                at: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_calendar_feed_secret: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                secret: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "text/calendar": string;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_calendar_feeds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Feed"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_calendar_feeds: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description What its owner calls it. */
                    name: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["NewFeedAnswer"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_calendar_feeds_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_calendar_scopes_id_subscriptions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Subscription"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_calendar_scopes_id_subscriptions: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description Every how many minutes it is read.
                     * @default null
                     */
                    refresh_minutes?: number | null;
                    /** @description What it is called. */
                    title: string;
                    /** @description Where the calendar is read from. */
                    url: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Subscription"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_calendar_subscriptions_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_calendar_subscriptions_id_refresh: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Refreshed"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_kind_id_references: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
                kind: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_kind_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                kind: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The identifier. */
                    external_id: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_courses_kind_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                kind: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_campuses_campus_kind_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
                external_id: string;
                kind: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Found"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_campuses_campus_categories: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_campuses_campus_categories: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                campus: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * @description Its description.
                     * @default null
                     */
                    description?: string | null;
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: uuid
                     * @description Its parent; the top of the campus when absent.
                     * @default null
                     */
                    parent?: string | null;
                    /** @description Where among its siblings. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_campuses_campus_categories_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                campus: string;
                external_id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * @description Its description.
                     * @default null
                     */
                    description?: string | null;
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: uuid
                     * @description Its parent when it is created; the top of the campus when absent.
                     * @default null
                     */
                    parent?: string | null;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_categories_category_courses_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                category: string;
                external_id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * Format: date-time
                     * @description When it ends.
                     * @default null
                     */
                    ends_at?: string | null;
                    /** @description Its full name. */
                    full_name: string;
                    /**
                     * @description The language it is forced to.
                     * @default null
                     */
                    language_code?: string | null;
                    /**
                     * @description The layout plugin; the default layout when absent.
                     * @default null
                     */
                    layout?: string | null;
                    /** @description Its short name. */
                    short_name: string;
                    /**
                     * Format: date-time
                     * @description When it starts.
                     * @default null
                     */
                    starts_at?: string | null;
                    /**
                     * @description Its summary.
                     * @default null
                     */
                    summary?: string | null;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Course"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_categories_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_categories_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * @description Its description.
                     * @default null
                     */
                    description?: string | null;
                    /** @description Its name. */
                    name: string;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_courses_categories_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_categories_id_courses: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Course"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_categories_id_courses: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * Format: date-time
                     * @description When it ends.
                     * @default null
                     */
                    ends_at?: string | null;
                    /** @description Its full name. */
                    full_name: string;
                    /**
                     * @description The language it is forced to.
                     * @default null
                     */
                    language_code?: string | null;
                    /**
                     * @description The layout plugin; the default layout when absent.
                     * @default null
                     */
                    layout?: string | null;
                    /** @description Where among the courses of the category. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /** @description Its short name. */
                    short_name: string;
                    /**
                     * Format: date-time
                     * @description When it starts.
                     * @default null
                     */
                    starts_at?: string | null;
                    /**
                     * @description Its summary.
                     * @default null
                     */
                    summary?: string | null;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Course"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_categories_id_move: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The new parent; the top when absent.
                     * @default null
                     */
                    parent?: string | null;
                    /** @description Where among the siblings. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_courses_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["WithOwners"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_courses_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * Format: date-time
                     * @description When it ends.
                     * @default null
                     */
                    ends_at?: string | null;
                    /** @description Its full name. */
                    full_name: string;
                    /**
                     * @description The language it is forced to.
                     * @default null
                     */
                    language_code?: string | null;
                    /**
                     * @description The layout plugin; the default layout when absent.
                     * @default null
                     */
                    layout?: string | null;
                    /** @description Its short name. */
                    short_name: string;
                    /**
                     * Format: date-time
                     * @description When it starts.
                     * @default null
                     */
                    starts_at?: string | null;
                    /**
                     * @description Its summary.
                     * @default null
                     */
                    summary?: string | null;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Course"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_courses_id_fields: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FieldEntry"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_courses_id_fields_code: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                code: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The value, or null to clear it. */
                    value: unknown;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FieldEntry"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_courses_id_group_sets: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GroupSet"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_courses_id_group_sets: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /** @description Its name. */
                    name: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GroupSet"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_courses_id_groups: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_courses_id_groups: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * @description Its description.
                     * @default null
                     */
                    description?: string | null;
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: uuid
                     * @description The section it is limited to.
                     * @default null
                     */
                    section?: string | null;
                    /**
                     * @description Who sees it.
                     * @default everyone
                     */
                    visibility?: "everyone" | "members" | "own_only" | "nobody";
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_courses_id_move: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The category it goes to.
                     */
                    category: string;
                    /** @description Where among its courses. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Course"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_courses_id_sections: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Section"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_courses_id_sections: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The section it goes in; the top of the course when absent.
                     * @default null
                     */
                    parent?: string | null;
                    /** @description Where among its siblings. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                    /**
                     * @description Its summary.
                     * @default null
                     */
                    summary?: string | null;
                    /**
                     * @description Its title.
                     * @default null
                     */
                    title?: string | null;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Section"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_courses_id_state: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The state to go to. */
                    to: "draft" | "published" | "archived" | "deletion_scheduled";
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Course"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_group_sets_id_groups_group: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                group: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_courses_group_sets_id_groups_group: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                group: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_groups_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_groups_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its code.
                     * @default null
                     */
                    code?: string | null;
                    /**
                     * @description Its description.
                     * @default null
                     */
                    description?: string | null;
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: uuid
                     * @description The section it is limited to.
                     * @default null
                     */
                    section?: string | null;
                    /**
                     * @description Who sees it.
                     * @default everyone
                     */
                    visibility?: "everyone" | "members" | "own_only" | "nobody";
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Group"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_courses_groups_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_groups_id_members: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Member"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_groups_id_members: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The account.
                     */
                    account: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_courses_groups_id_members_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_courses_sections_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Section"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_courses_sections_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its summary.
                     * @default null
                     */
                    summary?: string | null;
                    /**
                     * @description Its title.
                     * @default null
                     */
                    title?: string | null;
                    /**
                     * @description Its own visibility.
                     * @default {
                     *       "mode": "shown"
                     *     }
                     */
                    visibility?: {
                        /** @constant */
                        mode: "shown";
                    } | {
                        /** @constant */
                        mode: "hidden";
                    } | {
                        /**
                         * Format: date-time
                         * @description Shown from.
                         */
                        from?: string | null;
                        /** @constant */
                        mode: "scheduled";
                        /**
                         * Format: date-time
                         * @description Shown until.
                         */
                        until?: string | null;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Section"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_courses_sections_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_sections_id_move: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The new parent; the top when absent.
                     * @default null
                     */
                    parent?: string | null;
                    /** @description Where among the siblings. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Section"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_courses_sections_id_restore: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Section"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_fields_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Field"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_customise_fields_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_imports_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Import"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_customise_imports_id_apply: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Import"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_rules_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Rule"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_customise_rules_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description What it does, in order. */
                    actions: {
                        /** @description The action a module declares, by code. */
                        action: string;
                        /**
                         * @description Its parameters, each of its declared kind.
                         * @default {}
                         */
                        params?: {
                            [key: string]: unknown;
                        };
                    }[];
                    /**
                     * @description When it runs; always when absent.
                     * @default null
                     */
                    conditions?: ({
                        all: components["schemas"]["Condition"][];
                    } | {
                        any: components["schemas"]["Condition"][];
                    } | {
                        not: components["schemas"]["Condition"];
                    } | {
                        /** @description A comparison of a field of the payload of the event. */
                        compare: {
                            /** @description A field the event type declares. */
                            field: string;
                            /** @description How it is compared. */
                            op: "eq" | "ne" | "in" | "exists";
                            /**
                             * @description What it is compared with; a list for `in`, nothing for `exists`.
                             * @default null
                             */
                            value?: unknown;
                        };
                    }) | null;
                    /**
                     * @description Whether it runs.
                     * @default true
                     */
                    enabled?: boolean;
                    /** @description Its name. */
                    name: string;
                    /** @description The event type that triggers it. */
                    trigger: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Rule"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_customise_rules_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_rules_id_runs: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RuleRun"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_fields: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Field"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_customise_scopes_scope_fields: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Its choices, for choice fields.
                     * @default []
                     */
                    choices?: {
                        /** @description Its code, stable while its label changes. */
                        code: string;
                        /** @description What it says. */
                        label: string;
                    }[];
                    /** @description Its code. */
                    code: string;
                    /** @description The kind of entity it belongs to. */
                    kind: string;
                    /**
                     * Format: int32
                     * @description The longest text, for text fields.
                     * @default null
                     */
                    max_length?: number | null;
                    /** @description Its name. */
                    name: string;
                    /**
                     * @description Whether every entity has a value.
                     * @default false
                     */
                    required?: boolean;
                    /** @description Its type. */
                    type: "text" | "integer" | "date" | "boolean" | "choice" | "choices" | "account";
                    /**
                     * @description Who sees its values.
                     * @default public
                     */
                    visibility?: "public" | "private";
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Field"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_flags: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Flag"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_flags_code: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                code: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Flag"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_customise_scopes_scope_flags_code: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                code: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Whether the feature is on. */
                    enabled: boolean;
                    /**
                     * @description Whether the scopes below keep it.
                     * @default false
                     */
                    locked?: boolean;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Flag"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_customise_scopes_scope_imports: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description The custom fields.
                     * @default []
                     */
                    fields?: {
                        /**
                         * @description Its choices, for choice fields.
                         * @default []
                         */
                        choices?: {
                            /** @description Its code, stable while its label changes. */
                            code: string;
                            /** @description What it says. */
                            label: string;
                        }[];
                        /** @description Its code. */
                        code: string;
                        /** @description The kind of entity it belongs to. */
                        kind: string;
                        /**
                         * Format: int32
                         * @description The longest text, for text fields.
                         * @default null
                         */
                        max_length?: number | null;
                        /** @description Its name. */
                        name: string;
                        /**
                         * @description Whether every entity has a value.
                         * @default false
                         */
                        required?: boolean;
                        /** @description Its type. */
                        type: "text" | "integer" | "date" | "boolean" | "choice" | "choices" | "account";
                        /**
                         * @description Who sees its values.
                         * @default public
                         */
                        visibility?: "public" | "private";
                    }[];
                    /**
                     * @description The values of feature flags.
                     * @default []
                     */
                    flags?: {
                        /** @description The flag. */
                        code: string;
                        /** @description Whether the feature is on. */
                        enabled: boolean;
                        /**
                         * @description Whether the scopes below keep it.
                         * @default false
                         */
                        locked?: boolean;
                    }[];
                    /**
                     * Format: uint16
                     * @description The format it is written in.
                     */
                    format: number;
                    /**
                     * @description The layouts.
                     * @default []
                     */
                    layouts?: {
                        /**
                         * @description Whether the scopes below keep it.
                         * @default false
                         */
                        locked?: boolean;
                        /** @description The page kind. */
                        page_kind: string;
                        /** @description Its widgets, in order. */
                        placements: {
                            /**
                             * @description Whether its height is the height of its content rather than its rows, as its widget declares on the page kind;
                             *     given by the server and ignored when a layout is written.
                             * @default false
                             */
                            grows?: boolean;
                            /**
                             * Format: uint16
                             * @description How many rows it takes, within the sizes the widget declares for the page kind.
                             */
                            h: number;
                            /**
                             * @description Whether a scope above locked it on this page, so it shows as that scope placed it; given by the server and
                             *     ignored when a layout is written.
                             * @default false
                             */
                            held?: boolean;
                            /**
                             * @description Whether the scope it is placed in locks it for the scopes below: there it shows as placed here, and it cannot
                             *     be moved, resized, edited or removed.
                             * @default false
                             */
                            locked?: boolean;
                            /**
                             * @description The permission a person needs to see it; everyone who sees the page when absent.
                             * @default null
                             */
                            permission?: string | null;
                            /**
                             * @description Whether the person pinned it where it is; otherwise it rises to the widgets above it, so no row is left empty.
                             * @default false
                             */
                            pinned?: boolean;
                            /**
                             * @description Its settings, each of the kind the widget declares; the defaults fill what is left out.
                             * @default {}
                             */
                            settings?: {
                                [key: string]: unknown;
                            };
                            /**
                             * Format: uint16
                             * @description How many columns it takes, within the sizes the widget declares for the page kind.
                             */
                            w: number;
                            /** @description The widget, by code. */
                            widget: string;
                            /**
                             * Format: uint16
                             * @description The first column it takes, from 0.
                             */
                            x: number;
                            /**
                             * Format: uint16
                             * @description The first row it takes, from 0.
                             */
                            y: number;
                        }[];
                    }[];
                    /**
                     * @description The menus.
                     * @default []
                     */
                    menus?: {
                        /** @description Its items, in order. */
                        items: {
                            /**
                             * @description The items under it, in order.
                             * @default []
                             */
                            children?: components["schemas"]["MenuItem"][];
                            /** @description What it says. */
                            label: string;
                            /**
                             * @description The permission a person needs to see it, and the items under it; everyone when absent.
                             * @default null
                             */
                            permission?: string | null;
                            /** @description A path of the site (`/…`) or an `http`/`https` address. */
                            target: string;
                        }[];
                        /** @description The kind of menu. */
                        kind: string;
                        /**
                         * @description Whether the scopes below keep it.
                         * @default false
                         */
                        locked?: boolean;
                    }[];
                    /**
                     * @description The rules, which the importer owns once applied.
                     * @default []
                     */
                    rules?: {
                        /** @description What it does, in order. */
                        actions: {
                            /** @description The action a module declares, by code. */
                            action: string;
                            /**
                             * @description Its parameters, each of its declared kind.
                             * @default {}
                             */
                            params?: {
                                [key: string]: unknown;
                            };
                        }[];
                        /**
                         * @description When it runs; always when absent.
                         * @default null
                         */
                        conditions?: ({
                            all: components["schemas"]["Condition"][];
                        } | {
                            any: components["schemas"]["Condition"][];
                        } | {
                            not: components["schemas"]["Condition"];
                        } | {
                            /** @description A comparison of a field of the payload of the event. */
                            compare: {
                                /** @description A field the event type declares. */
                                field: string;
                                /** @description How it is compared. */
                                op: "eq" | "ne" | "in" | "exists";
                                /**
                                 * @description What it is compared with; a list for `in`, nothing for `exists`.
                                 * @default null
                                 */
                                value?: unknown;
                            };
                        }) | null;
                        /**
                         * @description Whether it runs.
                         * @default true
                         */
                        enabled?: boolean;
                        /** @description Its name. */
                        name: string;
                        /** @description The event type that triggers it. */
                        trigger: string;
                    }[];
                    /**
                     * @description The words of terms.
                     * @default []
                     */
                    terms?: {
                        /** @description The term. */
                        code: string;
                        /** @description The language. */
                        language: string;
                        /**
                         * @description Whether the scopes below keep it.
                         * @default false
                         */
                        locked?: boolean;
                        /** @description The word for several. */
                        plural: string;
                        /** @description The word for one. */
                        singular: string;
                    }[];
                    /**
                     * @description The theme.
                     * @default null
                     */
                    theme?: {
                        /**
                         * @description Whether the scopes below keep them.
                         * @default false
                         */
                        locked?: boolean;
                        /** @description The tokens, by name. */
                        tokens: {
                            [key: string]: unknown;
                        };
                    } | null;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Import"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_layouts_page_kind: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                page_kind: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Layout"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_customise_scopes_scope_layouts_page_kind: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                page_kind: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether the scopes below keep it.
                     * @default false
                     */
                    locked?: boolean;
                    /** @description Its widgets, each a rectangle of cells of the grid. */
                    placements: {
                        /**
                         * @description Whether its height is the height of its content rather than its rows, as its widget declares on the page kind;
                         *     given by the server and ignored when a layout is written.
                         * @default false
                         */
                        grows?: boolean;
                        /**
                         * Format: uint16
                         * @description How many rows it takes, within the sizes the widget declares for the page kind.
                         */
                        h: number;
                        /**
                         * @description Whether a scope above locked it on this page, so it shows as that scope placed it; given by the server and
                         *     ignored when a layout is written.
                         * @default false
                         */
                        held?: boolean;
                        /**
                         * @description Whether the scope it is placed in locks it for the scopes below: there it shows as placed here, and it cannot
                         *     be moved, resized, edited or removed.
                         * @default false
                         */
                        locked?: boolean;
                        /**
                         * @description The permission a person needs to see it; everyone who sees the page when absent.
                         * @default null
                         */
                        permission?: string | null;
                        /**
                         * @description Whether the person pinned it where it is; otherwise it rises to the widgets above it, so no row is left empty.
                         * @default false
                         */
                        pinned?: boolean;
                        /**
                         * @description Its settings, each of the kind the widget declares; the defaults fill what is left out.
                         * @default {}
                         */
                        settings?: {
                            [key: string]: unknown;
                        };
                        /**
                         * Format: uint16
                         * @description How many columns it takes, within the sizes the widget declares for the page kind.
                         */
                        w: number;
                        /** @description The widget, by code. */
                        widget: string;
                        /**
                         * Format: uint16
                         * @description The first column it takes, from 0.
                         */
                        x: number;
                        /**
                         * Format: uint16
                         * @description The first row it takes, from 0.
                         */
                        y: number;
                    }[];
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Layout"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_customise_scopes_scope_layouts_page_kind: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                page_kind: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Layout"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_menus_kind: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                kind: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Menu"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_customise_scopes_scope_menus_kind: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                kind: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its items, in order. */
                    items: {
                        /**
                         * @description The items under it, in order.
                         * @default []
                         */
                        children?: components["schemas"]["MenuItem"][];
                        /** @description What it says. */
                        label: string;
                        /**
                         * @description The permission a person needs to see it, and the items under it; everyone when absent.
                         * @default null
                         */
                        permission?: string | null;
                        /** @description A path of the site (`/…`) or an `http`/`https` address. */
                        target: string;
                    }[];
                    /**
                     * @description Whether the scopes below keep it.
                     * @default false
                     */
                    locked?: boolean;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Menu"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_customise_scopes_scope_menus_kind: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                kind: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Menu"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_customise_scopes_scope_packages: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The kinds of customisation the package holds. */
                    kinds: ("terms" | "flags" | "fields" | "layouts" | "menus" | "theme" | "rules")[];
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Exported"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_rules: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Rule"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_customise_scopes_scope_rules: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description What it does, in order. */
                    actions: {
                        /** @description The action a module declares, by code. */
                        action: string;
                        /**
                         * @description Its parameters, each of its declared kind.
                         * @default {}
                         */
                        params?: {
                            [key: string]: unknown;
                        };
                    }[];
                    /**
                     * @description When it runs; always when absent.
                     * @default null
                     */
                    conditions?: ({
                        all: components["schemas"]["Condition"][];
                    } | {
                        any: components["schemas"]["Condition"][];
                    } | {
                        not: components["schemas"]["Condition"];
                    } | {
                        /** @description A comparison of a field of the payload of the event. */
                        compare: {
                            /** @description A field the event type declares. */
                            field: string;
                            /** @description How it is compared. */
                            op: "eq" | "ne" | "in" | "exists";
                            /**
                             * @description What it is compared with; a list for `in`, nothing for `exists`.
                             * @default null
                             */
                            value?: unknown;
                        };
                    }) | null;
                    /**
                     * @description Whether it runs.
                     * @default true
                     */
                    enabled?: boolean;
                    /** @description Its name. */
                    name: string;
                    /** @description The event type that triggers it. */
                    trigger: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Rule"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_terms_code: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                code: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Term"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_customise_scopes_scope_terms_code: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                code: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description The language.
                     * @default en
                     */
                    language?: string;
                    /**
                     * @description Whether the scopes below keep it.
                     * @default false
                     */
                    locked?: boolean;
                    /** @description The word for several. */
                    plural: string;
                    /** @description The word for one. */
                    singular: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Term"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_customise_scopes_scope_terms_code: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                code: string;
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Term"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_theme: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Theme"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_customise_scopes_scope_theme: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether the scopes below keep them.
                     * @default false
                     */
                    locked?: boolean;
                    /** @description The tokens this scope sets. */
                    tokens: {
                        [key: string]: unknown;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Theme"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_customise_scopes_scope_theme: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Theme"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_customise_scopes_scope_theme_css: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "text/css": string;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_campuses_campus_enrolments_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
                external_id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Enrolment"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_courses_id_enrolment_instances: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Instance"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_enrol_courses_id_enrolment_instances: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description How many enrolments it holds at most.
                     * @default null
                     */
                    capacity?: number | null;
                    /**
                     * Format: int32
                     * @description How many days an enrolment lasts when it gives no end.
                     * @default null
                     */
                    default_days?: number | null;
                    /**
                     * @description Whether it gives participation.
                     * @default true
                     */
                    enabled?: boolean;
                    /**
                     * @description What happens when an enrolment ends.
                     * @default suspend
                     */
                    expiry_action?: "suspend" | "remove" | "keep_without_role";
                    /**
                     * @description The key of a self instance.
                     * @default null
                     */
                    key?: string | null;
                    /** @description Its method. */
                    method: "manual" | "self" | "system_group";
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: date-time
                     * @description When people may start joining.
                     * @default null
                     */
                    open_from?: string | null;
                    /**
                     * Format: date-time
                     * @description When people may no longer join.
                     * @default null
                     */
                    open_until?: string | null;
                    /**
                     * @description Whether joining waits for an approval.
                     * @default false
                     */
                    requires_approval?: boolean;
                    /**
                     * Format: uuid
                     * @description The role its enrolments give.
                     */
                    role: string;
                    /**
                     * Format: uuid
                     * @description The system group a system group instance follows.
                     * @default null
                     */
                    source?: string | null;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Instance"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_courses_id_enrolment_requests: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_courses_id_participants: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Enrolment"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_enrolment_instances_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Instance"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_enrol_enrolment_instances_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description How many enrolments it holds at most.
                     * @default null
                     */
                    capacity?: number | null;
                    /**
                     * Format: int32
                     * @description How many days an enrolment lasts when it gives no end.
                     * @default null
                     */
                    default_days?: number | null;
                    /**
                     * @description Whether it gives participation.
                     * @default true
                     */
                    enabled?: boolean;
                    /**
                     * @description What happens when an enrolment ends.
                     * @default suspend
                     */
                    expiry_action?: "suspend" | "remove" | "keep_without_role";
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: date-time
                     * @description When people may start joining.
                     * @default null
                     */
                    open_from?: string | null;
                    /**
                     * Format: date-time
                     * @description When people may no longer join.
                     * @default null
                     */
                    open_until?: string | null;
                    /**
                     * @description Whether joining waits for an approval.
                     * @default false
                     */
                    requires_approval?: boolean;
                    /**
                     * Format: uuid
                     * @description The role its enrolments give.
                     */
                    role: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Instance"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_enrol_enrolment_instances_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_enrol_enrolment_instances_id_enrolments: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Their accounts. */
                    accounts: string[];
                    /**
                     * Format: date-time
                     * @description When it ends; the instance's default duration, or open, when absent.
                     * @default null
                     */
                    ends_at?: string | null;
                    /**
                     * Format: date-time
                     * @description When it starts; now when absent.
                     * @default null
                     */
                    starts_at?: string | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Enrolled"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_enrol_enrolment_instances_id_enrolments_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                external_id: string;
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The account.
                     */
                    account: string;
                    /**
                     * Format: date-time
                     * @description When it ends; the instance's default duration, or open, when absent.
                     * @default null
                     */
                    ends_at?: string | null;
                    /**
                     * Format: date-time
                     * @description When it starts; now when absent.
                     * @default null
                     */
                    starts_at?: string | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Enrolment"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_enrol_enrolment_instances_id_keys: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The group the key puts people in; the whole course when absent.
                     * @default null
                     */
                    group?: string | null;
                    /**
                     * @description The key; removed when absent.
                     * @default null
                     */
                    key?: string | null;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_enrol_enrolment_instances_id_self: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description The key, when the instance has one.
                     * @default null
                     */
                    key?: string | null;
                    /**
                     * @description A message for whoever approves.
                     * @default null
                     */
                    message?: string | null;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Joined"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_enrol_enrolment_instances_id_sync: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Synced"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_enrol_enrolment_requests_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_enrol_enrolment_requests_id_decision: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Whether the request is approved. */
                    approve: boolean;
                    /**
                     * @description Why.
                     * @default null
                     */
                    reason?: string | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_enrolments_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Enrolment"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_enrol_enrolments_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: date-time
                     * @description When it ends; open when absent.
                     * @default null
                     */
                    ends_at?: string | null;
                    /**
                     * @description Why.
                     * @default null
                     */
                    reason?: string | null;
                    /**
                     * Format: date-time
                     * @description When it starts; unchanged when absent.
                     * @default null
                     */
                    starts_at?: string | null;
                    /** @description Its status. */
                    status: "active" | "suspended";
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Enrolment"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_enrol_enrolments_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_enrol_enrolments_id_references: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_enrol_enrolments_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The identifier. */
                    external_id: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_enrol_enrolments_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_files_content_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/octet-stream": string;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_files_files_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["File"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_files_files_id_conversions_kind: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
                kind: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conversion"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_files_files_id_url: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SignedUrl"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_files_people_id_quota: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Quota"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_files_people_id_quota: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int64
                     * @description The most, in bytes; no limit when null.
                     */
                    limit_bytes?: number | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Quota"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_files_scopes_scope_quota: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Quota"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_files_scopes_scope_quota: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int64
                     * @description The most, in bytes; no limit when null.
                     */
                    limit_bytes?: number | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Quota"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_files_uploads: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The area. */
                    area: string;
                    /**
                     * @description The type the sender says it has; the content decides.
                     * @default null
                     */
                    media_type?: string | null;
                    /** @description The name of the file. */
                    name: string;
                    /**
                     * Format: uuid
                     * @description The scope it is for.
                     */
                    scope: string;
                    /**
                     * Format: int64
                     * @description Its size in bytes.
                     */
                    size: number;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Upload"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_files_uploads_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/octet-stream": string;
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Upload"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_files_uploads_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["File"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_accounts_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Account"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_accounts_id_references: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_identity_accounts_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The identifier. */
                    external_id: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_identity_accounts_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_accounts_id_tokens: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: date-time
                     * @description When it expires.
                     */
                    expires_at?: string | null;
                    /** @description Its name. */
                    name: string;
                    /** @description What it may do. */
                    scopes: {
                        /** @description The permission. */
                        permission: string;
                        /** @description The scope, by id. */
                        scope: string;
                    }[];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["NewToken"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_accounts_id_transitions: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Why; kept with the change.
                     * @default
                     */
                    reason?: string;
                    /** @description `suspended`, `active`, `deletion_scheduled` or `deletion_cancelled`. */
                    to: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_confirm: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The password. */
                    password: string;
                    /** @description The token of the link. */
                    token: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_email_confirm: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The token of the link. */
                    token: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_invitations_accept: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The password. */
                    password: string;
                    /** @description The token of the link. */
                    token: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_auth_oidc_provider_callback: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                provider: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SignInAnswer"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_auth_oidc_provider_start: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                provider: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description A redirect to the address in `Location` */
            303: {
                headers: {
                    Location?: string;
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_recovery: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The email. */
                    email: string;
                };
            };
        };
        responses: {
            /** @description Accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_recovery_complete: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The new password. */
                    password: string;
                    /** @description The second factor, when the account has one. */
                    proof?: ({
                        /** @description The code. */
                        code: string;
                        /** @constant */
                        kind: "totp";
                    } | {
                        /** @description The code. */
                        code: string;
                        /** @constant */
                        kind: "recovery_code";
                    } | {
                        /** @description The code. */
                        code: string;
                        /** @constant */
                        kind: "email";
                    } | {
                        /** @description A current code. */
                        code: string;
                        /** @constant */
                        kind: "enrol_totp";
                        /** @description Its name. */
                        label: string;
                        /** @description The secret shown. */
                        secret: string;
                    }) | null;
                    /** @description The token of the link. */
                    token: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_recovery_needs: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The token of the link. */
                    token: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RecoveryNeeds"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_second_factor: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The challenge of the sign-in. */
                    challenge: string;
                    /** @description The factor. */
                    proof: {
                        /** @description The code. */
                        code: string;
                        /** @constant */
                        kind: "totp";
                    } | {
                        /** @description The code. */
                        code: string;
                        /** @constant */
                        kind: "recovery_code";
                    } | {
                        /** @description The code. */
                        code: string;
                        /** @constant */
                        kind: "email";
                    } | {
                        /** @description A current code. */
                        code: string;
                        /** @constant */
                        kind: "enrol_totp";
                        /** @description Its name. */
                        label: string;
                        /** @description The secret shown. */
                        secret: string;
                    };
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SignInAnswer"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_second_factor_email_code: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The challenge of the sign-in. */
                    challenge: string;
                };
            };
        };
        responses: {
            /** @description Accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_sign_in: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The username or email. */
                    name: string;
                    /** @description The password. */
                    password: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SignInAnswer"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_sign_out: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_sign_up: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether the campus terms were accepted.
                     * @default false
                     */
                    accepted_terms?: boolean;
                    /** @description The email. */
                    email: string;
                };
            };
        };
        responses: {
            /** @description Accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_auth_step_up: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The password. */
                    password: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SteppedUp"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_campuses_campus_accounts: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                campus: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The email. */
                    email: string;
                    /** @description The username; the email when absent. */
                    username?: string | null;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Identified"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_campuses_campus_accounts_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
                external_id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Account"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_campuses_campus_identity_providers: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Provider"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_campuses_campus_identity_providers: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                campus: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The client id at the provider. */
                    client: string;
                    /** @description The client secret. */
                    client_secret: string;
                    /** @description The issuer URL. */
                    issuer: string;
                    /** @description `subject_only`, `verified_email` or `confirm`. */
                    linking_policy: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Identified2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_campuses_campus_service_accounts: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                campus: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its username. */
                    username: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Identified"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_me: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Account"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_email: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The email. */
                    email: string;
                };
            };
        };
        responses: {
            /** @description Accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_identity_links: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The sealed link. */
                    link: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_password: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The current password. */
                    current: string;
                    /** @description The new password. */
                    new: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_recovery_codes: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RecoveryCodes"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_me_second_factors: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SecondFactor"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_identity_me_second_factors_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_second_factors_email: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The name. */
                    label: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Added"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_second_factors_totp: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TotpSetup"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_second_factors_totp_confirm: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description A current code. */
                    code: string;
                    /** @description Its name. */
                    label: string;
                    /** @description The secret shown. */
                    secret: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Added"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_me_sessions: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SessionSummary"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_identity_me_sessions_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_identity_me_tokens: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TokenSummary"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_identity_me_tokens: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: date-time
                     * @description When it expires.
                     */
                    expires_at?: string | null;
                    /** @description Its name. */
                    name: string;
                    /** @description What it may do. */
                    scopes: {
                        /** @description The permission. */
                        permission: string;
                        /** @description The scope, by id. */
                        scope: string;
                    }[];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["NewToken"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_identity_me_tokens_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_identity_tokens_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_blocks: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": string[];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_notify_blocks_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_notify_blocks_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_campuses_id_defaults: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CampusDefault"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_notify_campuses_id_defaults: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The channel. */
                    channel: string;
                    /** @description Whether people receive it. */
                    enabled: boolean;
                    /**
                     * @description Whether people may change it.
                     * @default false
                     */
                    locked?: boolean;
                    /** @description The type. */
                    type: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CampusDefault"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_campuses_id_templates: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Template"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_notify_campuses_id_templates: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The text, with placeholders taken from the notification. */
                    body: string;
                    /** @description The channel, `email` or `push`. */
                    channel: string;
                    /** @description The language, such as `es`. */
                    locale: string;
                    /** @description The subject, or the title of a push notification, with placeholders like `{{summary}}`. */
                    subject: string;
                    /** @description The type. */
                    type: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Template"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_notify_campuses_id_templates_type_code_channel_locale: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                channel: string;
                id: string;
                locale: string;
                type_code: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_conversations: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Page"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_notify_conversations: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The other person.
                     */
                    account: string;
                    /** @constant */
                    kind: "direct";
                } | {
                    /** @constant */
                    kind: "group";
                    /** @description The other people. */
                    members: string[];
                    /** @description Its title. */
                    title: string;
                } | {
                    /**
                     * Format: uuid
                     * @description The group.
                     */
                    group: string;
                    /** @constant */
                    kind: "course_group";
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conversation"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_conversations_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Conversation"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_notify_conversations_id_membership: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_conversations_id_messages: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Page2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_notify_conversations_id_messages: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description What it says, in rich text.
                     * @default
                     */
                    body?: string;
                    /**
                     * @description The files uploaded to the area of messages to attach, in order.
                     * @default []
                     */
                    files?: string[];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Message"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_notify_conversations_id_mute: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Muted or not. */
                    muted: boolean;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_notify_conversations_id_read: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_inbox: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Inbox"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_notify_inbox_id_read: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Notification"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_notify_inbox_read_all: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ReadAll"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_notify_messages_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_preferences: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Preference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_notify_preferences_type_code_channel: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                channel: string;
                type_code: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Whether the person receives it. */
                    enabled: boolean;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Preference"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_push_key: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PushKey"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_notify_push_subscriptions: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PushSubscription"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_notify_push_subscriptions: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its authentication secret, as the browser gives it. */
                    auth: string;
                    /** @description Where its push service receives messages for it. */
                    endpoint: string;
                    /**
                     * Format: uuid
                     * @description The subscription.
                     * @default null
                     */
                    id?: string | null;
                    /** @description Its public key, as the browser gives it. */
                    p256dh: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PushSubscription"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_notify_push_subscriptions_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_holds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Hold"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_privacy_holds: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The account it protects; the whole campus of the caller when left out.
                     * @default null
                     */
                    account?: string | null;
                    /** @description Why deletion waits. */
                    reason: string;
                    /**
                     * Format: date-time
                     * @description When it ends by itself.
                     * @default null
                     */
                    until?: string | null;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Hold"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_privacy_holds_id_release: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Hold"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_policies: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Policy"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_privacy_policies: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description What it says. */
                    body: string;
                    /** @description The code of the policy. */
                    code: string;
                    /** @description Its title. */
                    title: string;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Policy"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_privacy_policies_code_acceptance: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                code: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description The version.
                     */
                    version: number;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Policy"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_policies_pending: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Policy"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_requests: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request2"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_privacy_requests: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description Whose data; the caller when left out.
                     * @default null
                     */
                    account?: string | null;
                    /**
                     * @description `export` (the default) or `deletion`.
                     * @default null
                     */
                    kind?: string | null;
                    /**
                     * @description Why a deletion is asked for.
                     * @default null
                     */
                    reason?: string | null;
                };
            };
        };
        responses: {
            /** @description Accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_requests_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_privacy_requests_id_decision: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Whether the deletion goes ahead. */
                    approve: boolean;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_requests_id_package: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": unknown;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_requests_awaiting: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Request2"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_privacy_retention: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Retention"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_privacy_retention_category: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                category: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description Days after the account is deleted; none keeps the data for ever.
                     * @default null
                     */
                    keep_days?: number | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Retention"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_course_modules_id_exceptions: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Exception2"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_course_modules_id_exceptions_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Why. */
                    reason: string;
                    /** @description The state it sets. */
                    state: "not_started" | "in_progress" | "done" | "done_passed" | "done_failed";
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_progress_course_modules_id_exceptions_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_course_modules_id_mark: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_progress_course_modules_id_mark: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_course_modules_id_rules: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RuleKind"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_course_modules_id_rules: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description What the course module requires. */
                    kinds: ("manual" | "view" | "submit" | "grade" | "pass")[];
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RuleKind"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_course_modules_id_states_account: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_course_modules_id_type_rules: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TypeRulesAnswer"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_course_modules_id_type_rules: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Their codes. */
                    rules: string[];
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": string[];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_progress_course_modules_id_views: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_course_progress: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CourseProgress"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_course_progress_account: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CourseProgress"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_criteria: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CourseCriteria"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_courses_id_criteria: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description `all` of the criteria, or `any` of them.
                     * @default all
                     */
                    aggregation?: string;
                    /** @description The criteria, in order. */
                    criteria: ({
                        /** @constant */
                        kind: "activity";
                        /**
                         * Format: uuid
                         * @description The course module.
                         */
                        module: string;
                        /** @description `done` or `passed`. */
                        required: string;
                    } | {
                        /** @constant */
                        kind: "grade";
                        /** @description The share, from 0 to 100. */
                        min_percent: string | number;
                        /**
                         * Format: uuid
                         * @description The grade record.
                         */
                        record: string;
                    } | {
                        /** @constant */
                        kind: "date";
                        /**
                         * Format: date-time
                         * @description From when it holds.
                         */
                        on_or_after: string;
                    } | {
                        /**
                         * Format: int32
                         * @description The days.
                         */
                        enrolled_days: number;
                        /** @constant */
                        kind: "duration";
                    } | {
                        /** @constant */
                        kind: "role_approval";
                        /**
                         * Format: uuid
                         * @description The role.
                         */
                        role: string;
                    } | {
                        /**
                         * Format: uuid
                         * @description The other course.
                         */
                        course: string;
                        /** @constant */
                        kind: "course";
                    })[];
                    /**
                     * @description Whether a completion stays when a criterion stops being met.
                     * @default false
                     */
                    final?: boolean;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CourseCriteria"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_my_course_progress: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CourseProgress"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_my_progress: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_prerequisites: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Prerequisites"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_courses_id_prerequisites: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The courses. */
                    requires: string[];
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Prerequisites"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_progress_courses_id_states_account: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ActivityState"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_progress_criteria_id_approvals_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_exports: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Export"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_records_courses_id_exports: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Also the people who no longer take part.
                     * @default false
                     */
                    include_inactive?: boolean;
                };
            };
        };
        responses: {
            /** @description Accepted */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Export"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_grader_report: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Report"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_grades_account: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_letters: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LetterSet"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_my_grades: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_records: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Tree"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_records_courses_id_records_categories: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description How many of the lowest grades it drops.
                     * @default 0
                     */
                    drop_lowest?: number;
                    /**
                     * @description Whether an empty grade is left out instead of counted as zero.
                     * @default true
                     */
                    exclude_empty?: boolean;
                    /**
                     * Format: int32
                     * @description How many of the highest grades it keeps; zero keeps all.
                     * @default 0
                     */
                    keep_highest?: number;
                    /**
                     * @description How it aggregates.
                     * @default natural
                     */
                    method?: "mean" | "weighted_mean" | "simple_weighted_mean" | "natural" | "median" | "lowest" | "highest" | "mode";
                    /** @description Its name. */
                    name: string;
                    /**
                     * Format: uuid
                     * @description The category it goes under; the root of the course when left out.
                     * @default null
                     */
                    parent?: string | null;
                    /** @description Where among its siblings. */
                    place?: {
                        /** @constant */
                        at: "first";
                    } | {
                        /** @constant */
                        at: "last";
                    } | {
                        /** @constant */
                        at: "after";
                        /** Format: uuid */
                        sibling: string;
                    } | {
                        /** @constant */
                        at: "before";
                        /** Format: uuid */
                        sibling: string;
                    };
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_records_courses_id_records_items: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description The category it goes in; the root of the course when left out.
                     * @default null
                     */
                    category?: string | null;
                    /**
                     * @description Whether it counts on top of the maximum of its category.
                     * @default false
                     */
                    extra_credit?: boolean;
                    /**
                     * @description What its grades are made of.
                     * @default value
                     */
                    grade_type?: "value" | "scale" | "text";
                    /**
                     * Format: date-time
                     * @description Hidden until then.
                     * @default null
                     */
                    hidden_until?: string | null;
                    /**
                     * @description The highest grade.
                     * @default 100
                     */
                    max?: string | number;
                    /**
                     * @description The lowest grade.
                     * @default 0
                     */
                    min?: string | number;
                    /**
                     * @description What its grades are multiplied by.
                     * @default 1
                     */
                    multiplier?: string | number;
                    /**
                     * @description What is added to its grades after multiplying.
                     * @default 0
                     */
                    offset?: string | number;
                    /**
                     * @description The grade that passes.
                     * @default null
                     */
                    pass_grade?: string | number | null;
                    /**
                     * Format: uuid
                     * @description The scale its grades are items of; required for a scale record, whose range is the items.
                     * @default null
                     */
                    scale?: string | null;
                    /** @description Its title. */
                    title: string;
                    /**
                     * @description Its weight in the category.
                     * @default 1
                     */
                    weight?: string | number;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Record"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_courses_id_scales: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Scale"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_exports_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Export"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_record_categories_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: int32
                     * @description How many of the lowest grades it drops.
                     * @default 0
                     */
                    drop_lowest?: number;
                    /**
                     * @description Whether an empty grade is left out instead of counted as zero.
                     * @default true
                     */
                    exclude_empty?: boolean;
                    /**
                     * Format: int32
                     * @description How many of the highest grades it keeps; zero keeps all.
                     * @default 0
                     */
                    keep_highest?: number;
                    /**
                     * @description How it aggregates.
                     * @default natural
                     */
                    method?: "mean" | "weighted_mean" | "simple_weighted_mean" | "natural" | "median" | "lowest" | "highest" | "mode";
                    /** @description Its name. */
                    name: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Category2"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_record_categories_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_records_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Record"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description Whether it counts on top of the maximum of its category.
                     * @default false
                     */
                    extra_credit?: boolean;
                    /**
                     * @description What its grades are made of.
                     * @default value
                     */
                    grade_type?: "value" | "scale" | "text";
                    /**
                     * Format: date-time
                     * @description Hidden until then.
                     * @default null
                     */
                    hidden_until?: string | null;
                    /**
                     * @description The highest grade.
                     * @default 100
                     */
                    max?: string | number;
                    /**
                     * @description The lowest grade.
                     * @default 0
                     */
                    min?: string | number;
                    /**
                     * @description What its grades are multiplied by.
                     * @default 1
                     */
                    multiplier?: string | number;
                    /**
                     * @description What is added to its grades after multiplying.
                     * @default 0
                     */
                    offset?: string | number;
                    /**
                     * @description The grade that passes.
                     * @default null
                     */
                    pass_grade?: string | number | null;
                    /**
                     * Format: uuid
                     * @description The scale its grades are items of; required for a scale record, whose range is the items.
                     * @default null
                     */
                    scale?: string | null;
                    /** @description Its title. */
                    title: string;
                    /**
                     * @description Its weight in the category.
                     * @default 1
                     */
                    weight?: string | number;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Record"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_records_id_formula: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Formula"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_formula: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The expression, with the records it reads written `[[<record id>]]`. */
                    expression: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Formula"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id_formula: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_grades: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The grades. */
                    grades: {
                        /**
                         * Format: uuid
                         * @description The person.
                         */
                        account: string;
                        /**
                         * @description What the grader says about it.
                         * @default null
                         */
                        feedback?: string | null;
                        /**
                         * @description The grade, for a numeric record.
                         * @default null
                         */
                        raw?: string | number | null;
                        /**
                         * @description The text, for a text record.
                         * @default null
                         */
                        text?: string | null;
                    }[];
                    /**
                     * @description Where they come from; those a teacher sets unless an import says otherwise.
                     * @default null
                     */
                    source?: ("activity" | "teacher" | "import" | "formula") | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GradeChange"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_records_id_grades_account: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Detail"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_grades_account_excluded: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id_grades_account_excluded: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_grades_account_hidden: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: date-time
                     * @description The date.
                     */
                    until: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id_grades_account_hidden: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_grades_account_lock: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id_grades_account_lock: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Grade"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_grades_account_override: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * @description What the grader says about it.
                     * @default null
                     */
                    feedback?: string | null;
                    /** @description The grade that counts. */
                    value: string | number;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GradeChange"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id_grades_account_override: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GradeChange"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_records_records_id_lock: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Record"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_records_records_id_lock: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Record"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_scopes_id_letters: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LetterSet"] | null;
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_records_scopes_id_letters: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its bands. */
                    bands: {
                        /** @description The letter. */
                        letter: string;
                        /** @description The lowest percentage of the band. */
                        min_percent: string | number;
                    }[];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LetterSet"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_records_scopes_id_scales: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Scale"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_records_scopes_id_scales: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its code. */
                    code: string;
                    /** @description Its items, from the lowest to the highest. */
                    items: string[];
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Scale"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_accounts_id_picture: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PictureFile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_accounts_id_profile: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_users_accounts_id_profile: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description How the person is shown. */
                    display_name?: string | null;
                    /** @description The family name. */
                    family_name?: string | null;
                    /** @description The given name. */
                    given_name?: string | null;
                    /** @description A BCP 47 language tag. */
                    language_code?: string | null;
                    /** @description An IANA time zone name. */
                    time_zone?: string | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_campuses_campus_system_groups_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                campus: string;
                external_id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SystemGroup"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_users_me_picture: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /**
                     * Format: uuid
                     * @description A file uploaded into `users.picture`.
                     */
                    file: string;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_users_me_picture: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_me_preferences: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": {
                        [key: string]: unknown;
                    };
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    patch_users_me_preferences: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    [key: string]: unknown;
                };
            };
        };
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_me_profile: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_users_me_profile: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description How the person is shown. */
                    display_name?: string | null;
                    /** @description The family name. */
                    family_name?: string | null;
                    /** @description The given name. */
                    given_name?: string | null;
                    /** @description A BCP 47 language tag. */
                    language_code?: string | null;
                    /** @description An IANA time zone name. */
                    time_zone?: string | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_users_scopes_scope_system_groups: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                scope: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its code. */
                    code: string;
                    /** @description Its name. */
                    name: string;
                    /** @description Its rule; a group by hand without one. */
                    rule?: {
                        /** @description The value it must equal. */
                        equals: string;
                        /** @description The field compared. */
                        field: "given_name" | "family_name" | "language_code" | "time_zone";
                    } | null;
                };
            };
        };
        responses: {
            /** @description Created */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Identified3"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_users_scopes_scope_system_groups_by_reference_system_external_id: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                external_id: string;
                scope: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description Its code. */
                    code: string;
                    /** @description Its name. */
                    name: string;
                    /** @description Its rule; a group by hand without one. */
                    rule?: {
                        /** @description The value it must equal. */
                        equals: string;
                        /** @description The field compared. */
                        field: "given_name" | "family_name" | "language_code" | "time_zone";
                    } | null;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Identified3"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_system_groups_id: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SystemGroup"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    post_users_system_groups_id_apply_rule: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Applied"];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_system_groups_id_members: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Member2"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_users_system_groups_id_members_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_users_system_groups_id_members_account: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                account: string;
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    get_users_system_groups_id_references: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    put_users_system_groups_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": {
                    /** @description The identifier. */
                    external_id: string;
                };
            };
        };
        responses: {
            /** @description Success */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExternalReference"][];
                };
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
    delete_users_system_groups_id_references_system: {
        parameters: {
            query?: never;
            header: {
                /** @description Any value; a request that changes something with a session cookie must carry it. */
                "x-kampus": string;
            };
            path: {
                id: string;
                system: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Done, with nothing to answer */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description A problem (RFC 9457) */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["Problem"];
                };
            };
        };
    };
}
