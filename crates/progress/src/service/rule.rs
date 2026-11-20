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

//! What a course module requires and what people do about it: the rules, a view, a submission and the mark a
//! person puts on their own. Every one of them re-evaluates the state of the person.

use kampus_access::{Actor, PrincipalId};
use kampus_activities::ModuleId;
use kampus_common::{Error, FieldError, Result};

use super::principal;
use crate::permissions::{MANAGE, MARK_OWN};
use crate::{ActivityState, Progress, RuleKind, events, sql};

/// A kind of signal a person leaves on a course module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Signal {
    /// They viewed it.
    View,
    /// They submitted something.
    Submit,
    /// They marked it done.
    Manual,
}

impl Signal {
    /// The code stored for it.
    const fn code(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Submit => "submit",
            Self::Manual => "manual",
        }
    }
}

impl Progress {
    /// The rules of a course module the actor sees.
    ///
    /// # Errors
    ///
    /// `not_found` (`progress.module_not_found`) for a course module the actor does not see.
    pub async fn rules(&self, actor: &Actor, module: ModuleId) -> Result<Vec<RuleKind>> {
        self.require_module(actor, module).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        self.rules_in(&mut conn, module).await
    }

    /// Replaces the rules of a course module; needs `progress.manage` in it. People who already did
    /// something are re-evaluated by a job, and what teachers decided stays.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn define_rules(
        &self,
        actor: &Actor,
        module: ModuleId,
        kinds: Vec<RuleKind>,
    ) -> Result<Vec<RuleKind>> {
        let scope = self.activities.scope_of(module).await?;
        self.access.require(actor, MANAGE, scope).await?;
        let mut wanted = kinds;
        wanted.sort();
        wanted.dedup();
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where course_module_id = $1",
            self.table("rule")
        )))
        .bind(module.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for kind in &wanted {
            sqlx::query(sql(format!(
                "insert into {} (id, course_module_id, kind) values ($1, $2, $3)",
                self.table("rule")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(module.as_uuid())
            .bind(kind.code())
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
        }
        self.publish_in(&mut tx, events::RULES_CHANGED, module, None, principal(actor))
            .await?;
        self.queue_in(&mut tx, module).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(wanted)
    }

    /// Records what a person did on a course module and evaluates their state.
    async fn record(&self, module: ModuleId, account: PrincipalId, signal: Signal) -> Result<ActivityState> {
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (course_module_id, account_id, kind) values ($1, $2, $3) on conflict do nothing",
            self.table("signal")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .bind(signal.code())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let found = self.evaluate_in(&mut tx, module, account, signal.code()).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(found)
    }

    /// A person viewed a course module. It is meant for the module that shows the activity, which has
    /// already checked that the person may see it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn notify_view(&self, module: ModuleId, account: PrincipalId) -> Result<ActivityState> {
        self.record(module, account, Signal::View).await
    }

    /// A person submitted something to a course module. It is meant for the module that provides the
    /// activity.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn notify_submission(&self, module: ModuleId, account: PrincipalId) -> Result<ActivityState> {
        self.record(module, account, Signal::Submit).await
    }

    /// The actor opens a course module they see.
    ///
    /// # Errors
    ///
    /// `not_found` for a course module they do not see, `permission_denied` for a visitor.
    pub async fn view(&self, actor: &Actor, module: ModuleId) -> Result<ActivityState> {
        let person = principal(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        self.require_module(actor, module).await?;
        self.record(module, person, Signal::View).await
    }

    /// The actor marks a course module done, or takes the mark back; needs `progress.mark_own` in it, and
    /// the course module must ask for a manual mark.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`progress.no_manual_rule`).
    pub async fn mark(&self, actor: &Actor, module: ModuleId, done: bool) -> Result<ActivityState> {
        let person = principal(actor).ok_or_else(|| Error::permission_denied("access.denied"))?;
        let stored = self.require_module(actor, module).await?;
        self.access.require(actor, MARK_OWN, stored.scope).await?;
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        if !self
            .rules_in(&mut conn, module)
            .await?
            .contains(&RuleKind::Manual)
        {
            return Err(Error::validation(
                "progress.no_manual_rule",
                vec![FieldError::new("module", "manual")],
            ));
        }
        drop(conn);
        if done {
            return self.record(module, person, Signal::Manual).await;
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where course_module_id = $1 and account_id = $2 and kind = 'manual'",
            self.table("signal")
        )))
        .bind(module.as_uuid())
        .bind(person.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let found = self.evaluate_in(&mut tx, module, person, "unmark").await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(found)
    }

    /// Re-evaluates a person after something that is not a signal changed, such as their grade.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn reevaluate(
        &self,
        module: ModuleId,
        account: PrincipalId,
        cause: &str,
    ) -> Result<ActivityState> {
        let mut tx = self.db.begin().await?;
        let found = self.evaluate_in(&mut tx, module, account, cause).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(found)
    }
}

impl Progress {
    /// The scope of a course module, for the permission checked before a change.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module.
    pub async fn module_scope(&self, module: ModuleId) -> Result<kampus_access::ScopeId> {
        self.activities.scope_of(module).await
    }
}
