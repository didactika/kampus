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

//! The limit of an API token: a request with one may do only what the token carries in the scope or above
//! it, besides what its actor holds.

use kampus_common::{Error, Result};

use crate::{Access, Acting, Actor, ScopeId};

impl Access {
    /// Whether what a request acts with lets it use a permission in a scope: always without a token; with
    /// one, when the token carries the permission in the scope or a scope above it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown scope, `unavailable` when the database fails.
    pub async fn carries(&self, acting: &Acting, permission: &str, scope: ScopeId) -> Result<bool> {
        let Some(carried) = &acting.carried else {
            return Ok(true);
        };
        if !carried.iter().any(|c| c.permission == permission) {
            return Ok(false);
        }
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let path = self.path(&mut conn, scope).await?;
        Ok(carried
            .iter()
            .any(|c| c.permission == permission && path.iter().any(|p| p.id == c.scope)))
    }

    /// The actor of an operation that needs a permission in a scope, once the token of the request, if
    /// any, carries it there. The service that does the operation still checks the actor holds it.
    ///
    /// # Errors
    ///
    /// `permission_denied` (`access.token_scope`) when the token does not carry the permission there, and
    /// the errors of [`Access::carries`].
    pub async fn acting<'a>(
        &self,
        acting: &'a Acting,
        permission: &str,
        scope: ScopeId,
    ) -> Result<&'a Actor> {
        if self.carries(acting, permission, scope).await? {
            Ok(&acting.actor)
        } else {
            Err(Error::permission_denied("access.token_scope")
                .with_param("permission", permission.to_owned()))
        }
    }
}
