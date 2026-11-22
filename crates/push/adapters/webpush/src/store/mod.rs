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

//! A client of the push services of browsers: each message is encrypted for its device (RFC 8291), signed
//! with the VAPID key of the installation (RFC 8292) and posted to the endpoint of the device. Endpoints come
//! from browsers, so only public network addresses are reached, without following redirects.

use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use async_trait::async_trait;
use kampus_common::{AdapterSettings, Error, FieldError, Result};
use kampus_push::{Device, Outcome, Pusher, check_payload};
use p256::PublicKey;
use p256::ecdsa::SigningKey;

use crate::crypto;

/// How long talking to a push service may take.
const TIMEOUT: Duration = Duration::from_secs(20);

/// How long a push service keeps a message for a device that is offline.
const TIME_TO_LIVE: Duration = Duration::from_hours(12);

/// The longest endpoint, in characters.
const MAX_ENDPOINT: usize = 2000;

/// The push services of browsers, reached with the VAPID key of the installation.
pub struct WebPush {
    /// The VAPID key.
    key: SigningKey,
    /// Who push services contact about these messages, a `mailto:` or `https:` address.
    subject: String,
    /// Whether loopback and private addresses, and plain `http`, may be reached; only for tests.
    private_addresses: bool,
}

impl fmt::Debug for WebPush {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebPush")
            .field("subject", &self.subject)
            .finish_non_exhaustive()
    }
}

/// The error of settings that cannot open the adapter.
fn invalid(field: &str, rule: &str) -> Error {
    Error::validation("push.settings_invalid", vec![FieldError::new(field, rule)])
}

/// The error of a device that cannot be pushed to, or a message refused for good.
fn refused(rule: &str) -> Error {
    Error::validation("push.invalid", vec![FieldError::new("device", rule)])
}

/// The error of a push service that does not take a message now.
fn failed(what: &str) -> Error {
    Error::unavailable("push.failed").with_param("cause", what.to_owned())
}

/// Whether an address is on the public internet: not loopback, private, link-local, shared, documentation,
/// multicast or unspecified.
fn public(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_documentation()
                || a == 0
                || (a == 100 && (64..128).contains(&b)))
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            !(v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || v6.to_ipv4_mapped().is_some_and(|v4| !public(IpAddr::V4(v4))))
        }
    }
}

impl WebPush {
    /// Opens the adapter with the `vapid_private_key` option, the 32 bytes of a P-256 private key in base64url,
    /// and the `subject` option.
    ///
    /// # Errors
    ///
    /// `validation` (`push.settings_invalid`) without a key or a subject, or with one that cannot be used.
    pub fn open(settings: &AdapterSettings) -> Result<Self> {
        let key = settings
            .option("vapid_private_key")
            .ok_or_else(|| invalid("vapid_private_key", "required"))?;
        let subject = settings
            .option("subject")
            .ok_or_else(|| invalid("subject", "required"))?;
        Self::new(key, subject)
    }

    /// The adapter with a VAPID private key in base64url and a subject.
    ///
    /// # Errors
    ///
    /// `validation` (`push.settings_invalid`) for a key or a subject that cannot be used.
    pub fn new(private_key: &str, subject: &str) -> Result<Self> {
        let bytes = crypto::decode(private_key, "vapid_private_key")
            .map_err(|_| invalid("vapid_private_key", "base64url"))?;
        let key = SigningKey::from_slice(&bytes).map_err(|_| invalid("vapid_private_key", "p256"))?;
        if !(subject.starts_with("mailto:") || subject.starts_with("https://")) || subject.len() > 200 {
            return Err(invalid("subject", "mailto_or_https"));
        }
        Ok(Self {
            key,
            subject: subject.to_owned(),
            private_addresses: false,
        })
    }

    /// Lets the adapter reach loopback and private addresses over plain `http`, for a push service of a test.
    #[must_use]
    pub const fn allowing_private_addresses(mut self) -> Self {
        self.private_addresses = true;
        self
    }

    /// A new VAPID private key, in base64url.
    ///
    /// # Errors
    ///
    /// `unavailable` (`push.failed`) when the system gives no random bytes.
    pub fn generate_key() -> Result<String> {
        Ok(crypto::encode(&crypto::new_secret()?.to_bytes()))
    }

    /// The headers and the encrypted body of a message for a device at an origin.
    fn request(
        &self,
        device: &Device,
        payload: &[u8],
        origin: &str,
    ) -> Result<(reqwest::header::HeaderMap, Vec<u8>)> {
        let public = crypto::decode(&device.p256dh, "p256dh")
            .ok()
            .and_then(|bytes| PublicKey::from_sec1_bytes(&bytes).ok())
            .ok_or_else(|| refused("p256dh"))?;
        let auth = crypto::decode(&device.auth, "auth")
            .ok()
            .filter(|bytes| bytes.len() == 16)
            .ok_or_else(|| refused("auth"))?;
        let body = crypto::encrypt(payload, &public, &auth)?;
        let authorization = crypto::vapid(&self.key, &self.subject, origin, TIME_TO_LIVE)?;
        let mut headers = reqwest::header::HeaderMap::new();
        let mut put = |name: &'static str, value: String| -> Result<()> {
            headers.insert(name, value.parse().map_err(|_| refused("header"))?);
            Ok(())
        };
        put("authorization", authorization)?;
        put("ttl", TIME_TO_LIVE.as_secs().to_string())?;
        put("content-encoding", "aes128gcm".to_owned())?;
        put("content-type", "application/octet-stream".to_owned())?;
        put("urgency", "normal".to_owned())?;
        Ok((headers, body))
    }

    /// The address to reach the push service of an endpoint at, refused when it is not public.
    async fn address(&self, url: &reqwest::Url) -> Result<(String, SocketAddr)> {
        let secure = url.scheme() == "https" || (self.private_addresses && url.scheme() == "http");
        let host = url
            .host_str()
            .filter(|_| secure && url.username().is_empty() && url.as_str().len() <= MAX_ENDPOINT)
            .ok_or_else(|| refused("endpoint"))?
            .to_owned();
        let port = url.port_or_known_default().unwrap_or(443);
        let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|_| failed("resolve"))?
            .collect();
        let address = addresses.first().copied().ok_or_else(|| failed("resolve"))?;
        if !self.private_addresses && addresses.iter().any(|a| !public(a.ip())) {
            return Err(refused("public"));
        }
        Ok((host, address))
    }
}

#[async_trait]
impl Pusher for WebPush {
    async fn push(&self, device: &Device, payload: &[u8]) -> Result<Outcome> {
        check_payload(payload)?;
        let url = reqwest::Url::parse(&device.endpoint).map_err(|_| refused("endpoint"))?;
        let (host, address) = self.address(&url).await?;
        let (headers, body) = self.request(device, payload, &url.origin().ascii_serialization())?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(TIMEOUT)
            .resolve(&host, address)
            .build()
            .map_err(|_| failed("client"))?;
        let response = client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|_| failed("connect"))?;
        match response.status().as_u16() {
            200..=299 => Ok(Outcome::Delivered),
            404 | 410 => Ok(Outcome::Gone),
            400..=499 if response.status().as_u16() != 429 => Err(refused("refused")),
            status => Err(failed("status").with_param("status", status.to_string())),
        }
    }

    fn public_key(&self) -> String {
        crypto::encode(&crypto::public_bytes(&PublicKey::from(self.key.verifying_key())))
    }
}
