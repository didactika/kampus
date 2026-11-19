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

//! The contract of the cache port: lock tokens, the trait every adapter implements, and the keys of entries.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use kampus_common::{Error, FieldError, Result};

/// The longest key the port accepts.
const MAX_KEY: usize = 512;

/// Proves who holds a lock, so only the holder releases it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockToken(String);

impl LockToken {
    /// A new, unguessable token.
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }

    /// Wraps a token an adapter read back.
    #[must_use]
    pub const fn from_string(token: String) -> Self {
        Self(token)
    }

    /// The token as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The cache port.
#[async_trait]
pub trait Cache: Send + Sync + fmt::Debug {
    /// Whether other instances see what this cache holds; without a shared cache only one instance may run.
    fn shared(&self) -> bool;

    /// Reads an entry of a namespace.
    ///
    /// # Errors
    ///
    /// A `validation` error for an invalid namespace or key, an `unavailable` error when the cache fails.
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Bytes>>;

    /// Writes an entry that expires after `ttl`.
    ///
    /// # Errors
    ///
    /// As [`Cache::get`].
    async fn set(&self, namespace: &str, key: &str, value: Bytes, ttl: Duration) -> Result<()>;

    /// Removes an entry; removing a missing entry succeeds.
    ///
    /// # Errors
    ///
    /// As [`Cache::get`].
    async fn delete(&self, namespace: &str, key: &str) -> Result<()>;

    /// Adds one to a counter and returns its value; the counter starts again after `window` from its first
    /// increment.
    ///
    /// # Errors
    ///
    /// As [`Cache::get`].
    async fn increment(&self, namespace: &str, key: &str, window: Duration) -> Result<u64>;

    /// Takes a lock for `lease`, or returns `None` when someone else holds it.
    ///
    /// # Errors
    ///
    /// As [`Cache::get`].
    async fn try_lock(&self, namespace: &str, key: &str, lease: Duration) -> Result<Option<LockToken>>;

    /// Releases a lock when `token` still holds it, and tells whether it did.
    ///
    /// # Errors
    ///
    /// As [`Cache::get`].
    async fn unlock(&self, namespace: &str, key: &str, token: &LockToken) -> Result<bool>;

    /// Checks that the cache answers, for readiness.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when it does not.
    async fn check(&self) -> Result<()>;
}

/// The full key of an entry: the namespace a module declared, then the key.
///
/// # Errors
///
/// A `validation` error when the namespace is not lower-case letters, digits, `.`, `_` or `-`, or the key
/// is empty or too long.
pub fn entry_key(namespace: &str, key: &str) -> Result<String> {
    let mut fields = Vec::new();
    let valid_namespace = !namespace.is_empty()
        && namespace
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b));
    if !valid_namespace {
        fields.push(FieldError::new("namespace", "format"));
    }
    if key.is_empty() || key.len() > MAX_KEY {
        fields.push(FieldError::new("key", "length"));
    }
    if fields.is_empty() {
        Ok(format!("{namespace}:{key}"))
    } else {
        Err(Error::validation("cache.key_invalid", fields))
    }
}
