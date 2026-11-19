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

//! Which campus a request is for: with one campus, the campus of the installation; with a campus per
//! domain, the campus of the request's host, once campuses have domains.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use kampus_access::{Access, ScopeId};
use kampus_common::{Error, Result};
use kampus_identity::http::CurrentCampus;
use kampus_module::http::ApiError;
use tokio::sync::OnceCell;

use crate::config::Tenancy;

/// Finds the campus of each request.
#[derive(Debug, Clone)]
pub struct Campuses {
    /// How campuses are served.
    tenancy: Tenancy,
    /// Access, which knows the campuses.
    access: Access,
    /// The campus of a single-campus installation, read once.
    single: Arc<OnceCell<ScopeId>>,
}

impl Campuses {
    /// Finds campuses as the configuration says.
    #[must_use]
    pub fn new(tenancy: Tenancy, access: Access) -> Self {
        Self {
            tenancy,
            access,
            single: Arc::new(OnceCell::new()),
        }
    }

    /// The campus of a request.
    async fn of(&self) -> Result<ScopeId> {
        match &self.tenancy {
            Tenancy::Single => self
                .single
                .get_or_try_init(|| async {
                    self.access
                        .campuses()
                        .await?
                        .first()
                        .copied()
                        .ok_or_else(|| Error::not_found("server.campus_unknown"))
                })
                .await
                .copied(),
            Tenancy::Multi { .. } => Err(Error::not_found("server.campus_unknown")),
        }
    }
}

/// Sets the [`CurrentCampus`] of a request, or answers that the address has no campus.
pub async fn resolve(State(campuses): State<Campuses>, mut request: Request, next: Next) -> Response {
    match campuses.of().await {
        Ok(campus) => {
            request.extensions_mut().insert(CurrentCampus(campus));
            next.run(request).await
        }
        Err(error) => ApiError(error).into_response(),
    }
}
