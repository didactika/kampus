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

//! Exceptions: a teacher decides the state of a person on a course module, with a reason; the decision wins
//! over the rules until it is removed, and what the rules would say is kept, so removing it brings that back.

use kampus_access::{Actor, PrincipalId};
use kampus_activities::ModuleId;
use kampus_common::{Error, FieldError, Result, Timestamp};
use sqlx::Row;

use super::principal;
use crate::permissions::{EXCEPTION, VIEW_ALL};
use crate::{ActivityState, Exception, Progress, State, events, sql};

/// The most characters of a reason.
const REASON_LIMIT: usize = 500;

impl Progress {
    /// Decides the state of a person on a course module, replacing an earlier decision; needs
    /// `progress.exception` in the course module.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`, `validation` (`progress.exception_invalid`) for an empty or very
    /// long reason.
    pub async fn set_exception(
        &self,
        actor: &Actor,
        module: ModuleId,
        account: PrincipalId,
        decided: State,
        reason: &str,
    ) -> Result<ActivityState> {
        let scope = self.activities.scope_of(module).await?;
        self.access.require(actor, EXCEPTION, scope).await?;
        let reason = reason.trim();
        if reason.is_empty() || reason.chars().count() > REASON_LIMIT {
            return Err(Error::validation(
                "progress.exception_invalid",
                vec![FieldError::new("reason", "length")],
            ));
        }
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "insert into {t} (course_module_id, account_id, state, reason, author) values ($1, $2, $3, $4, $5)
             on conflict (course_module_id, account_id) do update set state = excluded.state,
                 reason = excluded.reason, author = excluded.author, at = now()",
            t = self.table("activity_exception")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .bind(decided.code())
        .bind(reason)
        .bind(principal(actor).map(|p| p.as_uuid()))
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        let found = self.evaluate_in(&mut tx, module, account, "exception").await?;
        self.publish_in(
            &mut tx,
            events::EXCEPTION_CHANGED,
            module,
            Some((account, decided)),
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(found)
    }

    /// Removes the decision about a person, and what the rules say counts again; needs `progress.exception`.
    ///
    /// # Errors
    ///
    /// `not_found` (`progress.exception_not_found`), `permission_denied`.
    pub async fn clear_exception(
        &self,
        actor: &Actor,
        module: ModuleId,
        account: PrincipalId,
    ) -> Result<ActivityState> {
        let scope = self.activities.scope_of(module).await?;
        self.access.require(actor, EXCEPTION, scope).await?;
        let mut tx = self.db.begin().await?;
        let removed = sqlx::query(sql(format!(
            "delete from {} where course_module_id = $1 and account_id = $2",
            self.table("activity_exception")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if removed == 0 {
            return Err(Error::not_found("progress.exception_not_found"));
        }
        let found = self
            .evaluate_in(&mut tx, module, account, "exception_removed")
            .await?;
        self.publish_in(
            &mut tx,
            events::EXCEPTION_CHANGED,
            module,
            Some((account, found.state)),
            principal(actor),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(found)
    }

    /// The decisions about a course module; needs `progress.exception` or `progress.view_all` in it.
    ///
    /// # Errors
    ///
    /// `not_found`, `permission_denied`.
    pub async fn exceptions(&self, actor: &Actor, module: ModuleId) -> Result<Vec<Exception>> {
        let scope = self.activities.scope_of(module).await?;
        if !self.access.check(actor, VIEW_ALL, scope).await?.allowed {
            self.access.require(actor, EXCEPTION, scope).await?;
        }
        let rows = sqlx::query(sql(format!(
            "select course_module_id, account_id, state, reason, author, at from {}
             where course_module_id = $1 order by at, account_id",
            self.table("activity_exception")
        )))
        .bind(module.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Exception {
                course_module: ModuleId::from_uuid(r.get("course_module_id")),
                account: PrincipalId::from_uuid(r.get("account_id")),
                state: State::from_code(&r.get::<String, _>("state")),
                reason: r.get("reason"),
                author: r.get("author"),
                at: Timestamp::from_offset_date_time(r.get("at")),
            })
            .collect())
    }
}
