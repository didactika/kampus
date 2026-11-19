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

//! The services of the modules, built once per process and shared by their routes through the
//! [`ServiceContext`]: each module reads the services it needs by type.

use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{Clock, SystemClock};
use kampus_courses::Courses;
use kampus_db::Database;
use kampus_enrol::{Enrol, Participation};
use kampus_events::Events;
use kampus_identity::http::HttpSettings;
use kampus_identity::{Courier, Identity, PasswordCost, PasswordHasher, SecretKey, SettingsPolicies};
use kampus_module::ServiceContext;
use kampus_settings::Settings;
use kampus_tasks::Tasks;
use kampus_users::Users;

use crate::wiring::registry::MODULES;

/// What the services of a process are built from.
pub struct Parts {
    /// The database of the installation.
    pub db: Database,
    /// The public address of the site.
    pub site_url: String,
    /// The key that seals what identity reads back.
    pub secret_key: SecretKey,
    /// Delivers the notices of identity.
    pub courier: Arc<dyn Courier>,
    /// Where "now" comes from.
    pub clock: Arc<dyn Clock>,
    /// The cost of new password hashes; the recommended one when absent.
    pub password_cost: Option<PasswordCost>,
}

impl Parts {
    /// The parts of a process with the system clock.
    #[must_use]
    pub fn new(db: Database, site_url: String, secret_key: SecretKey, courier: Arc<dyn Courier>) -> Self {
        Self {
            db,
            site_url,
            secret_key,
            courier,
            clock: Arc::new(SystemClock),
            password_cost: None,
        }
    }
}

/// The services of a process.
#[derive(Debug, Clone)]
pub struct Services {
    /// Access.
    pub access: Access,
    /// Identity, which also authenticates every request.
    pub identity: Identity,
    /// What every module's routes read.
    pub context: ServiceContext,
}

impl Services {
    /// Builds the services of the modules of this build.
    #[must_use]
    pub fn build(parts: Parts) -> Self {
        let access = Access::new(parts.db.clone(), parts.clock.clone());
        let settings = Settings::new(parts.db.clone(), access.clone());
        let identity = Identity::new(
            parts.db.clone(),
            access.clone(),
            parts.clock.clone(),
            Arc::new(SettingsPolicies::new(settings.clone())),
            parts.courier,
            parts.secret_key,
        );
        let identity = match parts.password_cost {
            Some(cost) => identity.with_password_cost(cost),
            None => identity,
        };
        let preferences = MODULES
            .iter()
            .flat_map(|m| m.preferences.iter().copied())
            .collect();
        let users = Users::new(parts.db.clone(), access.clone(), identity.clone(), preferences);
        let courses = Courses::new(
            parts.db.clone(),
            access.clone(),
            parts.clock.clone(),
            Arc::new(Participation::new(parts.db.clone())),
        );
        let hasher = PasswordHasher::new(parts.password_cost.unwrap_or_default());
        let tasks = Tasks::new(parts.db.clone());
        let events = Events::new(parts.db.clone());
        let enrol = Enrol::new(
            parts.db,
            access.clone(),
            courses.clone(),
            identity.clone(),
            users.clone(),
            parts.clock,
            hasher,
        );
        let context = ServiceContext::default()
            .with(access.clone())
            .with(identity.clone())
            .with(users)
            .with(courses)
            .with(enrol)
            .with(tasks)
            .with(events)
            .with(settings)
            .with(HttpSettings {
                api_url: format!(
                    "{}{}",
                    parts.site_url.trim_end_matches('/'),
                    crate::http::API_PREFIX
                ),
                site_url: parts.site_url,
            });
        Self {
            access,
            identity,
            context,
        }
    }

    /// The routes every module of the build declares.
    pub fn routes(&self) -> axum::Router {
        MODULES
            .iter()
            .filter_map(|m| m.routes)
            .fold(axum::Router::new(), |router, routes| {
                router.merge(routes(&self.context))
            })
    }
}
