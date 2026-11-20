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
// @package   kampus-progress
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The state of a person on a course module: what views, submissions, marks and grades do to it, what a
//! teacher decides, what changing the rules does, and what each person sees.

use kampus_common::ErrorKind;
use kampus_progress::{RuleKind, State};
use kampus_records::{GradeWrite, RecordFields};
use rust_decimal::Decimal;

use crate::world::{who, world};

#[tokio::test]
async fn views_and_submissions_move_a_person_through_the_states() {
    let w = world().await;
    let page = w.page("Report").await;
    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::View, RuleKind::Submit])
        .await
        .unwrap();
    let lea = who(&w.lea);
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::NotStarted
    );

    assert_eq!(
        w.progress.view(&w.lea, page.id).await.unwrap().state,
        State::InProgress
    );
    assert_eq!(
        w.progress.view(&w.lea, page.id).await.unwrap().version,
        1,
        "the same view changes nothing"
    );
    let done = w.progress.notify_submission(page.id, lea).await.unwrap();
    assert_eq!(done.state, State::Done);
    assert_eq!(done.version, 2);
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::Done
    );
    assert_eq!(
        w.progress.state_of(&w.admin, page.id, lea).await.unwrap().state,
        State::Done
    );

    let states = w.progress.states_for(&w.lea, w.course.id, lea).await.unwrap();
    assert_eq!(states.len(), 1);
    assert_eq!(
        w.progress
            .states_for(&w.admin, w.course.id, who(&w.max))
            .await
            .unwrap()
            .len(),
        0
    );

    let other = w
        .progress
        .state_of(&w.lea, page.id, who(&w.max))
        .await
        .unwrap_err();
    assert_eq!(
        other.kind(),
        ErrorKind::PermissionDenied,
        "a learner does not see the progress of others"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_person_marks_a_course_module_only_when_it_asks_for_a_mark() {
    let w = world().await;
    let page = w.page("Reading").await;
    let lea = who(&w.lea);
    let without_rule = w.progress.mark(&w.lea, page.id, true).await.unwrap_err();
    assert_eq!(without_rule.code(), "progress.no_manual_rule");

    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::Manual])
        .await
        .unwrap();
    assert_eq!(
        w.progress.mark(&w.lea, page.id, true).await.unwrap().state,
        State::Done
    );
    assert_eq!(
        w.progress.mark(&w.lea, page.id, false).await.unwrap().state,
        State::NotStarted
    );
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::NotStarted
    );

    let denied = w
        .progress
        .define_rules(&w.lea, page.id, vec![RuleKind::View])
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn a_grade_decides_passed_or_failed_and_how_it_ended_follows_the_visibility_of_the_grade() {
    let w = world().await;
    let page = w.page("Exam").await;
    let mut fields = RecordFields::new("Exam", Decimal::TEN);
    fields.pass_grade = Some(Decimal::from(5));
    w.records
        .declare_activity_record(page.id, 0, fields)
        .await
        .unwrap();
    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::Pass])
        .await
        .unwrap();
    let lea = who(&w.lea);
    let record = w
        .records
        .tree(&w.admin, w.course.id)
        .await
        .unwrap()
        .records
        .into_iter()
        .find(|r| r.course_module == Some(page.id))
        .unwrap();

    let grade = |raw: i64| GradeWrite {
        account: lea,
        raw: Some(Decimal::from(raw)),
        text: None,
        feedback: None,
    };
    w.records
        .submit_activity_grade(record.id, grade(3))
        .await
        .unwrap();
    assert_eq!(
        w.progress.reevaluate(page.id, lea, "grade").await.unwrap().state,
        State::DoneFailed
    );
    w.records
        .submit_activity_grade(record.id, grade(8))
        .await
        .unwrap();
    assert_eq!(
        w.progress.reevaluate(page.id, lea, "grade").await.unwrap().state,
        State::DonePassed
    );
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::DonePassed
    );

    let later = kampus_common::Timestamp::parse_rfc3339("2099-01-01T00:00:00Z").unwrap();
    w.records
        .set_grade_hidden(&w.admin, record.id, lea, Some(later))
        .await
        .unwrap();
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::Done,
        "the learner is told it is done, not how it ended, while the grade is hidden"
    );
    assert_eq!(
        w.progress.state_of(&w.admin, page.id, lea).await.unwrap().state,
        State::DonePassed,
        "graders see the outcome"
    );
    w.drop().await;
}

#[tokio::test]
async fn a_teacher_decision_wins_over_the_rules_until_it_is_removed() {
    let w = world().await;
    let page = w.page("Lab").await;
    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::Submit])
        .await
        .unwrap();
    let lea = who(&w.lea);
    w.progress.view(&w.lea, page.id).await.unwrap();

    let decided = w
        .progress
        .set_exception(&w.admin, page.id, lea, State::Done, "Handed in on paper")
        .await
        .unwrap();
    assert_eq!(decided.state, State::Done);
    let listed = w.progress.exceptions(&w.admin, page.id).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].reason, "Handed in on paper");

    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::Submit, RuleKind::View])
        .await
        .unwrap();
    w.progress.recalculate().await.unwrap();
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::Done,
        "the rules do not overwrite it"
    );

    let back = w.progress.clear_exception(&w.admin, page.id, lea).await.unwrap();
    assert_eq!(back.state, State::InProgress, "what the rules say counts again");
    assert_eq!(
        w.progress
            .clear_exception(&w.admin, page.id, lea)
            .await
            .unwrap_err()
            .code(),
        "progress.exception_not_found"
    );

    let empty = w
        .progress
        .set_exception(&w.admin, page.id, lea, State::Done, "  ")
        .await
        .unwrap_err();
    assert_eq!(empty.kind(), ErrorKind::Validation);
    let denied = w
        .progress
        .set_exception(&w.lea, page.id, lea, State::Done, "mine")
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    w.drop().await;
}

#[tokio::test]
async fn changing_the_rules_re_evaluates_the_people_who_did_something() {
    let w = world().await;
    let page = w.page("Video").await;
    let lea = who(&w.lea);
    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::Submit])
        .await
        .unwrap();
    w.progress.recalculate().await.unwrap();
    assert_eq!(
        w.progress.view(&w.lea, page.id).await.unwrap().state,
        State::InProgress
    );

    w.progress
        .define_rules(&w.admin, page.id, vec![RuleKind::View])
        .await
        .unwrap();
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::InProgress,
        "not until the job runs"
    );
    assert_eq!(w.progress.recalculate().await.unwrap(), 1);
    assert_eq!(
        w.progress.state_of(&w.lea, page.id, lea).await.unwrap().state,
        State::Done
    );
    assert_eq!(w.progress.recalculate().await.unwrap(), 0);
    w.drop().await;
}
