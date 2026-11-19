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
// @package   kampus-events
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The process that delivers events: it dispatches the committed events of the outbox and runs their
//! deliveries with the listeners this build declares, until it is told to stop.

use std::collections::HashMap;
use std::time::Duration;

use kampus_common::Result;
use kampus_module::{EventRun, ListenerDeclaration, Module, ServiceContext};
use sqlx::Row;
use tokio::sync::watch;

use crate::service::delivery::TIMEOUT_SECONDS;
use crate::{Events, sql};

/// How long an idle dispatcher waits before it looks for events again.
const IDLE: Duration = Duration::from_millis(500);

/// The key of a declared listener: its module, its name and its event.
type Key = (String, String, String);

/// Dispatches events and runs their deliveries.
#[derive(Clone)]
pub struct Dispatcher {
    /// The events.
    events: Events,
    /// The listeners of this build.
    declared: HashMap<Key, ListenerDeclaration>,
    /// The services the listeners use.
    services: ServiceContext,
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher")
            .field("listeners", &self.declared.len())
            .finish_non_exhaustive()
    }
}

impl Dispatcher {
    /// A dispatcher of the listeners of these modules.
    #[must_use]
    pub fn new(events: Events, modules: &[&Module], services: ServiceContext) -> Self {
        let declared = modules
            .iter()
            .flat_map(|m| {
                m.listeners
                    .iter()
                    .map(|l| ((m.name.to_owned(), l.name.to_owned(), l.event.to_owned()), *l))
            })
            .collect();
        Self {
            events,
            declared,
            services,
        }
    }

    /// The listeners of this build, by their id.
    async fn listener_ids(&self) -> Result<HashMap<uuid::Uuid, ListenerDeclaration>> {
        let rows = sqlx::query(sql(format!(
            "select id, module, name, type_code from {} where retired_at is null",
            self.events.table("listener")
        )))
        .fetch_all(self.events.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .filter_map(|r| {
                let key: Key = (r.get("module"), r.get("name"), r.get("type_code"));
                self.declared.get(&key).map(|l| (r.get("id"), *l))
            })
            .collect())
    }

    /// Dispatches the outbox, then runs one delivery; whether there was anything to do.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails; a listener's failure is recorded, not returned.
    pub async fn run_once(&self) -> Result<bool> {
        let dispatched = self.events.dispatch().await?;
        let listeners = self.listener_ids().await?;
        let ids: Vec<uuid::Uuid> = listeners.keys().copied().collect();
        let Some(delivery) = self.events.claim(&ids).await? else {
            return Ok(dispatched > 0);
        };
        let Some(listener) = listeners.get(&delivery.listener) else {
            return Ok(true);
        };
        let run = EventRun {
            event: &delivery.event,
            services: &self.services,
        };
        let timeout = Duration::from_secs(u64::try_from(TIMEOUT_SECONDS).unwrap_or(60));
        let error = match tokio::time::timeout(timeout, (listener.run)(run)).await {
            Ok(Ok(())) => None,
            Ok(Err(error)) => Some(error.to_string()),
            Err(_) => Some("the listener went past its timeout".to_owned()),
        };
        self.events.finish(&delivery, error).await?;
        Ok(true)
    }

    /// Delivers events until `stop` turns true.
    pub async fn run(self, mut stop: watch::Receiver<bool>) {
        tracing::info!(listeners = self.declared.len(), "event dispatcher started");
        while !*stop.borrow() {
            let busy = match self.run_once().await {
                Ok(busy) => busy,
                Err(error) => {
                    tracing::error!(%error, "event dispatcher could not reach the database");
                    false
                }
            };
            if !busy {
                tokio::select! {
                    () = tokio::time::sleep(IDLE) => {}
                    _ = stop.changed() => {}
                }
            }
        }
        tracing::info!("event dispatcher stopped");
    }
}
