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
// @package   kampus-push-webpush
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The cryptography of Web Push: a message encrypted for one device with `aes128gcm` (RFC 8188, RFC 8291),
//! and the VAPID signature that tells its push service who sends it (RFC 8292).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes128Gcm, Nonce};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hkdf::Hkdf;
use kampus_common::{Error, FieldError, Result};
use p256::ecdsa::signature::Signer as _;
use p256::ecdsa::{Signature, SigningKey};
use p256::{PublicKey, SecretKey};
use sha2::Sha256;

/// The size of the records of a message; one record holds every message the port allows.
const RECORD_SIZE: u32 = 4096;

/// The error of a device that cannot be encrypted for.
fn refused(rule: &str) -> Error {
    Error::validation("push.invalid", vec![FieldError::new("device", rule)])
}

/// Decodes base64url, with or without padding.
///
/// # Errors
///
/// `validation` (`push.invalid`) naming the field for text that is not base64url.
pub fn decode(text: &str, field: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(text.trim_end_matches('='))
        .map_err(|_| refused(field))
}

/// Encodes base64url without padding.
#[must_use]
pub fn encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Random bytes from the operating system.
///
/// # Errors
///
/// `unavailable` (`push.failed`) when the system gives none.
pub fn random<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|_| Error::unavailable("push.failed").with_param("cause", "random"))?;
    Ok(bytes)
}

/// A new P-256 private key.
///
/// # Errors
///
/// `unavailable` (`push.failed`) when the system gives no random bytes.
pub fn new_secret() -> Result<SecretKey> {
    loop {
        if let Ok(secret) = SecretKey::from_slice(&random::<32>()?) {
            return Ok(secret);
        }
    }
}

/// The uncompressed SEC1 bytes of a public key, 65 of them.
#[must_use]
pub fn public_bytes(public: &PublicKey) -> Vec<u8> {
    public.to_sec1_bytes().to_vec()
}

/// HKDF with SHA-256: `length` bytes from a salt, key material and info.
fn hkdf(salt: &[u8], material: &[u8], info: &[u8], out: &mut [u8]) -> Result<()> {
    Hkdf::<Sha256>::new(Some(salt), material)
        .expand(info, out)
        .map_err(|_| refused("encryption"))
}

/// Encrypts a message for a device with a chosen salt and sender key; [`encrypt`] chooses both at random.
///
/// # Errors
///
/// `validation` (`push.invalid`) when the device keys cannot be used.
pub fn encrypt_with(
    message: &[u8],
    device_public: &PublicKey,
    device_auth: &[u8],
    salt: [u8; 16],
    sender: &SecretKey,
) -> Result<Vec<u8>> {
    let sender_public = public_bytes(&sender.public_key());
    let device_bytes = public_bytes(device_public);
    let shared = p256::ecdh::diffie_hellman(sender.to_nonzero_scalar(), device_public.as_affine());
    let mut info = b"WebPush: info\0".to_vec();
    info.extend_from_slice(&device_bytes);
    info.extend_from_slice(&sender_public);
    let mut material = [0_u8; 32];
    hkdf(device_auth, shared.raw_secret_bytes(), &info, &mut material)?;
    let mut key = [0_u8; 16];
    hkdf(&salt, &material, b"Content-Encoding: aes128gcm\0", &mut key)?;
    let mut nonce = [0_u8; 12];
    hkdf(&salt, &material, b"Content-Encoding: nonce\0", &mut nonce)?;
    let mut record = message.to_vec();
    record.push(2);
    let sealed = Aes128Gcm::new_from_slice(&key)
        .map_err(|_| refused("encryption"))?
        .encrypt(&Nonce::from(nonce), record.as_slice())
        .map_err(|_| refused("encryption"))?;
    let mut out = Vec::with_capacity(21 + sender_public.len() + sealed.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&RECORD_SIZE.to_be_bytes());
    out.push(u8::try_from(sender_public.len()).map_err(|_| refused("encryption"))?);
    out.extend_from_slice(&sender_public);
    out.extend_from_slice(&sealed);
    Ok(out)
}

/// Encrypts a message for a device, with a new salt and sender key each time.
///
/// # Errors
///
/// `validation` (`push.invalid`) when the device keys cannot be used.
pub fn encrypt(message: &[u8], device_public: &PublicKey, device_auth: &[u8]) -> Result<Vec<u8>> {
    encrypt_with(
        message,
        device_public,
        device_auth,
        random::<16>()?,
        &new_secret()?,
    )
}

/// The VAPID `Authorization` header of a request to the push service at an origin, valid for a while.
///
/// # Errors
///
/// `unavailable` (`push.failed`) when the clock is before 1970.
pub fn vapid(key: &SigningKey, subject: &str, origin: &str, valid: Duration) -> Result<String> {
    let expires = SystemTime::now()
        .checked_add(valid)
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .ok_or_else(|| Error::unavailable("push.failed").with_param("cause", "clock"))?
        .as_secs();
    let header = encode(br#"{"typ":"JWT","alg":"ES256"}"#);
    let claims = encode(
        serde_json::json!({ "aud": origin, "exp": expires, "sub": subject })
            .to_string()
            .as_bytes(),
    );
    let signed = format!("{header}.{claims}");
    let signature: Signature = key.sign(signed.as_bytes());
    let public = encode(&public_bytes(&PublicKey::from(key.verifying_key())));
    Ok(format!(
        "vapid t={signed}.{}, k={public}",
        encode(&signature.to_bytes())
    ))
}
