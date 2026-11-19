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
// @package   kampus-courses
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The HTTP routes of courses: categories, courses, sections and groups, read as the caller sees them and
//! changed with `If-Match`.

mod categories;
mod courses;
mod fields;
mod groups;
mod sections;

use crate::Courses;
use axum::Router;
use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, Result};
use kampus_identity::Identity;
use kampus_identity::http::Caller;
use kampus_module::ServiceContext;

/// The routes of courses.
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared {
        courses: context.service::<Courses>().ok(),
        identity: context.service::<Identity>().ok(),
    };
    Router::new()
        .merge(categories::routes())
        .merge(courses::routes())
        .merge(fields::routes())
        .merge(sections::routes())
        .merge(groups::routes())
        .with_state(shared)
}

/// What the handlers share.
#[derive(Debug, Clone)]
struct Shared {
    /// The service.
    courses: Option<Courses>,
    /// Identity, for the permissions API tokens carry.
    identity: Option<Identity>,
}

impl Shared {
    /// The service.
    fn courses(&self) -> Result<&Courses> {
        self.courses
            .as_ref()
            .ok_or_else(|| Error::internal("courses.service_missing"))
    }

    /// The actor of an operation that needs a permission in a scope; an API token acts only within what
    /// it carries.
    async fn acting<'c>(&self, caller: &'c Caller, permission: &str, scope: ScopeId) -> Result<&'c Actor> {
        let identity = self
            .identity
            .as_ref()
            .ok_or_else(|| Error::internal("courses.identity_missing"))?;
        caller.acting(identity, permission, scope).await
    }
}

/// An answer that does not say whether a hidden node exists.
fn hidden(code: &'static str) -> Error {
    Error::not_found(code)
}
