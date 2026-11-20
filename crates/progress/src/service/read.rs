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

//! Reading progress: a person sees their own states, someone with `progress.view_all` sees anyone's, and how a
//! course module ended is shown only when the grade it depends on is, through the visibility service of
//! records.

use kampus_access::{Actor, PrincipalId};
use kampus_activities::ModuleId;
use kampus_common::{Error, Result, Timestamp};
use kampus_courses::CourseId;

use super::{STATE_COLUMNS, principal, state};
use crate::permissions::{VIEW_ALL, VIEW_OWN};
use crate::{ActivityState, Progress, Source, State, sql};

impl Progress {
    /// Fails unless the actor may see the progress of a person in a scope: their own with
    /// `progress.view_own`, anyone's with `progress.view_all`.
    async fn require_viewing(
        &self,
        actor: &Actor,
        scope: kampus_access::ScopeId,
        account: PrincipalId,
    ) -> Result<()> {
        if self.access.check(actor, VIEW_ALL, scope).await?.allowed {
            return Ok(());
        }
        if principal(actor) == Some(account) && self.access.check(actor, VIEW_OWN, scope).await?.allowed {
            return Ok(());
        }
        Err(Error::permission_denied("access.denied").with_param("permission", VIEW_ALL))
    }

    /// The state of a person on a course module as the actor may see it: not started when there is no row.
    ///
    /// # Errors
    ///
    /// `not_found` for a course module the actor does not see, `permission_denied`.
    pub async fn state_of(
        &self,
        actor: &Actor,
        module: ModuleId,
        account: PrincipalId,
    ) -> Result<ActivityState> {
        let stored = self.require_module(actor, module).await?;
        self.require_viewing(actor, stored.scope, account).await?;
        let course = self.activities.course_of_module(module).await?;
        let shown = self
            .records
            .modules_with_visible_grade(actor, course, account)
            .await?;
        let row = sqlx::query(sql(format!(
            "select {STATE_COLUMNS} from {} where course_module_id = $1 and account_id = $2",
            self.table("activity_state")
        )))
        .bind(module.as_uuid())
        .bind(account.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let found = row.as_ref().map_or_else(
            || ActivityState {
                course_module: module,
                account,
                state: State::NotStarted,
                source: Source::Automatic,
                at: Timestamp::from_offset_date_time(time::OffsetDateTime::now_utc()),
                version: 0,
            },
            state,
        );
        Ok(masked(found, &shown))
    }

    /// The states of a person in a course as the actor may see them: those of the course modules they see,
    /// where the person has started; the others are not started.
    ///
    /// # Errors
    ///
    /// `not_found` for a course the actor does not see, `permission_denied`.
    pub async fn states_for(
        &self,
        actor: &Actor,
        course: CourseId,
        account: PrincipalId,
    ) -> Result<Vec<ActivityState>> {
        let found = self.courses.course(course).await?;
        if !self.courses.sees(actor, found.scope).await? {
            return Err(Error::not_found("courses.course_not_found"));
        }
        self.require_viewing(actor, found.scope, account).await?;
        let shown = self
            .records
            .modules_with_visible_grade(actor, course, account)
            .await?;
        let mut modules = Vec::new();
        for section in self.courses.sections(actor, course).await? {
            for module in self.activities.modules(actor, section.id).await? {
                modules.push(module.id.as_uuid());
            }
        }
        let rows = sqlx::query(sql(format!(
            "select {STATE_COLUMNS} from {} where course_module_id = any($1) and account_id = $2",
            self.table("activity_state")
        )))
        .bind(&modules)
        .bind(account.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        let mut found: Vec<ActivityState> = rows.iter().map(state).map(|s| masked(s, &shown)).collect();
        found.sort_by_key(|s| modules.iter().position(|m| *m == s.course_module.as_uuid()));
        Ok(found)
    }
}

/// A state with how it ended taken out when the grade it depends on is not shown.
fn masked(mut found: ActivityState, shown: &std::collections::HashSet<ModuleId>) -> ActivityState {
    if !shown.contains(&found.course_module) {
        found.state = found.state.without_outcome();
    }
    found
}
