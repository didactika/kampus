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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! External references of accounts: the identifiers other systems give a person, such as the number of a
//! student information system, one per system and account, unique in the campus. They are read with
//! `identity.account.read` and set with `identity.account.invite`, which creates accounts.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use kampus_db::references::{ExternalReference, References};

use crate::permissions::{ACCOUNT_INVITE, ACCOUNT_READ};
use crate::{Account, AccountId, Identity};

impl Identity {
    /// The references of accounts.
    fn reference_store(&self) -> References {
        References::new(&self.db, crate::module::NAME, "account_reference")
    }

    /// The identifiers systems give an account; needs `identity.account.read` in its campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown account, `permission_denied` without the permission.
    pub async fn account_references(&self, actor: &Actor, id: AccountId) -> Result<Vec<ExternalReference>> {
        let account = self.account(actor, id).await?;
        self.access.require(actor, ACCOUNT_READ, account.campus).await?;
        self.reference_store().of(account.id.as_uuid()).await
    }

    /// Gives an account the identifier a system uses for it, replacing the one that system gave it before;
    /// needs `identity.account.invite` in its campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown account, `permission_denied` without the permission, `conflict`
    /// (`references.taken`) when another account of the campus has that identifier of the system.
    pub async fn set_account_reference(
        &self,
        actor: &Actor,
        id: AccountId,
        reference: &ExternalReference,
    ) -> Result<Vec<ExternalReference>> {
        let account = self.account(actor, id).await?;
        self.access.require(actor, ACCOUNT_INVITE, account.campus).await?;
        let references = self.reference_store();
        references
            .set(account.campus.as_uuid(), account.id.as_uuid(), reference)
            .await?;
        references.of(account.id.as_uuid()).await
    }

    /// Removes the identifier a system gave an account; needs `identity.account.invite` in its campus.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown account or one the system gave no identifier, `permission_denied`
    /// without the permission.
    pub async fn remove_account_reference(&self, actor: &Actor, id: AccountId, system: &str) -> Result<()> {
        let account = self.account(actor, id).await?;
        self.access.require(actor, ACCOUNT_INVITE, account.campus).await?;
        if self
            .reference_store()
            .remove(account.id.as_uuid(), system)
            .await?
        {
            Ok(())
        } else {
            Err(Error::not_found("references.not_found"))
        }
    }

    /// The account a system knows by an identifier in a campus; needs `identity.account.read` there.
    ///
    /// # Errors
    ///
    /// `not_found` (`references.not_found`) when no account has it, `permission_denied` without the
    /// permission.
    pub async fn account_by_reference(
        &self,
        actor: &Actor,
        campus: ScopeId,
        reference: &ExternalReference,
    ) -> Result<Account> {
        self.access.require(actor, ACCOUNT_READ, campus).await?;
        let id = self
            .reference_store()
            .find(campus.as_uuid(), reference)
            .await?
            .ok_or_else(|| Error::not_found("references.not_found"))?;
        self.account(actor, AccountId::from_uuid(id)).await
    }
}
