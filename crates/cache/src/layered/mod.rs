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
// @package   kampus-cache
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Two levels of cache: an in-process one in front of a shared one.

use std::sync::Arc;
use std::time::Duration;

use crate::{Cache, LockToken};
use async_trait::async_trait;
use bytes::Bytes;
use kampus_common::Result;

/// Two levels: an in-process cache in front of a shared one. Reads try the local level first and fill it
/// for a short time; writes and deletions go to both; counters and locks live only in the shared level.
#[derive(Debug, Clone)]
pub struct Layered {
    /// The in-process level.
    local: Arc<dyn Cache>,
    /// The shared level.
    shared: Arc<dyn Cache>,
    /// How long a value read from the shared level stays in the local one.
    local_ttl: Duration,
}

impl Layered {
    /// Puts `local` in front of `shared`; values read from `shared` stay at most `local_ttl` in `local`.
    #[must_use]
    pub fn new(local: Arc<dyn Cache>, shared: Arc<dyn Cache>, local_ttl: Duration) -> Self {
        Self {
            local,
            shared,
            local_ttl,
        }
    }
}

#[async_trait]
impl Cache for Layered {
    fn shared(&self) -> bool {
        self.shared.shared()
    }

    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Bytes>> {
        if let Some(value) = self.local.get(namespace, key).await? {
            return Ok(Some(value));
        }
        let value = self.shared.get(namespace, key).await?;
        if let Some(value) = &value {
            self.local
                .set(namespace, key, value.clone(), self.local_ttl)
                .await?;
        }
        Ok(value)
    }

    async fn set(&self, namespace: &str, key: &str, value: Bytes, ttl: Duration) -> Result<()> {
        self.shared.set(namespace, key, value.clone(), ttl).await?;
        self.local
            .set(namespace, key, value, ttl.min(self.local_ttl))
            .await
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        self.shared.delete(namespace, key).await?;
        self.local.delete(namespace, key).await
    }

    async fn increment(&self, namespace: &str, key: &str, window: Duration) -> Result<u64> {
        self.shared.increment(namespace, key, window).await
    }

    async fn try_lock(&self, namespace: &str, key: &str, lease: Duration) -> Result<Option<LockToken>> {
        self.shared.try_lock(namespace, key, lease).await
    }

    async fn unlock(&self, namespace: &str, key: &str, token: &LockToken) -> Result<bool> {
        self.shared.unlock(namespace, key, token).await
    }

    async fn check(&self) -> Result<()> {
        self.local.check().await?;
        self.shared.check().await
    }
}
