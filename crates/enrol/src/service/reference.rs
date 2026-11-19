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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! External references of enrolments: the identifiers other systems give an enrolment, such as the
//! registration of a student information system, one per system and enrolment, unique in the campus.
//! They are read with `enrol.enrolment.read` and set with `enrol.enrolment.manage`, and an integration
//! enrols a person through a manual instance by one.

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Result};
use kampus_db::references::{ExternalReference, References};

use super::enrolment::principal;
use crate::permissions::{ENROLMENT_MANAGE, ENROLMENT_READ};
use crate::{Enrol, Enrolment, EnrolmentChange, EnrolmentId, InstanceId, Method, Period};

impl Enrol {
    /// The references of enrolments.
    pub(crate) fn enrolment_references(&self) -> References {
        References::new(&self.db, crate::module::NAME, "enrolment_reference")
    }

    /// The identifiers systems give an enrolment; needs `enrol.enrolment.read` in its course.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown enrolment, `permission_denied` without the permission.
    pub async fn enrolment_external_references(
        &self,
        actor: &Actor,
        id: EnrolmentId,
    ) -> Result<Vec<ExternalReference>> {
        let stored = self.enrolment(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, ENROLMENT_READ, scope).await?;
        self.enrolment_references().of(id.as_uuid()).await
    }

    /// Gives an enrolment the identifier a system uses for it, replacing the one that system gave it;
    /// needs `enrol.enrolment.manage` in its course.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown enrolment, `permission_denied` without the permission, `conflict`
    /// (`references.taken`) when another enrolment of the campus has that identifier of the system.
    pub async fn set_enrolment_reference(
        &self,
        actor: &Actor,
        id: EnrolmentId,
        reference: &ExternalReference,
    ) -> Result<Vec<ExternalReference>> {
        let stored = self.enrolment(id).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, ENROLMENT_MANAGE, course.scope).await?;
        let references = self.enrolment_references();
        references
            .set(course.campus.as_uuid(), id.as_uuid(), reference)
            .await?;
        references.of(id.as_uuid()).await
    }

    /// Removes the identifier a system gave an enrolment; needs `enrol.enrolment.manage` in its course.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown enrolment or one the system gave no identifier, `permission_denied`
    /// without the permission.
    pub async fn remove_enrolment_reference(
        &self,
        actor: &Actor,
        id: EnrolmentId,
        system: &str,
    ) -> Result<()> {
        let stored = self.enrolment(id).await?;
        let scope = self.courses.course(stored.course).await?.scope;
        self.access.require(actor, ENROLMENT_MANAGE, scope).await?;
        if self.enrolment_references().remove(id.as_uuid(), system).await? {
            Ok(())
        } else {
            Err(Error::not_found("references.not_found"))
        }
    }

    /// The enrolment a system knows by an identifier in a campus.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn enrolment_by_reference(
        &self,
        campus: ScopeId,
        reference: &ExternalReference,
    ) -> Result<Option<EnrolmentId>> {
        Ok(self
            .enrolment_references()
            .find(campus.as_uuid(), reference)
            .await?
            .map(EnrolmentId::from_uuid))
    }

    /// Updates the period of the enrolment a system knows by an identifier, or enrols the person through a
    /// manual instance with that identifier; returns it and whether it was created. Needs
    /// `enrol.enrolment.manage` in the course.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an instance that is not manual,
    /// `conflict` (`enrol.reference_other_account`) when the identifier names the enrolment of someone
    /// else, and those of enrolling.
    pub async fn upsert_enrolment_by_reference(
        &self,
        actor: &Actor,
        instance: InstanceId,
        account: PrincipalId,
        period: Period,
        reference: &ExternalReference,
    ) -> Result<(Enrolment, bool)> {
        let stored = self.instance(instance).await?;
        let course = self.courses.course(stored.course).await?;
        self.access.require(actor, ENROLMENT_MANAGE, course.scope).await?;
        if stored.method != Method::Manual {
            return Err(Error::validation(
                "enrol.method_not_manual",
                vec![FieldError::new("instance", "manual")],
            ));
        }
        if let Some(id) = self.enrolment_by_reference(course.campus, reference).await? {
            let existing = self.enrolment(id).await?;
            if existing.account != account {
                return Err(Error::conflict("enrol.reference_other_account"));
            }
            let change = EnrolmentChange {
                status: existing.status,
                starts_at: period.starts_at,
                ends_at: period.ends_at,
                reason: None,
            };
            return Ok((self.update_enrolment(actor, id, change, None).await?, false));
        }
        let id = self
            .create_enrolment_with(
                &stored,
                account,
                period,
                principal(actor),
                Some((course.campus, reference)),
            )
            .await?;
        Ok((self.enrolment(id).await?, true))
    }
}
