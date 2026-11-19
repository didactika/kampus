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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What courses offers to customisation: the words of its domain in English and Spanish, the groups
//! feature, custom fields on courses, and the page of a course with its widgets.

use kampus_module::{
    FieldKindDeclaration, FlagDeclaration, PageKindDeclaration, TermDeclaration, TermForms, ValueKind,
    WidgetDeclaration, WidgetSetting,
};

/// The kind of entity of the custom fields of a course.
pub const COURSE_KIND: &str = "courses.course";

/// The page kind of a course.
pub const COURSE_PAGE: &str = "courses.course";

/// The feature of groups and group sets.
pub const GROUPS_FEATURE: &str = "courses.groups";

/// The words of this module a campus may rename.
pub const TERMS: &[TermDeclaration] = &[
    TermDeclaration {
        code: "courses.category",
        forms: &[
            TermForms {
                language: "en",
                singular: "category",
                plural: "categories",
            },
            TermForms {
                language: "es",
                singular: "categoría",
                plural: "categorías",
            },
        ],
    },
    TermDeclaration {
        code: "courses.course",
        forms: &[
            TermForms {
                language: "en",
                singular: "course",
                plural: "courses",
            },
            TermForms {
                language: "es",
                singular: "curso",
                plural: "cursos",
            },
        ],
    },
    TermDeclaration {
        code: "courses.section",
        forms: &[
            TermForms {
                language: "en",
                singular: "section",
                plural: "sections",
            },
            TermForms {
                language: "es",
                singular: "sección",
                plural: "secciones",
            },
        ],
    },
];

/// The features of this module a scope may turn off.
pub const FLAGS: &[FlagDeclaration] = &[FlagDeclaration {
    code: GROUPS_FEATURE,
    summary: "Groups and group sets in courses",
    default: true,
}];

/// The kinds of entity of this module that accept custom fields.
pub const FIELD_KINDS: &[FieldKindDeclaration] = &[FieldKindDeclaration {
    code: COURSE_KIND,
    summary: "Courses",
}];

/// The page of a course, with its regions.
pub const PAGE_KINDS: &[PageKindDeclaration] = &[PageKindDeclaration {
    code: COURSE_PAGE,
    summary: "The page of a course",
    regions: &["top", "main", "side"],
}];

/// The widgets of this module.
pub const WIDGETS: &[WidgetDeclaration] = &[
    WidgetDeclaration {
        code: "courses.outline",
        summary: "The sections of the course the reader sees",
        page_kinds: &[COURSE_PAGE],
        settings: &[WidgetSetting {
            key: "collapsed",
            kind: ValueKind::Boolean { default: false },
        }],
    },
    WidgetDeclaration {
        code: "courses.summary",
        summary: "The summary and dates of the course",
        page_kinds: &[COURSE_PAGE],
        settings: &[],
    },
];
