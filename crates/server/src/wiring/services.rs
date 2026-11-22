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
use kampus_customise::Customise;
use kampus_db::Database;
use kampus_enrol::{Enrol, Participation};
use kampus_events::Events;
use kampus_files::Files;
use kampus_identity::http::HttpSettings;
use kampus_identity::{Courier, Identity, PasswordCost, PasswordHasher, SecretKey, SettingsPolicies};
use kampus_module::ServiceContext;
use kampus_notify::ports::transport::{LogTransport, MailTransport, PushTransport, Transport};
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
    /// Where files keep their content; none in a process that does not open storage.
    pub storage: Option<Arc<dyn kampus_storage::Storage>>,
    /// The malware scanner uploads go through; none to keep them unscanned.
    pub scanner: Option<Arc<dyn kampus_scan::Scanner>>,
    /// The mail server of the email channel; none to write mail to the log.
    pub mailer: Option<Arc<dyn kampus_mail::Mailer>>,
    /// The push services of the push channel; none to write push notifications to the log.
    pub pusher: Option<Arc<dyn kampus_push::Pusher>>,
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
            storage: None,
            scanner: None,
            mailer: None,
            pusher: None,
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
        let api_url = format!(
            "{}{}",
            parts.site_url.trim_end_matches('/'),
            crate::http::API_PREFIX
        );
        let files = Files::new(
            parts.db.clone(),
            access.clone(),
            parts.storage.clone(),
            parts.secret_key.derive("files.links"),
            api_url.clone(),
        )
        .with_modules(MODULES)
        .with_scanner(parts.scanner.clone());
        let customise = Customise::new(parts.db.clone(), access.clone()).with_modules(MODULES);
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
        let users = Users::new(parts.db.clone(), access.clone(), identity.clone(), preferences)
            .with_files(files.clone());
        let courses = Courses::new(
            parts.db.clone(),
            access.clone(),
            parts.clock.clone(),
            Arc::new(Participation::new(parts.db.clone(), parts.clock.clone())),
        )
        .with_customise(customise.clone());
        let learning = Learning::build(&parts.db, &access, &courses, &files);
        let email: Arc<dyn Transport> = match parts.mailer.clone() {
            Some(mailer) => Arc::new(MailTransport::new(mailer)),
            None => Arc::new(LogTransport),
        };
        let push: Arc<dyn Transport> = match parts.pusher.clone() {
            Some(pusher) => Arc::new(PushTransport::new(pusher)),
            None => Arc::new(LogTransport),
        };
        let notify = kampus_notify::Notify::new(parts.db.clone())
            .with_transports(email, push)
            .with_courses(courses.clone());
        let privacy = kampus_privacy::Privacy::new(parts.db.clone(), access.clone())
            .with_modules(MODULES)
            .with_files(files.clone());
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
        )
        .with_gate(Arc::new(kampus_progress::PrerequisiteGate::new(
            learning.progress.clone(),
        )));
        let context = ServiceContext::default()
            .with(access.clone())
            .with(identity.clone())
            .with(users)
            .with(courses)
            .with(enrol)
            .with(tasks)
            .with(events)
            .with(settings)
            .with(customise)
            .with(files)
            .with(learning.activities)
            .with(learning.records)
            .with(learning.progress)
            .with(learning.calendar)
            .with(notify)
            .with(privacy)
            .with(HttpSettings {
                api_url,
                site_url: parts.site_url,
            });
        Self {
            access,
            identity,
            context,
        }
    }

    /// The routes every module of the build declares, each under the name of its module, so the routes of
    /// two modules never meet whatever they add.
    pub fn routes(&self) -> axum::Router {
        MODULES
            .iter()
            .fold(axum::Router::new(), |router, module| match module.routes {
                Some(routes) => router.nest(&format!("/{}", module.name), routes(&self.context)),
                None => router,
            })
    }
}

/// The modules that build on courses: course modules and the grades, progress and calendar around them.
struct Learning {
    /// Course modules.
    activities: kampus_activities::Activities,
    /// Grades.
    records: kampus_records::Records,
    /// What people have done.
    progress: kampus_progress::Progress,
    /// Entries and dates.
    calendar: kampus_calendar::Calendar,
}

impl Learning {
    /// Builds them on the services they need.
    fn build(db: &Database, access: &Access, courses: &Courses, files: &Files) -> Self {
        let activities =
            kampus_activities::Activities::new(db.clone(), access.clone(), courses.clone(), files.clone())
                .with_modules(MODULES);
        let records =
            kampus_records::Records::new(db.clone(), access.clone(), courses.clone(), activities.clone());
        let progress = kampus_progress::Progress::new(
            db.clone(),
            access.clone(),
            courses.clone(),
            activities.clone(),
            records.clone(),
        )
        .with_modules(MODULES);
        let calendar =
            kampus_calendar::Calendar::new(db.clone(), access.clone(), courses.clone(), activities.clone());
        Self {
            activities,
            records,
            progress,
            calendar,
        }
    }
}
