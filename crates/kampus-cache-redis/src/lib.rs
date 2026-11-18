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
// @package   kampus-cache-redis
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The shared cache adapter on Redis or Valkey: what every instance of an installation sees. Keys carry
//! the installation prefix (DEC-039), so several installations can share one server.

use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use kampus_cache::{Cache, LockToken, entry_key};
use kampus_common::{Error, Result};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

/// Releases a lock only when the caller's token still holds it, atomically on the server.
const UNLOCK: &str =
    r"if redis.call('get', KEYS[1]) == ARGV[1] then return redis.call('del', KEYS[1]) else return 0 end";

/// Adds one to a counter and starts its window on the first increment, atomically on the server.
const INCREMENT: &str = r"local n = redis.call('incr', KEYS[1]) if n == 1 then redis.call('pexpire', KEYS[1], ARGV[1]) end return n";

/// The shared cache.
#[derive(Clone)]
pub struct RedisCache {
    /// A connection that reconnects by itself.
    connection: ConnectionManager,
    /// The installation prefix of every key.
    prefix: String,
}

impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("prefix", &self.prefix)
            .finish_non_exhaustive()
    }
}

impl RedisCache {
    /// Connects to `url` (`redis://` or `valkey://`), prefixing every key with the installation prefix.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the server cannot be reached.
    pub async fn connect(url: &str, prefix: &str) -> Result<Self> {
        let url = url
            .strip_prefix("valkey://")
            .map_or_else(|| url.to_owned(), |rest| format!("redis://{rest}"));
        let client = redis::Client::open(url).map_err(unavailable)?;
        let connection = ConnectionManager::new(client).await.map_err(unavailable)?;
        Ok(Self {
            connection,
            prefix: prefix.to_owned(),
        })
    }

    /// The key on the server: prefix, namespace and key.
    fn key(&self, namespace: &str, key: &str) -> Result<String> {
        Ok(format!("{}{}", self.prefix, entry_key(namespace, key)?))
    }
}

#[async_trait]
impl Cache for RedisCache {
    fn shared(&self) -> bool {
        true
    }

    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Bytes>> {
        let key = self.key(namespace, key)?;
        let value: Option<Vec<u8>> = self.connection.clone().get(key).await.map_err(unavailable)?;
        Ok(value.map(Bytes::from))
    }

    async fn set(&self, namespace: &str, key: &str, value: Bytes, ttl: Duration) -> Result<()> {
        let key = self.key(namespace, key)?;
        let millis = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX).max(1);
        self.connection
            .clone()
            .pset_ex::<_, _, ()>(key, value.as_ref(), millis)
            .await
            .map_err(unavailable)
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        let key = self.key(namespace, key)?;
        self.connection
            .clone()
            .del::<_, ()>(key)
            .await
            .map_err(unavailable)
    }

    async fn increment(&self, namespace: &str, key: &str, window: Duration) -> Result<u64> {
        let key = self.key(namespace, key)?;
        let millis = u64::try_from(window.as_millis()).unwrap_or(u64::MAX).max(1);
        redis::Script::new(INCREMENT)
            .key(key)
            .arg(millis)
            .invoke_async(&mut self.connection.clone())
            .await
            .map_err(unavailable)
    }

    async fn try_lock(&self, namespace: &str, key: &str, lease: Duration) -> Result<Option<LockToken>> {
        let key = self.key(namespace, key)?;
        let token = LockToken::generate();
        let millis = u64::try_from(lease.as_millis()).unwrap_or(u64::MAX).max(1);
        let acquired: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg(token.as_str())
            .arg("NX")
            .arg("PX")
            .arg(millis)
            .query_async(&mut self.connection.clone())
            .await
            .map_err(unavailable)?;
        Ok(acquired.map(|_| token))
    }

    async fn unlock(&self, namespace: &str, key: &str, token: &LockToken) -> Result<bool> {
        let key = self.key(namespace, key)?;
        let released: i64 = redis::Script::new(UNLOCK)
            .key(key)
            .arg(token.as_str())
            .invoke_async(&mut self.connection.clone())
            .await
            .map_err(unavailable)?;
        Ok(released == 1)
    }

    async fn check(&self) -> Result<()> {
        redis::cmd("PING")
            .query_async::<String>(&mut self.connection.clone())
            .await
            .map(drop)
            .map_err(unavailable)
    }
}

/// Wraps a server failure as `unavailable`.
fn unavailable(error: redis::RedisError) -> Error {
    Error::unavailable("cache.unavailable").with_source(error)
}
