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

//! Rules: written in a scope by who holds `customise.rule.manage` there and every permission its actions
//! need, validated against the event type of its trigger and the actions modules declare; free code is
//! never accepted.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result, Timestamp};
use kampus_module::{RuleActionDeclaration, ValueKind};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::principal;
use crate::permissions::RULE_MANAGE;
use crate::{Condition, Customise, Rule, RuleAction, RuleId, RuleRun, RuleSpec, sql};

/// The longest name, in characters.
const MAX_NAME: usize = 100;

/// The most actions a rule runs.
const MAX_ACTIONS: usize = 10;

/// The most comparisons and groups in the conditions of a rule.
const MAX_CONDITIONS: usize = 50;

/// The deepest the conditions of a rule nest.
const MAX_NESTING: usize = 5;

/// The most runs a listing returns.
const MAX_RUNS: i64 = 50;

/// The columns of a rule.
const COLUMNS: &str = "id, scope_id, owner, name, trigger, conditions, actions, enabled, version";

/// A rule from its row.
fn rule(row: &PgRow) -> Rule {
    Rule {
        id: RuleId::from_uuid(row.get("id")),
        scope: ScopeId::from_uuid(row.get("scope_id")),
        owner: row.get("owner"),
        name: row.get("name"),
        trigger: row.get("trigger"),
        conditions: row
            .get::<Option<Value>, _>("conditions")
            .and_then(|c| serde_json::from_value(c).ok()),
        actions: serde_json::from_value(row.get("actions")).unwrap_or_default(),
        enabled: row.get("enabled"),
        version: row.get("version"),
    }
}

/// The problems of the conditions of a rule, given the fields of its trigger.
fn condition_problems(condition: &Condition, fields: &[String], problems: &mut Vec<FieldError>) {
    let mut count = 0;
    let mut pending = vec![(condition, 1, "conditions".to_owned())];
    while let Some((node, depth, path)) = pending.pop() {
        count += 1;
        if depth > MAX_NESTING {
            problems.push(FieldError::new(path, "depth"));
            continue;
        }
        match node {
            Condition::All(list) | Condition::Any(list) => {
                if list.is_empty() {
                    problems.push(FieldError::new(path.clone(), "required"));
                }
                for (at, child) in list.iter().enumerate() {
                    pending.push((child, depth + 1, format!("{path}[{at}]")));
                }
            }
            Condition::Not(child) => pending.push((child, depth + 1, format!("{path}.not"))),
            Condition::Compare(c) => {
                if !fields.contains(&c.field) {
                    problems.push(FieldError::new(format!("{path}.field"), "one_of"));
                }
                if c.op == crate::Operator::In && !c.value.is_array() {
                    problems.push(FieldError::new(format!("{path}.value"), "type"));
                }
            }
        }
    }
    if count > MAX_CONDITIONS {
        problems.push(FieldError::new("conditions", "length"));
    }
}

/// The problems of an action of a rule, given its declaration and the trigger with its fields.
fn action_problems(
    at: usize,
    action: &RuleAction,
    declared: &RuleActionDeclaration,
    (trigger, fields): (&str, &[String]),
    problems: &mut Vec<FieldError>,
) {
    let name = |part: &str| format!("actions[{at}].{part}");
    if declared.publishes.contains(&trigger) {
        problems.push(FieldError::new(name("action"), "triggers_itself"));
    }
    if declared.needs.iter().any(|n| !fields.iter().any(|f| f == n)) {
        problems.push(FieldError::new(name("action"), "trigger_fields"));
    }
    for (key, value) in &action.params {
        let fits = declared
            .params
            .iter()
            .find(|p| p.key == key)
            .is_some_and(|p| p.kind.accepts(value));
        if !fits {
            problems.push(FieldError::new(name(&format!("params.{key}")), "type"));
        }
    }
    for param in declared.params.iter().filter(|p| p.required) {
        if !action.params.contains_key(param.key) {
            problems.push(FieldError::new(
                name(&format!("params.{}", param.key)),
                "required",
            ));
        }
    }
}

/// The parameters of an action with the defaults of the optional ones it leaves out.
pub(crate) fn with_defaults(action: &RuleAction, declared: &RuleActionDeclaration) -> Value {
    let mut params = action.params.clone();
    for param in declared.params {
        params
            .entry(param.key.to_owned())
            .or_insert_with(|| ValueKind::default_value(param.kind));
    }
    json!(params)
}

impl Customise {
    /// The rules a scope writes, by name.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn rules(&self, scope: ScopeId) -> Result<Vec<Rule>> {
        let rows = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where scope_id = $1 order by name, id",
            self.table("rule")
        )))
        .bind(scope.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows.iter().map(rule).collect())
    }

    /// A rule.
    ///
    /// # Errors
    ///
    /// `not_found` (`customise.rule_not_found`) for an unknown rule.
    pub async fn rule(&self, id: RuleId) -> Result<Rule> {
        let row = sqlx::query(sql(format!(
            "select {COLUMNS} from {} where id = $1",
            self.table("rule")
        )))
        .bind(id.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("customise.rule_not_found"))?;
        Ok(rule(&row))
    }

    /// Writes a rule in a scope; its author becomes its owner, whose permissions its actions run with.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `customise.rule.manage` or a permission an action needs, `validation`
    /// (`customise.rule_invalid`) for an unknown trigger or action, conditions on fields the trigger does
    /// not have, parameters of another kind, or an action that publishes its own trigger.
    pub async fn create_rule(&self, actor: &Actor, scope: ScopeId, spec: RuleSpec) -> Result<Rule> {
        let owner = self.checked_rule(actor, scope, &spec).await?;
        let mut tx = self.db.begin().await?;
        let id = RuleId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, scope_id, owner, name, trigger, conditions, actions, enabled)
             values ($1, $2, $3, $4, $5, $6, $7, $8)",
            self.table("rule")
        )))
        .bind(id.as_uuid())
        .bind(scope.as_uuid())
        .bind(owner)
        .bind(spec.name.trim())
        .bind(&spec.trigger)
        .bind(spec.conditions.as_ref().map(|c| json!(c)))
        .bind(json!(spec.actions))
        .bind(spec.enabled)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let after = json!({ "name": spec.name.trim(), "trigger": spec.trigger, "enabled": spec.enabled });
        self.record_in(
            &mut tx,
            "rule",
            &id.to_string(),
            scope,
            Some(owner),
            (None, Some(after)),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.rule(id).await
    }

    /// Replaces a rule, on the version it was read at; its author becomes its owner.
    ///
    /// # Errors
    ///
    /// Those of [`Customise::create_rule`], `not_found` for an unknown rule, and `conflict`
    /// (`customise.rule_changed`) when it changed since it was read.
    pub async fn update_rule(
        &self,
        actor: &Actor,
        id: RuleId,
        spec: RuleSpec,
        expected: i32,
    ) -> Result<Rule> {
        let stored = self.rule(id).await?;
        let owner = self.checked_rule(actor, stored.scope, &spec).await?;
        let mut tx = self.db.begin().await?;
        let changed = sqlx::query(sql(format!(
            "update {} set owner = $2, name = $3, trigger = $4, conditions = $5, actions = $6, enabled = $7,
             version = version + 1 where id = $1 and version = $8",
            self.table("rule")
        )))
        .bind(id.as_uuid())
        .bind(owner)
        .bind(spec.name.trim())
        .bind(&spec.trigger)
        .bind(spec.conditions.as_ref().map(|c| json!(c)))
        .bind(json!(spec.actions))
        .bind(spec.enabled)
        .bind(expected)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if changed == 0 {
            return Err(Error::conflict("customise.rule_changed"));
        }
        let before = json!({ "name": stored.name, "trigger": stored.trigger, "enabled": stored.enabled });
        let after = json!({ "name": spec.name.trim(), "trigger": spec.trigger, "enabled": spec.enabled });
        self.record_in(
            &mut tx,
            "rule",
            &id.to_string(),
            stored.scope,
            Some(owner),
            (Some(before), Some(after)),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.rule(id).await
    }

    /// Removes a rule with its runs; needs `customise.rule.manage` in its scope.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown rule, `permission_denied` without the permission.
    pub async fn delete_rule(&self, actor: &Actor, id: RuleId) -> Result<()> {
        let stored = self.rule(id).await?;
        self.access.require(actor, RULE_MANAGE, stored.scope).await?;
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!("delete from {} where id = $1", self.table("rule"))))
            .bind(id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        self.record_in(
            &mut tx,
            "rule",
            &id.to_string(),
            stored.scope,
            principal(actor),
            (None, None),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }

    /// The latest evaluations of a rule, newest first.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn rule_runs(&self, id: RuleId) -> Result<Vec<RuleRun>> {
        let rows = sqlx::query(sql(format!(
            "select event, outcome, actions_done, error, at from {} where rule_id = $1 order by at desc, id desc
             limit $2",
            self.table("rule_run")
        )))
        .bind(id.as_uuid())
        .bind(MAX_RUNS)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| RuleRun {
                event: r.get("event"),
                outcome: r.get("outcome"),
                actions_done: r.get("actions_done"),
                error: r.get("error"),
                at: Timestamp::from_offset_date_time(r.get("at")),
            })
            .collect())
    }

    /// Checks a rule an actor writes in a scope; returns its owner.
    async fn checked_rule(&self, actor: &Actor, scope: ScopeId, spec: &RuleSpec) -> Result<uuid::Uuid> {
        self.access.require(actor, RULE_MANAGE, scope).await?;
        let owner = principal(actor).ok_or_else(|| Error::permission_denied("customise.rule_owner"))?;
        let mut problems = Vec::new();
        let name = spec.name.trim();
        if name.is_empty() || name.chars().count() > MAX_NAME {
            problems.push(FieldError::new("name", "length"));
        }
        let events = kampus_events::Events::new(self.db.clone());
        let fields = events.declared_fields(&spec.trigger).await?;
        if fields.is_none() {
            problems.push(FieldError::new("trigger", "one_of"));
        }
        let fields = fields.unwrap_or_default();
        if let Some(conditions) = &spec.conditions {
            condition_problems(conditions, &fields, &mut problems);
        }
        if spec.actions.is_empty() || spec.actions.len() > MAX_ACTIONS {
            problems.push(FieldError::new("actions", "length"));
        }
        let mut permissions = Vec::new();
        for (at, action) in spec.actions.iter().enumerate() {
            match self.rule_action(&action.action) {
                Some(declared) => {
                    action_problems(at, action, declared, (&spec.trigger, &fields), &mut problems);
                    permissions.push(declared.permission);
                }
                None => problems.push(FieldError::new(format!("actions[{at}].action"), "one_of")),
            }
        }
        if !problems.is_empty() {
            return Err(Error::validation("customise.rule_invalid", problems));
        }
        for permission in permissions {
            self.access.require(actor, permission, scope).await?;
        }
        Ok(owner)
    }
}
