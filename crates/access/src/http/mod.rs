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

//! The HTTP routes of access, for its administration: the roles of a campus and their values in
//! [`roles`], and the grants, overrides and explanations of a scope in [`scopes`].

mod roles;
mod scopes;

use axum::Router;
use kampus_common::{Error, Result};
use kampus_module::{Operation, ServiceContext};

use crate::Access;

/// The operations of the routes, one list per file of routes.
pub const OPERATIONS: &[&[Operation]] = &[roles::OPERATIONS, scopes::OPERATIONS];

/// The routes of access.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        access: context.service::<Access>().ok(),
    };
    Router::new()
        .merge(roles::routes())
        .merge(scopes::routes())
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    access: Option<Access>,
}

impl Shared {
    /// The service.
    fn access(&self) -> Result<&Access> {
        self.access
            .as_ref()
            .ok_or_else(|| Error::internal("access.service_missing"))
    }
}
