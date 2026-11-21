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

//! Rules of activity types: a course module can require a rule its type declares, such as using every
//! attempt; the module of the type reports, for each person, whether they meet it, and progress evaluates the
//! state with the other rules.

use kampus_access::{Actor, PrincipalId};
use kampus_activities::ModuleId;
use kampus_common::{Error, FieldError, Result};
use kampus_module::ProgressRuleDeclaration;

use super::principal;
use crate::permissions::MANAGE;
use crate::{ActivityState, Progress, events, sql};

impl Progress {
    /// The rules the type of a course module declares.
    fn declared_for(&self, type_code: &str) -> Vec<&'static ProgressRuleDeclaration> {
        self.modules
            .iter()
            .flat_map(|m| m.progress_rules.iter())
            .filter(|r| r.activity_type == type_code)
            .collect()
    }

    /// The rules of its type a course module requires, and those its type offers.
    ///
    /// # Errors
    ///
    /// `not_found` for a course module the actor does not see.
    pub async fn type_rules(
        &self,
        actor: &Actor,
        module: ModuleId,
    ) -> Result<(Vec<String>, Vec<&'static ProgressRuleDeclaration>)> {
        let stored = self.require_module(actor, module).await?;
        let required: Vec<String> = sqlx::query_scalar(sql(format!(
            "select type_rule_code from {} where course_module_id = $1 and kind = 'type' order by type_rule_code",
            self.table("rule")
        )))
        .bind(module.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok((required, self.declared_for(&stored.type_code)))
    }

    /// Replaces the rules of its type a course module requires; needs `progress.manage` there. Everyone who
    /// did something on it is evaluated again.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`progress.type_rule_unknown`) for a rule its type does
    /// not declare.
    pub async fn define_type_rules(
        &self,
        actor: &Actor,
        module: ModuleId,
        codes: Vec<String>,
    ) -> Result<Vec<String>> {
        let stored = self.require_module(actor, module).await?;
        self.access.require(actor, MANAGE, stored.scope).await?;
        let offered = self.declared_for(&stored.type_code);
        let mut wanted = codes;
        wanted.sort();
        wanted.dedup();
        if let Some(unknown) = wanted
            .iter()
            .find(|c| !offered.iter().any(|r| r.code == c.as_str()))
        {
            return Err(Error::validation(
                "progress.type_rule_unknown",
                vec![FieldError::new("rules", "one_of")],
            )
            .with_param("rule", unknown.clone()));
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where course_module_id = $1 and kind = 'type'",
            self.table("rule")
        )))
        .bind(module.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        for code in &wanted {
            sqlx::query(sql(format!(
                "insert into {} (id, course_module_id, kind, type_rule_code) values ($1, $2, 'type', $3)",
                self.table("rule")
            )))
            .bind(uuid::Uuid::now_v7())
            .bind(module.as_uuid())
            .bind(code)
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

    /// Records what the module of a type says about a person on one of its rules and evaluates their state;
    /// for the module that provides the type, after what the rule depends on changed.
    ///
    /// # Errors
    ///
    /// `validation` (`progress.type_rule_unknown`) for a rule the type of the course module does not declare.
    pub async fn report_type_rule(
        &self,
        module: ModuleId,
        account: PrincipalId,
        code: &str,
        met: bool,
    ) -> Result<ActivityState> {
        let type_code = self.activities.type_of(module).await?;
        if !self.declared_for(&type_code).iter().any(|r| r.code == code) {
            return Err(Error::validation(
                "progress.type_rule_unknown",
                vec![FieldError::new("rule", "one_of")],
            )
            .with_param("rule", code.to_owned()));
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {} (course_module_id, account_id, code, met) values ($1, $2, $3, $4)
             on conflict (course_module_id, account_id, code) do update set met = excluded.met, at = now()",
            self.table("type_signal")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .bind(code)
        .bind(met)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let found = self.evaluate_in(&mut tx, module, account, code).await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(found)
    }
}
