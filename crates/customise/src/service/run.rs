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

//! Running rules: for an event, every enabled rule of its scope and of the scopes above with that trigger
//! is evaluated once; when its conditions hold its actions run in order, after the event committed, as
//! its owner. Every evaluation is recorded, and a failure publishes `customise.rule_failed`.

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, ErrorKind, Result};
use kampus_module::{Event, RuleActionRun, ServiceContext};
use serde_json::json;
use sqlx::Row;

use super::rule::with_defaults;
use crate::{Customise, Rule, RuleAction, RuleId, sql};

/// The longest chain of events rules react to: an event further down a chain runs no rule.
pub const MAX_RULE_DEPTH: u8 = 5;

impl Customise {
    /// Evaluates the rules an event triggers; returns how many it evaluated for the first time.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; a failing action is recorded, not returned.
    pub async fn run_rules(&self, event: &Event, services: &ServiceContext) -> Result<usize> {
        let Some(scope) = event.scope else { return Ok(0) };
        if event.depth > MAX_RULE_DEPTH {
            return Ok(0);
        }
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let depths = match self.depths(&mut conn, ScopeId::from_uuid(scope)).await {
            Ok(depths) => depths,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };
        let ids: Vec<uuid::Uuid> = depths.keys().copied().collect();
        let rules: Vec<RuleId> = sqlx::query_scalar(sql(format!(
            "select id from {} where enabled and trigger = $1 and scope_id = any($2) order by created_at, id",
            self.table("rule")
        )))
        .bind(&event.code)
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?
        .into_iter()
        .map(RuleId::from_uuid)
        .collect();
        drop(conn);
        let mut evaluated = 0;
        for id in rules {
            let rule = self.rule(id).await?;
            if self.run_rule(&rule, event, services).await? {
                evaluated += 1;
            }
        }
        Ok(evaluated)
    }

    /// Evaluates one rule for an event, going on from the actions a run before already did; returns
    /// whether it was evaluated for the first time.
    async fn run_rule(&self, rule: &Rule, event: &Event, services: &ServiceContext) -> Result<bool> {
        let runs = self.table("rule_run");
        let fresh = sqlx::query(sql(format!(
            "insert into {runs} (id, rule_id, event, outcome) values ($1, $2, $3, 'running')
             on conflict (rule_id, event) do nothing"
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(rule.id.as_uuid())
        .bind(event.id)
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .rows_affected()
            > 0;
        let row = sqlx::query(sql(format!(
            "select outcome, actions_done from {runs} where rule_id = $1 and event = $2"
        )))
        .bind(rule.id.as_uuid())
        .bind(event.id)
        .fetch_one(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        if row.get::<String, _>("outcome") != "running" {
            return Ok(fresh);
        }
        if rule.conditions.as_ref().is_some_and(|c| !c.holds(&event.payload)) {
            self.finish_run(rule, event, "skipped", None).await?;
            return Ok(fresh);
        }
        let done = usize::try_from(row.get::<i32, _>("actions_done")).unwrap_or(0);
        for (at, action) in rule.actions.iter().enumerate().skip(done) {
            if let Err(error) = self.run_action(rule, action, event, services).await {
                self.finish_run(rule, event, "failed", Some(error.code())).await?;
                return Ok(fresh);
            }
            sqlx::query(sql(format!(
                "update {runs} set actions_done = $3 where rule_id = $1 and event = $2"
            )))
            .bind(rule.id.as_uuid())
            .bind(event.id)
            .bind(i32::try_from(at + 1).unwrap_or(i32::MAX))
            .execute(self.db.pool())
            .await
            .map_err(kampus_db::classify)?;
        }
        self.finish_run(rule, event, "succeeded", None).await?;
        Ok(fresh)
    }

    /// Runs one action of a rule as its owner, who must hold the action's permission in the event's scope.
    async fn run_action(
        &self,
        rule: &Rule,
        action: &RuleAction,
        event: &Event,
        services: &ServiceContext,
    ) -> Result<()> {
        let declared = self
            .rule_action(&action.action)
            .ok_or_else(|| Error::not_found("customise.rule_action_unknown"))?;
        let scope = event
            .scope
            .map(ScopeId::from_uuid)
            .ok_or_else(|| Error::internal("customise.rule_without_scope"))?;
        let owner = Actor::Principal(PrincipalId::from_uuid(rule.owner));
        self.access.require(&owner, declared.permission, scope).await?;
        let params = with_defaults(action, declared);
        (declared.run)(RuleActionRun {
            event,
            params: &params,
            owner: rule.owner,
            services,
        })
        .await
    }

    /// Records how an evaluation ended, and publishes a failure.
    async fn finish_run(&self, rule: &Rule, event: &Event, outcome: &str, error: Option<&str>) -> Result<()> {
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "update {} set outcome = $3, error = $4, at = now() where rule_id = $1 and event = $2",
            self.table("rule_run")
        )))
        .bind(rule.id.as_uuid())
        .bind(event.id)
        .bind(outcome)
        .bind(error)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(error) = error {
            let mut failed = kampus_events::Publish::new(
                crate::events::RULE_FAILED,
                Some(rule.id.as_uuid()),
                json!({ "rule": rule.id, "event": event.id, "error": error }),
            );
            failed.scope = Some(rule.scope.as_uuid());
            failed.actor = Some(rule.owner);
            failed.summary = Some(format!("rule {} failed: {error}", rule.name));
            failed.cause = Some(event);
            kampus_events::Events::new(self.db.clone())
                .publish_in(&mut tx, failed)
                .await?;
        }
        tx.commit().await.map_err(kampus_db::classify)
    }
}
