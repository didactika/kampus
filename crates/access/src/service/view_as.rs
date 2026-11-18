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
// @package   kampus-access
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! "View as": a principal sees a scope and its descendants as another role, read-only,
//! until the session ends; starting and ending it is recorded.

use kampus_common::{Error, Result};
use serde_json::json;

use crate::permissions::VIEW_AS;
use crate::{Access, Actor, AllowanceKind, PrincipalId, RoleId, ScopeId, ViewAsId, sql};

impl Access {
    /// Starts seeing a scope as a role; needs `access.view_as` in it and an allowance for the role. Checks
    /// made with [`Actor::ViewAs`] and the returned session then read as that role only.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission or the allowance.
    pub async fn start_view_as(
        &self,
        principal: PrincipalId,
        scope: ScopeId,
        role: RoleId,
    ) -> Result<ViewAsId> {
        let mut tx = self.db.begin().await?;
        let actor = Actor::Principal(principal);
        self.require_with_allowance(&mut tx, &actor, VIEW_AS, scope, role, AllowanceKind::ViewAs)
            .await?;
        let id = ViewAsId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, principal_id, scope_id, role_id, started_at) values ($1, $2, $3, $4, $5)",
            self.table("view_as_session")
        )))
        .bind(id.as_uuid())
        .bind(principal.as_uuid())
        .bind(scope.as_uuid())
        .bind(role.as_uuid())
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        self.record(
            &mut tx,
            Some(principal),
            "view_as",
            &id.to_string(),
            None,
            Some(json!({ "scope": scope, "role": role })),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        Ok(id)
    }

    /// Ends a "view as" session of the principal.
    ///
    /// # Errors
    ///
    /// `not_found` when the principal has no such open session.
    pub async fn end_view_as(&self, principal: PrincipalId, session: ViewAsId) -> Result<()> {
        let mut tx = self.db.begin().await?;
        let ended = sqlx::query(sql(format!(
            "update {} set ended_at = greatest($3, started_at + interval '1 microsecond') where id = $1 and principal_id = $2 and ended_at is null",
            self.table("view_as_session")
        )))
        .bind(session.as_uuid())
        .bind(principal.as_uuid())
        .bind(self.now().as_offset_date_time())
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?
        .rows_affected();
        if ended == 0 {
            return Err(Error::not_found("access.view_as_not_found"));
        }
        self.record(
            &mut tx,
            Some(principal),
            "view_as",
            &session.to_string(),
            Some(json!("open")),
            Some(json!("ended")),
        )
        .await?;
        tx.commit().await.map_err(kampus_db::classify)
    }
}
