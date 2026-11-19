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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! One test per behaviour of rules, against PostgreSQL: validation against the trigger and the declared
//! actions, evaluation once per event down the scope tree, conditions, failures and chain limits.

use kampus_access::ScopeId;
use kampus_common::{Clock, ErrorKind, SystemClock};
use kampus_customise::{Comparison, Condition, Operator, RuleAction, RuleSpec};
use kampus_module::{Event, ServiceContext};
use serde_json::{Value, json};

use crate::world::{NOTED, World, world};

/// An action with its parameters.
fn action(code: &str, params: Value) -> RuleAction {
    RuleAction {
        action: code.to_owned(),
        params: serde_json::from_value(params).unwrap(),
    }
}

/// A rule of `testing.happened` that notes the event with a tag when its level is high.
fn noting(tag: &str) -> RuleSpec {
    RuleSpec {
        name: format!("Note {tag}"),
        trigger: "testing.happened".to_owned(),
        conditions: Some(Condition::Compare(Comparison {
            field: "level".to_owned(),
            op: Operator::Eq,
            value: json!("high"),
        })),
        actions: vec![action("testing.note", json!({ "tag": tag }))],
        enabled: true,
    }
}

/// An event of `testing.happened` in a scope.
fn happened(scope: ScopeId, level: &str, depth: u8) -> Event {
    Event {
        id: uuid::Uuid::now_v7(),
        code: "testing.happened".to_owned(),
        version: 1,
        scope: Some(scope.as_uuid()),
        actor: None,
        object: None,
        payload: json!({ "account": uuid::Uuid::now_v7(), "level": level }),
        occurred_at: SystemClock.now(),
        depth,
    }
}

/// How many times the action noted an event.
fn noted(event: &Event) -> usize {
    NOTED
        .lock()
        .unwrap()
        .iter()
        .filter(|(id, _)| *id == event.id)
        .count()
}

/// The problems a rule written in the course is refused with.
async fn refused(w: &World, spec: RuleSpec) -> Vec<String> {
    let error = w
        .customise
        .create_rule(&w.admin, w.course, spec)
        .await
        .unwrap_err();
    assert_eq!(error.code(), "customise.rule_invalid");
    error
        .fields()
        .iter()
        .map(|f| format!("{} {}", f.field, f.rule))
        .collect()
}

#[tokio::test]
async fn a_rule_keeps_to_its_trigger_and_to_the_actions_modules_declare() {
    let w = world().await;
    let mut spec = noting("x");
    spec.trigger = "testing.unknown".to_owned();
    assert!(refused(&w, spec).await.contains(&"trigger one_of".to_owned()));
    let mut spec = noting("x");
    spec.conditions = Some(Condition::Compare(Comparison {
        field: "colour".to_owned(),
        op: Operator::Exists,
        value: Value::Null,
    }));
    assert_eq!(refused(&w, spec).await, ["conditions.field one_of"]);
    let mut spec = noting("x");
    spec.actions = vec![action("testing.note", json!({ "loud": "yes" }))];
    let problems = refused(&w, spec).await;
    assert!(
        problems.contains(&"actions[0].params.loud type".to_owned()),
        "{problems:?}"
    );
    assert!(
        problems.contains(&"actions[0].params.tag required".to_owned()),
        "{problems:?}"
    );
    let mut spec = noting("x");
    spec.actions = vec![action("testing.echo", json!({}))];
    assert_eq!(refused(&w, spec).await, ["actions[0].action triggers_itself"]);
    let mut spec = noting("x");
    spec.actions = vec![action("testing.nothing", json!({}))];
    assert_eq!(refused(&w, spec).await, ["actions[0].action one_of"]);
    let denied = w
        .customise
        .create_rule(&w.nobody, w.course, noting("x"))
        .await
        .unwrap_err();
    assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
    let rule = w
        .customise
        .create_rule(&w.admin, w.category, noting("x"))
        .await
        .unwrap();
    assert_eq!(
        w.customise.rules(w.category).await.unwrap(),
        std::slice::from_ref(&rule)
    );
    let mut changed = noting("y");
    changed.enabled = false;
    let updated = w
        .customise
        .update_rule(&w.admin, rule.id, changed.clone(), rule.version)
        .await
        .unwrap();
    assert!(!updated.enabled);
    let stale = w
        .customise
        .update_rule(&w.admin, rule.id, changed, rule.version)
        .await
        .unwrap_err();
    assert_eq!(stale.code(), "customise.rule_changed");
    w.customise.delete_rule(&w.admin, rule.id).await.unwrap();
    assert_eq!(
        w.customise.rules(w.category).await.unwrap(),
        Vec::<kampus_customise::Rule>::new()
    );
    w.drop().await;
}

#[tokio::test]
async fn a_rule_runs_once_per_event_below_its_scope_when_its_conditions_hold() {
    let w = world().await;
    let c = &w.customise;
    let services = ServiceContext::default();
    let rule = c.create_rule(&w.admin, w.category, noting("high")).await.unwrap();
    let mut off = noting("off");
    off.enabled = false;
    c.create_rule(&w.admin, w.category, off).await.unwrap();
    c.create_rule(&w.admin, w.section, noting("below")).await.unwrap();
    let event = happened(w.course, "high", 0);
    assert_eq!(c.run_rules(&event, &services).await.unwrap(), 1);
    assert_eq!(c.run_rules(&event, &services).await.unwrap(), 0);
    assert_eq!(noted(&event), 1, "once, by the rule of the category only");
    let params = NOTED
        .lock()
        .unwrap()
        .iter()
        .find(|(id, _)| *id == event.id)
        .unwrap()
        .1
        .clone();
    assert_eq!(params, json!({ "tag": "high", "loud": false }));
    let low = happened(w.course, "low", 0);
    c.run_rules(&low, &services).await.unwrap();
    assert_eq!(noted(&low), 0);
    let deep = happened(w.course, "high", 6);
    assert_eq!(c.run_rules(&deep, &services).await.unwrap(), 0);
    let outcomes: Vec<String> = c
        .rule_runs(rule.id)
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.outcome)
        .collect();
    assert_eq!(outcomes, ["skipped", "succeeded"]);
    w.drop().await;
}

#[tokio::test]
async fn a_failing_action_is_recorded_and_published_and_the_actions_before_it_stay_done() {
    let w = world().await;
    let c = &w.customise;
    let mut spec = noting("then fail");
    spec.conditions = None;
    spec.actions.push(action("testing.fail", json!({})));
    let rule = c.create_rule(&w.admin, w.campus, spec).await.unwrap();
    let event = happened(w.section, "any", 0);
    c.run_rules(&event, &ServiceContext::default()).await.unwrap();
    let run = c.rule_runs(rule.id).await.unwrap().remove(0);
    assert_eq!((run.outcome.as_str(), run.actions_done), ("failed", 1));
    assert_eq!(run.error.as_deref(), Some("testing.failed"));
    assert_eq!(noted(&event), 1);
    let published: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {}.\"event\" where type_code = 'customise.rule_failed'",
        w.db.prefix().schema("events")
    )))
    .fetch_one(w.db.pool())
    .await
    .unwrap();
    assert_eq!(published, 1);
    w.drop().await;
}
