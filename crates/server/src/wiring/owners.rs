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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The owners of the scopes of people, from their accounts: what customisation asks to let each person arrange their
//! own personal pages.

use async_trait::async_trait;
use kampus_access::{PrincipalId, ScopeId};
use kampus_common::Result;
use kampus_customise::ScopeOwners;
use kampus_identity::Identity;

/// The owner of a scope of a person is the account it was made for.
#[derive(Clone)]
pub struct AccountScopes(pub Identity);

impl std::fmt::Debug for AccountScopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccountScopes")
    }
}

#[async_trait]
impl ScopeOwners for AccountScopes {
    async fn owner_of(&self, scope: ScopeId) -> Result<Option<PrincipalId>> {
        Ok(self
            .0
            .account_of_scope(scope)
            .await?
            .map(|account| PrincipalId::from_uuid(account.as_uuid())))
    }
}
