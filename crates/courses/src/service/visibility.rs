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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Effective visibility: a category, course or section is shown to learners only when it and every
//! ancestor are shown now. Each node keeps only its own setting, so hiding a category and showing it again
//! restores exactly what was there.

use kampus_access::{Actor, ScopeId};
use kampus_common::Result;
use sqlx::Row;

use super::visibility_of;
use crate::permissions::HIDDEN_VIEW;
use crate::{Courses, sql};

impl Courses {
    /// Whether a node and every ancestor are shown now; a deleted section is never shown.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn shown(&self, scope: ScopeId) -> Result<bool> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let path: Vec<uuid::Uuid> = self
            .access
            .path(&mut conn, scope)
            .await?
            .iter()
            .map(|s| s.id.as_uuid())
            .collect();
        let rows = sqlx::query(sql(format!(
            "select visibility, visible_from, visible_until, false as deleted from {} where scope_id = any($1)
             union all select visibility, visible_from, visible_until, false from {} where scope_id = any($1)
             union all select visibility, visible_from, visible_until, deleted_at is not null from {}
             where scope_id = any($1)",
            self.table("category"),
            self.table("course"),
            self.table("section")
        )))
        .bind(path.as_slice())
        .fetch_all(&mut *conn)
        .await
        .map_err(kampus_db::classify)?;
        let now = self.now();
        Ok(rows
            .iter()
            .all(|r| !r.get::<bool, _>("deleted") && visibility_of(r).shown_at(now)))
    }

    /// Whether the actor sees a node: it is shown, or the actor holds `courses.hidden.view` there and sees
    /// it marked as hidden.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope.
    pub async fn sees(&self, actor: &Actor, scope: ScopeId) -> Result<bool> {
        if self.shown(scope).await? {
            return Ok(true);
        }
        Ok(self.access.check(actor, HIDDEN_VIEW, scope).await?.allowed)
    }
}

impl Courses {
    /// The children of a node the actor sees, in their order: those shown when the node is shown, and the
    /// others only with `courses.hidden.view` in them. The node's ancestors are read once.
    pub(crate) async fn visible_children<T>(
        &self,
        actor: &Actor,
        parent: ScopeId,
        children: Vec<T>,
        node: impl Fn(&T) -> (ScopeId, crate::Visibility),
    ) -> Result<Vec<T>> {
        let parent_shown = self.shown(parent).await?;
        let now = self.now();
        let mut seen = Vec::with_capacity(children.len());
        for child in children {
            let (scope, visibility) = node(&child);
            if (parent_shown && visibility.shown_at(now))
                || self.access.check(actor, HIDDEN_VIEW, scope).await?.allowed
            {
                seen.push(child);
            }
        }
        Ok(seen)
    }
}
