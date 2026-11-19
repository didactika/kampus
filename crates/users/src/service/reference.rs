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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! External references of system groups: the identifiers other systems give a group, such as the code
//! of a cohort in a student information system, one per system and group, unique in the campus. They are
//! read and set with `users.system_group.manage`, and an integration creates a group by one.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use kampus_db::references::{ExternalReference, References};

use crate::permissions::GROUP_MANAGE;
use crate::{GroupId, GroupKind, Users};

impl Users {
    /// The references of system groups.
    pub(crate) fn group_references(&self) -> References {
        References::new(&self.db, crate::module::NAME, "system_group_reference")
    }

    /// The campus of a scope: the scope itself, or the campus above it.
    async fn campus_of(&self, scope: ScopeId) -> Result<ScopeId> {
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let path = self.access.path(&mut conn, scope).await?;
        Ok(path
            .iter()
            .find(|s| s.kind == kampus_access::ScopeKind::Campus)
            .map_or(scope, |s| s.id))
    }

    /// The identifiers systems give a system group; needs `users.system_group.manage` where it is.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group, `permission_denied` without the permission.
    pub async fn group_external_references(
        &self,
        actor: &Actor,
        id: GroupId,
    ) -> Result<Vec<ExternalReference>> {
        let group = self.group(id).await?;
        self.access.require(actor, GROUP_MANAGE, group.scope).await?;
        self.group_references().of(id.as_uuid()).await
    }

    /// Gives a system group the identifier a system uses for it, replacing the one that system gave it;
    /// needs `users.system_group.manage` where it is.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group, `permission_denied` without the permission, `conflict`
    /// (`references.taken`) when another group of the campus has that identifier of the system.
    pub async fn set_group_reference(
        &self,
        actor: &Actor,
        id: GroupId,
        reference: &ExternalReference,
    ) -> Result<Vec<ExternalReference>> {
        let group = self.group(id).await?;
        self.access.require(actor, GROUP_MANAGE, group.scope).await?;
        let campus = self.campus_of(group.scope).await?;
        let references = self.group_references();
        references.set(campus.as_uuid(), id.as_uuid(), reference).await?;
        references.of(id.as_uuid()).await
    }

    /// Removes the identifier a system gave a system group; needs `users.system_group.manage` where it is.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown group or one the system gave no identifier, `permission_denied` without
    /// the permission.
    pub async fn remove_group_reference(&self, actor: &Actor, id: GroupId, system: &str) -> Result<()> {
        let group = self.group(id).await?;
        self.access.require(actor, GROUP_MANAGE, group.scope).await?;
        if self.group_references().remove(id.as_uuid(), system).await? {
            Ok(())
        } else {
            Err(Error::not_found("references.not_found"))
        }
    }

    /// The system group a system knows by an identifier in a campus.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn group_by_reference(
        &self,
        campus: ScopeId,
        reference: &ExternalReference,
    ) -> Result<Option<GroupId>> {
        Ok(self
            .group_references()
            .find(campus.as_uuid(), reference)
            .await?
            .map(GroupId::from_uuid))
    }

    /// The system group a system knows by an identifier in the campus of a scope, created in the scope with
    /// that identifier when there is none; returns it and whether it was created. A group that exists is
    /// left as it is.
    ///
    /// # Errors
    ///
    /// Those of [`Users::create_group`].
    pub async fn ensure_group_by_reference(
        &self,
        actor: &Actor,
        scope: ScopeId,
        reference: &ExternalReference,
        (code, name): (&str, &str),
        kind: GroupKind,
    ) -> Result<(GroupId, bool)> {
        let campus = self.campus_of(scope).await?;
        if let Some(id) = self.group_by_reference(campus, reference).await? {
            return Ok((id, false));
        }
        let id = self
            .create_group_with(actor, scope, (code, name), kind, Some(reference))
            .await?;
        Ok((id, true))
    }
}
