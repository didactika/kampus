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

//! The service API of progress, one file per area, and what they share: reading states from their rows,
//! evaluating the state of a person on a course module, and the events.

pub mod exception;
pub mod read;
pub mod recalc;
pub mod rule;

use kampus_access::{Actor, PrincipalId};
use kampus_activities::ModuleId;
use kampus_common::{Error, Result, Timestamp};
use serde_json::json;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, Row};

use crate::support::evaluate::{Facts, evaluate};
use crate::{ActivityState, Progress, RuleKind, Source, State, events, sql};

/// The columns of a state.
pub(crate) const STATE_COLUMNS: &str = "course_module_id, account_id, state, source, at, version";

/// A state from its row.
pub(crate) fn state(row: &PgRow) -> ActivityState {
    ActivityState {
        course_module: ModuleId::from_uuid(row.get("course_module_id")),
        account: PrincipalId::from_uuid(row.get("account_id")),
        state: State::from_code(&row.get::<String, _>("state")),
        source: Source::from_code(&row.get::<String, _>("source")),
        at: Timestamp::from_offset_date_time(row.get("at")),
        version: row.get("version"),
    }
}

/// The principal behind an actor.
pub(crate) const fn principal(actor: &Actor) -> Option<PrincipalId> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(*p),
        Actor::Visitor(_) => None,
    }
}

impl Progress {
    /// The rules of a course module, whoever asks.
    pub(crate) async fn rules_in(&self, conn: &mut PgConnection, module: ModuleId) -> Result<Vec<RuleKind>> {
        let codes: Vec<String> = sqlx::query_scalar(sql(format!(
            "select kind from {} where course_module_id = $1 order by kind",
            self.table("rule")
        )))
        .bind(module.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        Ok(codes.iter().map(|c| RuleKind::from_code(c)).collect())
    }

    /// What a person has done on a course module.
    async fn facts_in(
        &self,
        conn: &mut PgConnection,
        module: ModuleId,
        account: PrincipalId,
    ) -> Result<Facts> {
        let kinds: Vec<String> = sqlx::query_scalar(sql(format!(
            "select kind from {} where course_module_id = $1 and account_id = $2",
            self.table("signal")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let grade = self.records.activity_grade(module, account).await?;
        Ok(Facts {
            viewed: kinds.iter().any(|k| k == "view"),
            submitted: kinds.iter().any(|k| k == "submit"),
            marked: kinds.iter().any(|k| k == "manual"),
            grade: grade.and_then(|g| g.value),
            pass_grade: grade.and_then(|g| g.pass_grade),
        })
    }

    /// Evaluates the state of a person on a course module, and when it differs from the stored one writes it,
    /// keeps the change in the history and publishes it. A teacher's exception wins over the rules. A person
    /// who has not started has no row.
    pub(crate) async fn evaluate_in(
        &self,
        conn: &mut PgConnection,
        module: ModuleId,
        account: PrincipalId,
        cause: &str,
    ) -> Result<ActivityState> {
        let exception: Option<String> = sqlx::query_scalar(sql(format!(
            "select state from {} where course_module_id = $1 and account_id = $2",
            self.table("activity_exception")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let (wanted, source) = if let Some(code) = exception {
            (State::from_code(&code), Source::Exception)
        } else {
            let rules = self.rules_in(conn, module).await?;
            let facts = self.facts_in(conn, module, account).await?;
            (evaluate(&rules, &facts), Source::Automatic)
        };
        let row = sqlx::query(sql(format!(
            "select {STATE_COLUMNS} from {} where course_module_id = $1 and account_id = $2 for update",
            self.table("activity_state")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let current = row.as_ref().map(state);
        if let Some(current) = &current
            && current.state == wanted
            && current.source == source
        {
            return Ok(current.clone());
        }
        if current.is_none() && wanted == State::NotStarted {
            return Ok(ActivityState {
                course_module: module,
                account,
                state: wanted,
                source,
                at: Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc()),
                version: 0,
            });
        }
        let stored = sqlx::query(sql(format!(
            "insert into {t} (course_module_id, account_id, state, source) values ($1, $2, $3, $4)
             on conflict (course_module_id, account_id) do update set state = excluded.state,
                 source = excluded.source, at = now(), version = {t}.version + 1
             returning {STATE_COLUMNS}",
            t = self.table("activity_state")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .bind(wanted.code())
        .bind(source.code())
        .fetch_one(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        sqlx::query(sql(format!(
            "insert into {} (id, course_module_id, account_id, from_state, to_state, cause)
             values ($1, $2, $3, $4, $5, $6)",
            self.table("activity_history")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .bind(current.as_ref().map(|c| c.state.code()))
        .bind(wanted.code())
        .bind(cause)
        .execute(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        self.publish_in(
            conn,
            events::ACTIVITY_CHANGED,
            module,
            Some((account, wanted)),
            None,
        )
        .await?;
        Ok(state(&stored))
    }

    /// Publishes an event about a course module, and about a person when there is one, in the transaction of
    /// the change.
    pub(crate) async fn publish_in(
        &self,
        conn: &mut PgConnection,
        code: &str,
        module: ModuleId,
        about: Option<(PrincipalId, State)>,
        actor: Option<PrincipalId>,
    ) -> Result<()> {
        let course = self.activities.course_of_module(module).await?;
        let scope = self.activities.scope_of(module).await?;
        let mut payload = json!({ "module": module, "course": course });
        if let Some((account, state)) = about {
            payload["account"] = json!(account);
            payload["state"] = json!(state);
        }
        let mut event = kampus_events::Publish::new(code, Some(module.as_uuid()), payload);
        event.scope = Some(scope.as_uuid());
        event.actor = actor.map(|a| a.as_uuid());
        event.summary = Some(format!("{code} {module}"));
        kampus_events::Events::new(self.db.clone())
            .publish_in(conn, event)
            .await
            .map(drop)
    }

    /// Fails with `not_found` unless the actor sees the course module.
    pub(crate) async fn require_module(
        &self,
        actor: &Actor,
        module: ModuleId,
    ) -> Result<kampus_activities::Module> {
        self.activities
            .module(actor, module)
            .await
            .map_err(|_| Error::not_found("progress.module_not_found"))
    }
}
