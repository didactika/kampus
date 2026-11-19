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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Secrets: random tokens from the operating system, their hashes, and Argon2id password hashes.
//! Tokens are stored only as SHA-256 hashes; they carry enough entropy for a fast hash.

use std::sync::{Arc, OnceLock};

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use chacha20poly1305::aead::{Aead, Key, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use kampus_common::{Error, Result};
use sha2::{Digest, Sha256};

/// Random bytes from the operating system.
///
/// # Errors
///
/// An `internal` error when the operating system gives none.
pub fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes).map_err(|error| {
        Error::internal("identity.random_unavailable").with_param("cause", error.to_string())
    })?;
    Ok(bytes)
}

/// Random bytes as lower-case hexadecimal, for short names such as token prefixes.
///
/// # Errors
///
/// As [`random_bytes`].
pub fn random_hex<const N: usize>() -> Result<String> {
    /// The hexadecimal digits.
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(N * 2);
    for byte in random_bytes::<N>()? {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    Ok(text)
}

/// A new token of 256 bits, URL-safe.
///
/// # Errors
///
/// As [`random_bytes`].
pub fn new_token() -> Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes::<32>()?))
}

/// The SHA-256 hash a token is stored as.
#[must_use]
pub fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// Compares two byte strings in time that depends only on their length.
#[must_use]
pub fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// The cost of Argon2id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordCost {
    /// Memory in KiB.
    pub memory_kib: u32,
    /// Iterations.
    pub iterations: u32,
    /// Lanes.
    pub parallelism: u32,
}

impl Default for PasswordCost {
    /// The minimum OWASP recommends for Argon2id: 19 MiB, two iterations, one lane.
    fn default() -> Self {
        Self {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 1,
        }
    }
}

/// The length of a password hash, in bytes.
const HASH_LENGTH: usize = 32;

/// Hashes and verifies passwords with Argon2id, off the async threads.
#[derive(Debug, Clone)]
pub struct PasswordHasher {
    /// The cost of new hashes.
    cost: PasswordCost,
    /// A hash of nothing, verified when the account does not exist so the answer takes the same time.
    decoy: Arc<OnceLock<String>>,
}

impl PasswordHasher {
    /// A hasher with this cost for new hashes; existing hashes are verified with their own.
    #[must_use]
    pub fn new(cost: PasswordCost) -> Self {
        Self {
            cost,
            decoy: Arc::new(OnceLock::new()),
        }
    }

    /// The parameters of new hashes, as stored next to them.
    #[must_use]
    pub fn params(&self) -> String {
        format!(
            "m={},t={},p={}",
            self.cost.memory_kib, self.cost.iterations, self.cost.parallelism
        )
    }

    /// Hashes a password into a PHC string.
    ///
    /// # Errors
    ///
    /// An `internal` error when hashing fails.
    pub async fn hash(&self, password: &str) -> Result<String> {
        let cost = self.cost;
        let password = password.to_owned();
        let salt = random_bytes::<16>()?;
        blocking(move || phc(cost, &password, &salt)).await
    }

    /// Whether a password matches a PHC string; `None` verifies against a decoy and answers `false`.
    ///
    /// # Errors
    ///
    /// An `internal` error when the stored hash cannot be read.
    pub async fn verify(&self, password: &str, stored: Option<&str>) -> Result<bool> {
        let (stored, real) = match stored {
            Some(stored) => (stored.to_owned(), true),
            None => (self.decoy().await?, false),
        };
        let password = password.to_owned();
        let matches = blocking(move || verify_phc(&password, &stored)).await?;
        Ok(real && matches)
    }

    /// The decoy hash, made once.
    async fn decoy(&self) -> Result<String> {
        if let Some(decoy) = self.decoy.get() {
            return Ok(decoy.clone());
        }
        let decoy = self.hash("kampus decoy password").await?;
        Ok(self.decoy.get_or_init(|| decoy).clone())
    }
}

/// Runs CPU-bound work on the blocking pool.
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| Error::internal("identity.hash_failed").with_source(error))?
}

/// The raw Argon2id hash of a password.
fn raw(cost: PasswordCost, password: &str, salt: &[u8]) -> Result<[u8; HASH_LENGTH]> {
    let params = Params::new(
        cost.memory_kib,
        cost.iterations,
        cost.parallelism,
        Some(HASH_LENGTH),
    )
    .map_err(|error| Error::internal("identity.hash_failed").with_param("cause", error.to_string()))?;
    let mut out = [0_u8; HASH_LENGTH];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|error| Error::internal("identity.hash_failed").with_param("cause", error.to_string()))?;
    Ok(out)
}

/// The PHC string of a password: `$argon2id$v=19$m=…,t=…,p=…$salt$hash`.
fn phc(cost: PasswordCost, password: &str, salt: &[u8]) -> Result<String> {
    let hash = raw(cost, password, salt)?;
    Ok(format!(
        "$argon2id$v=19$m={},t={},p={}${}${}",
        cost.memory_kib,
        cost.iterations,
        cost.parallelism,
        STANDARD_NO_PAD.encode(salt),
        STANDARD_NO_PAD.encode(hash)
    ))
}

/// Whether a password matches a PHC string made by [`phc`].
fn verify_phc(password: &str, stored: &str) -> Result<bool> {
    let unreadable = || Error::internal("identity.password_hash_unreadable");
    let parts: Vec<&str> = stored.split('$').collect();
    let [_, "argon2id", "v=19", params, salt, hash] = parts.as_slice() else {
        return Err(unreadable());
    };
    let mut cost = PasswordCost::default();
    for pair in params.split(',') {
        let (name, value) = pair.split_once('=').ok_or_else(unreadable)?;
        let value: u32 = value.parse().map_err(|_| unreadable())?;
        match name {
            "m" => cost.memory_kib = value,
            "t" => cost.iterations = value,
            "p" => cost.parallelism = value,
            _ => return Err(unreadable()),
        }
    }
    let salt = STANDARD_NO_PAD.decode(salt).map_err(|_| unreadable())?;
    let expected = STANDARD_NO_PAD.decode(hash).map_err(|_| unreadable())?;
    Ok(same(&raw(cost, password, &salt)?, &expected))
}

/// The version byte of sealed secrets, so the key or the cipher can change later.
const SEALED_V1: u8 = 1;

/// The key that seals secrets identity must read back, such as TOTP secrets and provider secrets.
#[derive(Clone)]
pub struct SecretKey([u8; 32]);

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretKey(..)")
    }
}

impl SecretKey {
    /// A key from 32 bytes in base64, as the configuration gives it.
    ///
    /// # Errors
    ///
    /// `validation` on `secret_key` when it is not 32 bytes of base64.
    pub fn from_base64(text: &str) -> Result<Self> {
        STANDARD_NO_PAD
            .decode(text.trim().trim_end_matches('='))
            .ok()
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .map(Self)
            .ok_or_else(|| {
                Error::validation(
                    "identity.secret_key_invalid",
                    vec![kampus_common::FieldError::new("secret_key", "base64_32_bytes")],
                )
            })
    }

    /// A new random key.
    ///
    /// # Errors
    ///
    /// As [`random_bytes`].
    pub fn generate() -> Result<Self> {
        random_bytes::<32>().map(Self)
    }

    /// The key in base64, to put in the configuration.
    #[must_use]
    pub fn to_base64(&self) -> String {
        STANDARD_NO_PAD.encode(self.0)
    }

    /// Seals a secret: a version byte, a random nonce and the XChaCha20-Poly1305 ciphertext.
    ///
    /// # Errors
    ///
    /// An `internal` error when sealing fails.
    pub fn seal(&self, plain: &[u8]) -> Result<Vec<u8>> {
        let nonce = random_bytes::<24>()?;
        let sealed = XChaCha20Poly1305::new(&Key::<XChaCha20Poly1305>::from(self.0))
            .encrypt(&XNonce::from(nonce), plain)
            .map_err(|_| Error::internal("identity.seal_failed"))?;
        let mut out = Vec::with_capacity(1 + nonce.len() + sealed.len());
        out.push(SEALED_V1);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&sealed);
        Ok(out)
    }

    /// Opens a sealed secret.
    ///
    /// # Errors
    ///
    /// An `internal` error when the secret was sealed with another key or changed.
    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>> {
        let unreadable = || Error::internal("identity.sealed_secret_unreadable");
        let (&version, rest) = sealed.split_first().ok_or_else(unreadable)?;
        if version != SEALED_V1 || rest.len() < 24 {
            return Err(unreadable());
        }
        let (nonce, ciphertext) = rest.split_at(24);
        let nonce = <[u8; 24]>::try_from(nonce).map_err(|_| unreadable())?;
        XChaCha20Poly1305::new(&Key::<XChaCha20Poly1305>::from(self.0))
            .decrypt(&XNonce::from(nonce), ciphertext)
            .map_err(|_| unreadable())
    }
}
