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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The services the server composed once per process, by type, shared by the routes and the jobs of
//! every module.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use kampus_common::Error;

/// The services the server composed, by type, shared by the routes and jobs of every module.
#[derive(Clone, Default)]
pub struct ServiceContext {
    /// The services, by their type.
    services: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl std::fmt::Debug for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceContext")
            .field("services", &self.services.len())
            .finish()
    }
}

impl ServiceContext {
    /// A context with these services.
    #[must_use]
    pub fn with<T: Any + Send + Sync>(mut self, service: T) -> Self {
        Arc::make_mut(&mut self.services).insert(TypeId::of::<T>(), Arc::new(service));
        self
    }

    /// A service of the context.
    ///
    /// # Errors
    ///
    /// `internal` (`module.service_missing`) when the server did not compose it.
    pub fn service<T: Any + Send + Sync + Clone>(&self) -> Result<T, Error> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<T>())
            .cloned()
            .ok_or_else(|| {
                Error::internal("module.service_missing").with_param("type", std::any::type_name::<T>())
            })
    }
}
