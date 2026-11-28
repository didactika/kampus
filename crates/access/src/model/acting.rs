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

//! Who makes a request, as the routes of every module see it: the actor, and the permissions an API token
//! carries when the request comes with one.

use serde::{Deserialize, Serialize};

use crate::{Actor, ScopeId};

/// A permission an API token carries, in a scope and the scopes below it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carried {
    /// The permission.
    pub permission: String,
    /// The scope where it starts.
    pub scope: ScopeId,
}

/// The actor of a request and, when it comes with an API token, the permissions that token carries: an
/// operation is allowed only to what both the actor holds and the token carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acting {
    /// The actor.
    pub actor: Actor,
    /// What the API token of the request carries; absent without a token, which limits nothing.
    pub carried: Option<Vec<Carried>>,
}

impl Acting {
    /// The actor of a request without a token.
    #[must_use]
    pub const fn new(actor: Actor) -> Self {
        Self { actor, carried: None }
    }
}
