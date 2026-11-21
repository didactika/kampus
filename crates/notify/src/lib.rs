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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Notifications of Kampus: modules declare the types of notification they send, listeners create them after
//! the change that causes them commits, and each person reads them in an inbox and chooses which types they
//! receive. Nothing is sent inside the request that causes it.

pub mod db;
pub mod events;
pub mod http;
pub mod jobs;
pub mod model;
pub mod module;
pub mod observers;
pub mod permissions;
pub mod ports;
pub mod privacy;
pub mod service;

use std::sync::Arc;

use kampus_db::Database;

use crate::ports::transport::{LogTransport, Transport};

pub use model::{Inbox, Notification, NotificationId, Preference};
pub use module::MODULE;

/// The channel every notification reaches: the inbox of the person.
pub const INBOX: &str = "inbox";

/// The channel of email.
pub const EMAIL: &str = "email";

/// The channel of push notifications to the devices of a person.
pub const PUSH: &str = "push";

/// Every channel a notification can reach a person through.
pub const CHANNELS: &[&str] = &[INBOX, EMAIL, PUSH];

/// The notifications of an installation.
#[derive(Clone)]
pub struct Notify {
    /// The database of the installation.
    db: Database,
    /// The transport of email.
    email: Arc<dyn Transport>,
    /// The transport of push notifications.
    push: Arc<dyn Transport>,
}

impl std::fmt::Debug for Notify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Notify")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Notify {
    /// The notifications of an installation.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self {
            db,
            email: Arc::new(LogTransport),
            push: Arc::new(LogTransport),
        }
    }

    /// The same with the transports of email and push.
    #[must_use]
    pub fn with_transports(self, email: Arc<dyn Transport>, push: Arc<dyn Transport>) -> Self {
        Self { email, push, ..self }
    }

    /// The quoted name of a table of this module.
    fn table(&self, name: &str) -> String {
        format!("{}.\"{name}\"", self.db.prefix().schema(module::NAME))
    }
}

/// Wraps SQL built by this module as audited: only quoted, prefixed table names are interpolated, and
/// every value is bound.
fn sql(text: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(text)
}
