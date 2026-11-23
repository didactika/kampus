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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Who a scope of a person belongs to, so each person arranges their own personal pages, such as their home.

use async_trait::async_trait;
use kampus_access::{PrincipalId, ScopeId};
use kampus_common::Result;

/// Who owns a scope of a person.
#[async_trait]
pub trait ScopeOwners: Send + Sync + std::fmt::Debug {
    /// The person a scope belongs to; none for a scope that is not a person's.
    async fn owner_of(&self, scope: ScopeId) -> Result<Option<PrincipalId>>;
}

/// No scope belongs to anyone: personal pages are arranged only by who may arrange the scope, until accounts answer.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOwners;

#[async_trait]
impl ScopeOwners for NoOwners {
    async fn owner_of(&self, _scope: ScopeId) -> Result<Option<PrincipalId>> {
        Ok(None)
    }
}
