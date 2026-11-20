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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Exceptions of course modules: who may grant them, which types accept them, and what applies to a person
//! from their own exception, their groups' and the course module itself.

use std::collections::BTreeMap;

use kampus_access::{Actor, PrincipalId};
use kampus_activities::{ModuleFields, Target, Values};
use kampus_common::{ErrorKind, Timestamp};
use kampus_courses::{GroupFields, GroupVisibility, Place, Visibility};
use kampus_module::{ActivityKind, ActivitySetting, ActivityTypeDeclaration, Module, ValueKind};
use serde_json::json;

use crate::rules::{World, world};

/// The type of the activity these tests grant exceptions on.
const QUIZ: &str = "quiz.quiz";

/// A module that provides one activity type with exceptions.
pub(crate) static QUIZ_MODULE: Module = Module {
    name: "quiz",
    steps: &[],
    permissions: &[],
    templates: &[],
    terms: &[],
    file_areas: &[],
    events: &[],
    jobs: &[],
    schedules: &[],
    routes: None,
    plugin_types: &[],
    activity_types: &[ActivityTypeDeclaration {
        code: QUIZ,
        kind: ActivityKind::Activity,
        summary: "A timed quiz",
        features: &["exceptions"],
        settings: &[
            ActivitySetting {
                key: "closes_at",
                kind: ValueKind::Text { max: 40, default: "" },
            },
            ActivitySetting {
                key: "time_limit_seconds",
                kind: ValueKind::Integer {
                    min: 0,
                    max: 86_400,
                    default: 0,
                },
            },
            ActivitySetting {
                key: "attempts",
                kind: ValueKind::Integer {
                    min: 0,
                    max: 100,
                    default: 0,
                },
            },
        ],
    }],
    ..kampus_activities::MODULE
};

/// The fields of a quiz closing at a moment with a time limit and attempts.
fn quiz(closes_at: &str, limit: i64, attempts: i64) -> ModuleFields {
    ModuleFields {
        title: "Quiz".to_owned(),
        on_course_page: true,
        visibility: Visibility::Shown,
        settings: BTreeMap::from([
            ("closes_at".to_owned(), json!(closes_at)),
            ("time_limit_seconds".to_owned(), json!(limit)),
            ("attempts".to_owned(), json!(attempts)),
        ]),
    }
}

/// A group of the course.
async fn group(w: &World, name: &str) -> kampus_courses::Group {
    w.courses
        .create_group(
            &w.admin,
            w.course.id,
            GroupFields {
                name: name.to_owned(),
                code: None,
                description: None,
                section: None,
                visibility: GroupVisibility::Everyone,
            },
        )
        .await
        .unwrap()
}

/// The principal behind an actor.
fn person(actor: &Actor) -> PrincipalId {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => *p,
        Actor::Visitor(_) => panic!("a person"),
    }
}

/// A moment from its text.
fn at(text: &str) -> Timestamp {
    Timestamp::parse_rfc3339(text).unwrap()
}

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one scenario told from start to end")]
async fn exceptions_resolve_person_then_groups_then_the_course_module() {
    let w = world().await;
    let learner = person(&w.learner);
    let module = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            QUIZ,
            quiz("2030-01-10T10:00:00Z", 1800, 1),
            Place::Last,
        )
        .await
        .unwrap();
    assert_eq!(
        w.activities.effective_values(module.id, learner).await.unwrap(),
        Values {
            closes_at: Some(at("2030-01-10T10:00:00Z")),
            time_limit_seconds: Some(1800),
            attempts: Some(1),
            ..Values::default()
        }
    );

    let slow = group(&w, "Slow").await;
    let late = group(&w, "Late").await;
    w.courses.add_member(&w.admin, slow.id, learner).await.unwrap();
    w.courses.add_member(&w.admin, late.id, learner).await.unwrap();
    w.activities
        .set_exception(
            &w.admin,
            module.id,
            Target::Group(slow.id),
            Values {
                time_limit_seconds: Some(3600),
                attempts: Some(2),
                ..Values::default()
            },
            Some("extra time"),
        )
        .await
        .unwrap();
    w.activities
        .set_exception(
            &w.admin,
            module.id,
            Target::Group(late.id),
            Values {
                closes_at: Some(at("2030-01-12T10:00:00Z")),
                time_limit_seconds: Some(2400),
                ..Values::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        w.activities.effective_values(module.id, learner).await.unwrap(),
        Values {
            closes_at: Some(at("2030-01-12T10:00:00Z")),
            time_limit_seconds: Some(3600),
            attempts: Some(2),
            ..Values::default()
        },
        "the most generous value of each setting among the groups"
    );

    w.activities
        .set_exception(
            &w.admin,
            module.id,
            Target::Person(learner),
            Values {
                time_limit_seconds: Some(900),
                ..Values::default()
            },
            None,
        )
        .await
        .unwrap();
    let own = w.activities.effective_values(module.id, learner).await.unwrap();
    assert_eq!(
        own.time_limit_seconds,
        Some(900),
        "their own wins, even when smaller"
    );
    assert_eq!(
        own.attempts,
        Some(2),
        "what they did not change comes from the groups"
    );
    assert_eq!(own.closes_at, Some(at("2030-01-12T10:00:00Z")));

    let listed = w.activities.exceptions(&w.admin, module.id).await.unwrap();
    assert_eq!(listed.len(), 3);
    let own_exception = listed
        .iter()
        .find(|e| e.target == Target::Person(learner))
        .unwrap();
    w.activities
        .remove_exception(&w.admin, own_exception.id)
        .await
        .unwrap();
    assert_eq!(
        w.activities
            .effective_values(module.id, learner)
            .await
            .unwrap()
            .time_limit_seconds,
        Some(3600)
    );
    w.drop().await;
}

#[tokio::test]
async fn exceptions_are_replaced_validated_and_limited_to_types_that_support_them() {
    let w = world().await;
    let learner = person(&w.learner);
    let module = w
        .activities
        .create_module(
            &w.admin,
            w.first.id,
            QUIZ,
            quiz("2030-01-10T10:00:00Z", 1800, 1),
            Place::Last,
        )
        .await
        .unwrap();
    let attempts = |n| Values {
        attempts: Some(n),
        ..Values::default()
    };
    w.activities
        .set_exception(&w.admin, module.id, Target::Person(learner), attempts(2), None)
        .await
        .unwrap();
    w.activities
        .set_exception(&w.admin, module.id, Target::Person(learner), attempts(5), None)
        .await
        .unwrap();
    let listed = w.activities.exceptions(&w.admin, module.id).await.unwrap();
    assert_eq!(listed.len(), 1, "one per person");
    assert_eq!(listed[0].values.attempts, Some(5));

    let empty = w
        .activities
        .set_exception(
            &w.admin,
            module.id,
            Target::Person(learner),
            Values::default(),
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(empty.kind(), ErrorKind::Validation);
    let backwards = w
        .activities
        .set_exception(
            &w.admin,
            module.id,
            Target::Person(learner),
            Values {
                opens_at: Some(at("2030-02-01T00:00:00Z")),
                closes_at: Some(at("2030-01-01T00:00:00Z")),
                ..Values::default()
            },
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(backwards.kind(), ErrorKind::Validation);

    let page = w.add_page(&w.first, "A page").await;
    let unsupported = w
        .activities
        .set_exception(&w.admin, page.id, Target::Person(learner), attempts(2), None)
        .await
        .unwrap_err();
    assert_eq!(unsupported.code(), "activities.exceptions_unsupported");

    let denied = w
        .activities
        .set_exception(&w.learner, module.id, Target::Person(learner), attempts(9), None)
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    assert_eq!(
        w.activities
            .exceptions(&w.learner, module.id)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    assert_eq!(
        w.activities
            .effective_values_for(&w.learner, module.id, learner)
            .await
            .unwrap()
            .attempts,
        Some(5),
        "a person reads what applies to them"
    );
    assert_eq!(
        w.activities
            .effective_values_for(&w.learner, module.id, person(&w.admin))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PermissionDenied
    );
    w.drop().await;
}
