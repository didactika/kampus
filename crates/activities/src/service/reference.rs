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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! External references of course modules: the identifiers other systems give an activity or a material,
//! one per system and course module, unique in the campus. They are read and set with
//! `activities.module.manage`, and an integration creates or updates a course module by one.

use kampus_access::{Actor, ScopeId};
use kampus_common::Result;
use kampus_courses::{Place, SectionId};
use kampus_db::references::{ExternalReference, References};

use crate::permissions::MODULE_MANAGE;
use crate::{Activities, Module, ModuleFields, ModuleId};

impl Activities {
    /// The references of course modules.
    pub(crate) fn module_references(&self) -> References {
        References::new(&self.db, crate::module::NAME, "course_module_reference")
    }

    /// The campus of a course module and the scope its permission is checked in.
    async fn module_place(&self, id: ModuleId) -> Result<(ScopeId, ScopeId)> {
        let module = self.stored(id).await?;
        let section = self.courses.section(module.section).await?;
        let campus = self.courses.course(section.course).await?.campus;
        Ok((campus, module.scope))
    }

    /// The identifiers systems give a course module; needs `activities.module.manage` in it.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown course module, `permission_denied` without the permission.
    pub async fn module_external_references(
        &self,
        actor: &Actor,
        id: ModuleId,
    ) -> Result<Vec<ExternalReference>> {
        let (_, scope) = self.module_place(id).await?;
        self.access.require(actor, MODULE_MANAGE, scope).await?;
        self.module_references().of(id.as_uuid()).await
    }

    /// Gives a course module the identifier a system uses for it, replacing the one that system gave it.
    ///
    /// # Errors
    ///
    /// As [`Activities::module_external_references`], and `conflict` (`references.taken`) when another course
    /// module of the campus has it.
    pub async fn set_module_reference(
        &self,
        actor: &Actor,
        id: ModuleId,
        reference: &ExternalReference,
    ) -> Result<Vec<ExternalReference>> {
        let (campus, scope) = self.module_place(id).await?;
        self.access.require(actor, MODULE_MANAGE, scope).await?;
        let references = self.module_references();
        references.set(campus.as_uuid(), id.as_uuid(), reference).await?;
        references.of(id.as_uuid()).await
    }

    /// Removes the identifier a system gave a course module.
    ///
    /// # Errors
    ///
    /// As [`Activities::module_external_references`], and `not_found` (`references.not_found`) when the
    /// system gave none.
    pub async fn remove_module_reference(&self, actor: &Actor, id: ModuleId, system: &str) -> Result<()> {
        let (_, scope) = self.module_place(id).await?;
        self.access.require(actor, MODULE_MANAGE, scope).await?;
        if self.module_references().remove(id.as_uuid(), system).await? {
            Ok(())
        } else {
            Err(kampus_common::Error::not_found("references.not_found"))
        }
    }

    /// The course module a system knows by an identifier in a campus.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn module_by_reference(
        &self,
        campus: ScopeId,
        reference: &ExternalReference,
    ) -> Result<Option<ModuleId>> {
        Ok(self
            .module_references()
            .find(campus.as_uuid(), reference)
            .await?
            .map(ModuleId::from_uuid))
    }

    /// Updates the course module a system knows by an identifier, or adds one of a type to a section with
    /// that identifier last; returns it and whether it was added.
    ///
    /// # Errors
    ///
    /// Those of adding and of updating a course module.
    pub async fn upsert_module_by_reference(
        &self,
        actor: &Actor,
        section: SectionId,
        type_code: &str,
        reference: &ExternalReference,
        fields: ModuleFields,
    ) -> Result<(Module, bool)> {
        let target = self.courses.section(section).await?;
        let campus = self.courses.course(target.course).await?.campus;
        match self.module_by_reference(campus, reference).await? {
            Some(id) => Ok((self.update_module(actor, id, fields, None).await?, false)),
            None => Ok((
                self.create_module_with(actor, section, type_code, fields, Place::Last, Some(reference))
                    .await?,
                true,
            )),
        }
    }
}
