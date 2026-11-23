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
//! feature, custom fields on courses, the page of a course with its widgets, the catalogue for the home,
//! and the actions rules run.

use kampus_access::{Actor, PrincipalId};
use kampus_common::{Error, FieldError};
use kampus_module::{
    Cells, FieldKindDeclaration, FlagDeclaration, Grid, JobFuture, PageKindDeclaration, Resize,
    RuleActionDeclaration, RuleActionRun, RuleParameter, TermDeclaration, TermForms, ValueKind,
    WidgetDeclaration, WidgetSetting, WidgetSize,
};
use serde_json::Value;

use crate::permissions::GROUPS_MANAGE;
use crate::{Courses, GroupId};

/// The kind of entity of the custom fields of a course.
pub const COURSE_KIND: &str = "courses.course";

/// The page kind of a course.
pub const COURSE_PAGE: &str = "courses.course";

/// The home of the site, a page kind customisation declares.
const DASHBOARD_PAGE: &str = "customise.dashboard";

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
    grid: Grid::STANDARD,
    personal: false,
}];

/// The widgets of this module.
pub const WIDGETS: &[WidgetDeclaration] = &[
    WidgetDeclaration {
        code: "courses.outline",
        summary: "The sections of the course the reader sees",
        sizes: &[WidgetSize::fixed(COURSE_PAGE, Cells::new(8, 10))],
        settings: &[WidgetSetting {
            key: "collapsed",
            kind: ValueKind::Boolean { default: false },
        }],
    },
    WidgetDeclaration {
        code: "courses.summary",
        summary: "The summary and dates of the course",
        sizes: &[WidgetSize {
            page_kind: COURSE_PAGE,
            min: Cells::new(4, 2),
            default: Cells::new(12, 3),
            max: Cells::new(12, 5),
            resize: Resize::Both,
        }],
        settings: &[],
    },
    WidgetDeclaration {
        code: "courses.catalogue",
        summary: "The courses of the campus, to search, filter and open",
        sizes: &[WidgetSize {
            page_kind: DASHBOARD_PAGE,
            min: Cells::new(6, 5),
            default: Cells::new(8, 8),
            max: Cells::new(12, 16),
            resize: Resize::Both,
        }],
        settings: &[
            WidgetSetting {
                key: "heading",
                kind: ValueKind::Text {
                    max: 100,
                    default: "",
                },
            },
            WidgetSetting {
                key: "view",
                kind: ValueKind::Choice {
                    options: &["grid", "list"],
                    default: "grid",
                },
            },
            WidgetSetting {
                key: "size",
                kind: ValueKind::Integer {
                    min: 4,
                    max: 48,
                    default: 8,
                },
            },
            WidgetSetting {
                key: "sizes",
                kind: ValueKind::Text { max: 60, default: "" },
            },
        ],
    },
];

/// The actions of this module rules run.
pub const RULE_ACTIONS: &[RuleActionDeclaration] = &[RuleActionDeclaration {
    code: "courses.add_to_group",
    summary: "Adds the account of the event to a group; the membership belongs to rules",
    params: &[RuleParameter {
        key: "group",
        kind: ValueKind::Text { max: 36, default: "" },
        required: true,
    }],
    needs: &["account"],
    permission: GROUPS_MANAGE,
    publishes: &[],
    run: add_to_group,
}];

/// Adds the account of the event to the group of the rule, as the owner of the rule.
fn add_to_group(run: RuleActionRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        let invalid = || {
            Error::validation(
                "courses.rule_params_invalid",
                vec![FieldError::new("group", "uuid")],
            )
        };
        let group = run
            .params
            .get("group")
            .and_then(Value::as_str)
            .and_then(|g| uuid::Uuid::parse_str(g).ok())
            .ok_or_else(invalid)?;
        let account = run
            .event
            .payload
            .get("account")
            .and_then(Value::as_str)
            .and_then(|a| uuid::Uuid::parse_str(a).ok())
            .ok_or_else(|| {
                Error::validation(
                    "courses.rule_event_invalid",
                    vec![FieldError::new("account", "uuid")],
                )
            })?;
        let courses = run.services.service::<Courses>()?;
        courses
            .add_member_for(
                &Actor::Principal(PrincipalId::from_uuid(run.owner)),
                "rules",
                GroupId::from_uuid(group),
                PrincipalId::from_uuid(account),
            )
            .await
    })
}
