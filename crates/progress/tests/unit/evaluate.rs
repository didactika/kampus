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

//! Worked examples of the state of a person: what each rule asks, how a pass rule ends, and what counts
//! as having started.

use kampus_progress::evaluate::{Facts, evaluate};
use kampus_progress::{RuleKind, State};
use rust_decimal::Decimal;

/// What a person did.
fn facts() -> Facts {
    Facts::default()
}

#[test]
fn nobody_has_started_until_they_do_something() {
    assert_eq!(evaluate(&[RuleKind::View], &facts()), State::NotStarted);
    assert_eq!(evaluate(&[], &facts()), State::NotStarted);
    let viewed = Facts {
        viewed: true,
        ..facts()
    };
    assert_eq!(
        evaluate(&[RuleKind::View, RuleKind::Submit], &viewed),
        State::InProgress
    );
    assert_eq!(
        evaluate(&[], &viewed),
        State::InProgress,
        "a module without rules never finishes by itself"
    );
}

#[test]
fn a_person_is_done_when_every_rule_is_met() {
    let rules = [RuleKind::View, RuleKind::Submit];
    let both = Facts {
        viewed: true,
        submitted: true,
        ..facts()
    };
    assert_eq!(evaluate(&rules, &both), State::Done);
    let marked = Facts {
        marked: true,
        ..facts()
    };
    assert_eq!(evaluate(&[RuleKind::Manual], &marked), State::Done);
    assert_eq!(evaluate(&[RuleKind::Manual], &facts()), State::NotStarted);
    let graded = Facts {
        grade: Some(Decimal::ONE),
        ..facts()
    };
    assert_eq!(evaluate(&[RuleKind::Grade], &graded), State::Done);
}

#[test]
fn a_pass_rule_ends_passed_or_failed_by_the_grade() {
    let rules = [RuleKind::Pass];
    let passing = Facts {
        grade: Some(Decimal::from(7)),
        pass_grade: Some(Decimal::from(5)),
        ..facts()
    };
    assert_eq!(evaluate(&rules, &passing), State::DonePassed);
    let exactly = Facts {
        grade: Some(Decimal::from(5)),
        pass_grade: Some(Decimal::from(5)),
        ..facts()
    };
    assert_eq!(
        evaluate(&rules, &exactly),
        State::DonePassed,
        "the passing grade itself passes"
    );
    let failing = Facts {
        grade: Some(Decimal::from(4)),
        pass_grade: Some(Decimal::from(5)),
        ..facts()
    };
    assert_eq!(evaluate(&rules, &failing), State::DoneFailed);
    let without_pass_grade = Facts {
        grade: Some(Decimal::ZERO),
        ..facts()
    };
    assert_eq!(
        evaluate(&rules, &without_pass_grade),
        State::DonePassed,
        "any grade passes without a passing grade"
    );
    assert_eq!(evaluate(&rules, &facts()), State::NotStarted);

    let several = [RuleKind::View, RuleKind::Pass];
    assert_eq!(
        evaluate(&several, &failing),
        State::DoneFailed,
        "a failing grade ends it even if other rules wait"
    );
    let viewed_and_passing = Facts {
        viewed: true,
        ..passing
    };
    assert_eq!(evaluate(&several, &viewed_and_passing), State::DonePassed);
    assert_eq!(
        evaluate(&several, &passing),
        State::InProgress,
        "passing is not enough while viewing is missing"
    );
}

#[test]
fn a_state_without_its_outcome_is_just_done() {
    assert_eq!(State::DonePassed.without_outcome(), State::Done);
    assert_eq!(State::DoneFailed.without_outcome(), State::Done);
    assert_eq!(State::InProgress.without_outcome(), State::InProgress);
}

#[test]
fn a_rule_of_the_type_counts_like_the_others() {
    let waiting = Facts {
        type_rules: Some(false),
        ..facts()
    };
    assert_eq!(
        evaluate(&[], &waiting),
        State::NotStarted,
        "only rules of the type"
    );
    let said = Facts {
        type_signal: true,
        ..waiting
    };
    assert_eq!(evaluate(&[], &said), State::InProgress);
    let met = Facts {
        type_rules: Some(true),
        type_signal: true,
        ..facts()
    };
    assert_eq!(evaluate(&[], &met), State::Done);
    assert_eq!(
        evaluate(&[RuleKind::View], &met),
        State::InProgress,
        "the other rules still count"
    );
    let viewed = Facts { viewed: true, ..met };
    assert_eq!(evaluate(&[RuleKind::View], &viewed), State::Done);
}
