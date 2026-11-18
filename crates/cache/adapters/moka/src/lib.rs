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
// @package   kampus-cache-moka
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The in-process cache adapter on moka: the local level of the cache, and the whole cache of an
//! installation that runs a single instance.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bytes::Bytes;
use kampus_cache::{Cache, LockToken, entry_key};
use kampus_common::Result;
use moka::Expiry;

/// A cached value with its own time to live.
#[derive(Debug, Clone)]
struct Entry {
    /// The value.
    value: Bytes,
    /// How long it lives from when it is written.
    ttl: Duration,
}

/// Expires each entry after the time to live it was written with.
#[derive(Debug, Clone, Copy)]
struct PerEntry;

impl Expiry<String, Entry> for PerEntry {
    fn expire_after_create(&self, _key: &String, value: &Entry, _created_at: Instant) -> Option<Duration> {
        Some(value.ttl)
    }

    fn expire_after_update(
        &self,
        _key: &String,
        value: &Entry,
        _updated_at: Instant,
        _left: Option<Duration>,
    ) -> Option<Duration> {
        Some(value.ttl)
    }
}

/// The in-process cache.
#[derive(Debug)]
pub struct MokaCache {
    /// The entries.
    entries: moka::future::Cache<String, Entry>,
    /// The counters: value and end of their window.
    counters: Mutex<HashMap<String, (u64, Instant)>>,
    /// The locks: token and end of their lease.
    locks: Mutex<HashMap<String, (String, Instant)>>,
}

impl MokaCache {
    /// A cache holding at most `max_entries` entries; the least used ones go first.
    #[must_use]
    pub fn new(max_entries: u64) -> Self {
        Self {
            entries: moka::future::Cache::builder()
                .max_capacity(max_entries)
                .expire_after(PerEntry)
                .build(),
            counters: Mutex::new(HashMap::new()),
            locks: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Cache for MokaCache {
    fn shared(&self) -> bool {
        false
    }

    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Bytes>> {
        let key = entry_key(namespace, key)?;
        Ok(self.entries.get(&key).await.map(|entry| entry.value))
    }

    async fn set(&self, namespace: &str, key: &str, value: Bytes, ttl: Duration) -> Result<()> {
        let key = entry_key(namespace, key)?;
        self.entries.insert(key, Entry { value, ttl }).await;
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        let key = entry_key(namespace, key)?;
        self.entries.invalidate(&key).await;
        Ok(())
    }

    async fn increment(&self, namespace: &str, key: &str, window: Duration) -> Result<u64> {
        let key = entry_key(namespace, key)?;
        let now = Instant::now();
        let mut counters = self.counters.lock().unwrap_or_else(PoisonError::into_inner);
        counters.retain(|_, (_, until)| *until > now);
        let counter = counters.entry(key).or_insert((0, now + window));
        counter.0 += 1;
        Ok(counter.0)
    }

    async fn try_lock(&self, namespace: &str, key: &str, lease: Duration) -> Result<Option<LockToken>> {
        let key = entry_key(namespace, key)?;
        let now = Instant::now();
        let mut locks = self.locks.lock().unwrap_or_else(PoisonError::into_inner);
        locks.retain(|_, (_, until)| *until > now);
        if locks.contains_key(&key) {
            return Ok(None);
        }
        let token = LockToken::generate();
        locks.insert(key, (token.as_str().to_owned(), now + lease));
        Ok(Some(token))
    }

    async fn unlock(&self, namespace: &str, key: &str, token: &LockToken) -> Result<bool> {
        let key = entry_key(namespace, key)?;
        let mut locks = self.locks.lock().unwrap_or_else(PoisonError::into_inner);
        let held = locks
            .get(&key)
            .is_some_and(|(holder, until)| holder == token.as_str() && *until > Instant::now());
        if held {
            locks.remove(&key);
        }
        Ok(held)
    }

    async fn check(&self) -> Result<()> {
        Ok(())
    }
}

/// The registration of this adapter (DEC-086): `memory://` gives an in-process cache, the local level of
/// every installation; the option `max_entries` bounds it (100 000 by default).
pub const ADAPTER: kampus_cache::Adapter = kampus_cache::Adapter {
    name: "moka",
    schemes: &["memory"],
    open: |settings| {
        Box::pin(async move {
            let max_entries = settings
                .option("max_entries")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000);
            Ok(std::sync::Arc::new(MokaCache::new(max_entries)) as std::sync::Arc<dyn Cache>)
        })
    },
};
