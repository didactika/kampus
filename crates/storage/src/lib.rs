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
// @package   kampus-storage
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The file storage port of Kampus (DEC-009, infrastructure ports): blobs addressed by keys that
//! `kampus-files` chooses (content hashes, DEC-053), written only when complete and verified by their hash.
//!
//! Adapters implement [`Storage`]; every adapter passes the suite of [`conformance`] (DEC-066). Callers
//! ask for [`Capabilities`], never for an adapter.

use std::fmt;
use std::ops::Range;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use kampus_common::{Error, FieldError, Result};
use sha2::{Digest, Sha256};

#[cfg(feature = "conformance")]
pub mod conformance;

/// The longest key the port accepts.
const MAX_KEY: usize = 512;

/// What an adapter returns while it opens.
pub type Opening =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<std::sync::Arc<dyn Storage>>> + Send>>;

/// A storage adapter as it registers itself in a build (DEC-086): the URL schemes it serves and how it opens.
#[derive(Debug, Clone, Copy)]
pub struct Adapter {
    /// The name of the adapter, for logs and errors.
    pub name: &'static str,
    /// The URL schemes it serves (`file`, `s3`).
    pub schemes: &'static [&'static str],
    /// Opens the adapter with its settings.
    pub open: fn(kampus_common::AdapterSettings) -> Opening,
}

/// Opens the adapter, among those of the build, that serves the scheme of the settings' URL.
///
/// # Errors
///
/// A `validation` error when no adapter serves the scheme, or the adapter's own error when it cannot open.
pub async fn open(
    adapters: &[Adapter],
    settings: kampus_common::AdapterSettings,
) -> Result<std::sync::Arc<dyn Storage>> {
    let adapter = kampus_common::adapter::choose(adapters, |a| a.schemes, &settings, "storage")?;
    (adapter.open)(settings).await
}

/// The key of a blob: segments of lower-case letters, digits, `.`, `_` and `-`, separated by `/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageKey(String);

impl StorageKey {
    /// Validates a key.
    ///
    /// # Errors
    ///
    /// A `validation` error when the key is empty, too long, or has an invalid segment.
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let segment = |s: &str| {
            !s.is_empty()
                && s != "."
                && s != ".."
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        };
        if key.len() <= MAX_KEY && key.split('/').all(segment) {
            Ok(Self(key))
        } else {
            Err(
                Error::validation("storage.key_invalid", vec![FieldError::new("key", "format")])
                    .with_param("key", key),
            )
        }
    }

    /// The key of a blob with this content hash, sharded by its first bytes: `ab/cd/abcd…`.
    #[must_use]
    pub fn for_hash(hash: &[u8; 32]) -> Self {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let hex: String = hash
            .iter()
            .flat_map(|byte| {
                [
                    char::from(DIGITS[usize::from(byte >> 4)]),
                    char::from(DIGITS[usize::from(byte & 0x0f)]),
                ]
            })
            .collect();
        Self(format!("{}/{}/{hex}", &hex[..2], &hex[2..4]))
    }

    /// The key as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StorageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What the store knows about a blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectInfo {
    /// The size in bytes.
    pub size: u64,
}

/// The optional features of an adapter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// The store can sign URLs that read a blob directly, for a limited time.
    pub presign_read: bool,
}

/// The file storage port.
#[async_trait]
pub trait Storage: Send + Sync + fmt::Debug {
    /// The optional features of this adapter.
    fn capabilities(&self) -> Capabilities;

    /// Stores a blob, only when its content has the expected SHA-256; it becomes visible once complete.
    ///
    /// # Errors
    ///
    /// A `validation` error when the hash does not match, an `unavailable` error when the store fails.
    async fn put(&self, key: &StorageKey, data: Bytes, sha256: [u8; 32]) -> Result<()>;

    /// Reads a blob.
    ///
    /// # Errors
    ///
    /// A `not_found` error when there is no blob, an `unavailable` error when the store fails.
    async fn get(&self, key: &StorageKey) -> Result<Bytes>;

    /// Reads a range of bytes of a blob.
    ///
    /// # Errors
    ///
    /// As [`Storage::get`].
    async fn get_range(&self, key: &StorageKey, range: Range<u64>) -> Result<Bytes>;

    /// Tells whether a blob exists, and its size.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the store fails.
    async fn head(&self, key: &StorageKey) -> Result<Option<ObjectInfo>>;

    /// Deletes a blob; deleting a missing blob succeeds.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the store fails.
    async fn delete(&self, key: &StorageKey) -> Result<()>;

    /// Copies a blob to another key.
    ///
    /// # Errors
    ///
    /// A `not_found` error when the source is missing, an `unavailable` error when the store fails.
    async fn copy(&self, from: &StorageKey, to: &StorageKey) -> Result<()>;

    /// Signs a URL that reads a blob for a limited time, when the adapter can.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when the store fails.
    async fn presign_read(&self, key: &StorageKey, ttl: Duration) -> Result<Option<String>>;

    /// Checks that the store answers, for readiness.
    ///
    /// # Errors
    ///
    /// An `unavailable` error when it does not.
    async fn check(&self) -> Result<()>;
}

/// The SHA-256 of some content, as the port expects it.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Checks content against its expected hash before an adapter stores it.
///
/// # Errors
///
/// A `validation` error when the hash does not match.
pub fn verify(key: &StorageKey, data: &[u8], expected: &[u8; 32]) -> Result<()> {
    if &sha256(data) == expected {
        Ok(())
    } else {
        Err(Error::validation(
            "storage.hash_mismatch",
            vec![FieldError::new("content", "sha256")],
        )
        .with_param("key", key.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::{StorageKey, sha256, verify};

    #[test]
    fn keys_are_validated() {
        for key in ["ab/cd/abcd", "files/report-2026_01.pdf", "a"] {
            assert!(StorageKey::new(key).is_ok(), "{key}");
        }
        for key in ["", "/a", "a//b", "A/b", "a/../b", "a b", "ñ"] {
            assert_eq!(
                StorageKey::new(key).unwrap_err().code(),
                "storage.key_invalid",
                "{key:?}"
            );
        }
    }

    #[test]
    fn hash_keys_are_sharded() {
        let key = StorageKey::for_hash(&sha256(b"hello"));
        assert!(key.as_str().starts_with("2c/f2/2cf24dba5fb0a30e"));
        assert!(StorageKey::new(key.as_str()).is_ok());
    }

    #[test]
    fn content_must_match_its_hash() {
        let key = StorageKey::new("a").unwrap();
        assert!(verify(&key, b"hello", &sha256(b"hello")).is_ok());
        assert_eq!(
            verify(&key, b"hello", &sha256(b"other")).unwrap_err().code(),
            "storage.hash_mismatch"
        );
    }
}
