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

//! The push conformance suite against a push service that checks the VAPID signature, decrypts what it
//! receives with the keys of its devices and answers for each state of a device.

use std::sync::{Arc, Mutex};

use aes_gcm::aead::{Aead, KeyInit};
use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kampus_common::AdapterSettings;
use kampus_push::conformance::{self, PushService};
use kampus_push::{Device, Pusher};
use kampus_push_webpush::WebPush;
use p256::ecdsa::signature::Verifier as _;
use p256::ecdsa::{Signature, VerifyingKey};
use p256::{PublicKey, SecretKey};
use sha2::Sha256;

/// The private key of the devices of the service.
const DEVICE_KEY: [u8; 32] = [7; 32];

/// The authentication secret of the devices of the service.
const DEVICE_AUTH: [u8; 16] = [9; 16];

/// What the service keeps.
#[derive(Clone)]
struct Service {
    /// Where it listens.
    base: String,
    /// What it received, decrypted.
    received: Arc<Mutex<Vec<Vec<u8>>>>,
    /// The public key it accepts signatures of.
    vapid: Arc<Mutex<Option<String>>>,
}

/// HKDF with SHA-256.
fn hkdf<const N: usize>(salt: &[u8], material: &[u8], info: &[u8]) -> [u8; N] {
    let mut out = [0_u8; N];
    hkdf::Hkdf::<Sha256>::new(Some(salt), material)
        .expand(info, &mut out)
        .unwrap();
    out
}

/// Decrypts what a device receives, as the device would (RFC 8188, RFC 8291).
fn decrypt(body: &[u8]) -> Vec<u8> {
    let (salt, rest) = body.split_at(16);
    let record_size = u32::from_be_bytes(rest[..4].try_into().unwrap());
    assert!(record_size as usize >= body.len() - 86, "one record");
    let id_length = usize::from(rest[4]);
    let (sender, sealed) = rest[5..].split_at(id_length);
    let sender = PublicKey::from_sec1_bytes(sender).unwrap();
    let device = SecretKey::from_slice(&DEVICE_KEY).unwrap();
    let shared = p256::ecdh::diffie_hellman(device.to_nonzero_scalar(), sender.as_affine());
    let mut info = b"WebPush: info\0".to_vec();
    info.extend_from_slice(&device.public_key().to_sec1_bytes());
    info.extend_from_slice(&sender.to_sec1_bytes());
    let material: [u8; 32] = hkdf(&DEVICE_AUTH, shared.raw_secret_bytes(), &info);
    let key: [u8; 16] = hkdf(salt, &material, b"Content-Encoding: aes128gcm\0");
    let nonce: [u8; 12] = hkdf(salt, &material, b"Content-Encoding: nonce\0");
    let mut plain = aes_gcm::Aes128Gcm::new_from_slice(&key)
        .unwrap()
        .decrypt(&aes_gcm::Nonce::from(nonce), sealed)
        .expect("encrypted for the device");
    while plain.last() == Some(&0) {
        plain.pop();
    }
    assert_eq!(plain.pop(), Some(2), "the delimiter of the last record");
    plain
}

/// Checks a VAPID token: signed by the key, for the origin, from the subject, not expired.
fn check_vapid(token: &str, key: &str, origin: &str) {
    let (signed, signature) = token.rsplit_once('.').unwrap();
    let public = VerifyingKey::from_sec1_bytes(&URL_SAFE_NO_PAD.decode(key).unwrap()).unwrap();
    let signature = Signature::from_slice(&URL_SAFE_NO_PAD.decode(signature).unwrap()).unwrap();
    public
        .verify(signed.as_bytes(), &signature)
        .expect("a valid signature");
    let (header, claims) = signed.split_once('.').unwrap();
    let header: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(header).unwrap()).unwrap();
    assert_eq!(header["alg"], "ES256");
    let claims: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(claims).unwrap()).unwrap();
    assert_eq!(claims["aud"], origin, "the origin of the endpoint");
    assert_eq!(claims["sub"], "mailto:push@kampus.test");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expires = claims["exp"].as_u64().unwrap();
    assert!(
        expires > now && expires <= now + 24 * 60 * 60,
        "expires within a day"
    );
}

/// Takes one message, as a push service would.
async fn take(
    State(service): State<Service>,
    Path(state): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    match state.as_str() {
        "gone" => return StatusCode::GONE,
        "refusing" => return StatusCode::BAD_REQUEST,
        "busy" => return StatusCode::TOO_MANY_REQUESTS,
        _ => {}
    }
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned()
    };
    assert_eq!(header("content-encoding"), "aes128gcm");
    assert!(!header("ttl").is_empty(), "a time to live");
    let authorization = header("authorization");
    let (token, key) = authorization
        .strip_prefix("vapid t=")
        .and_then(|rest| rest.split_once(", k="))
        .expect("a VAPID authorization");
    let expected = service.vapid.lock().unwrap().clone().unwrap();
    assert_eq!(key, expected, "signed with the key browsers subscribed with");
    check_vapid(token, key, &service.base);
    let plain = decrypt(&body);
    service.received.lock().unwrap().push(plain);
    StatusCode::CREATED
}

impl Service {
    /// A service on a free local port.
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let service = Self {
            base,
            received: Arc::default(),
            vapid: Arc::default(),
        };
        let app = axum::Router::new()
            .route("/push/{state}", axum::routing::post(take))
            .with_state(service.clone());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        service
    }

    /// A device of the service in a state.
    fn at(&self, state: &str) -> Device {
        let public = SecretKey::from_slice(&DEVICE_KEY).unwrap().public_key();
        Device {
            endpoint: format!("{}/push/{state}", self.base),
            p256dh: URL_SAFE_NO_PAD.encode(public.to_sec1_bytes()),
            auth: URL_SAFE_NO_PAD.encode(DEVICE_AUTH),
        }
    }
}

#[async_trait]
impl PushService for Service {
    fn device(&self) -> Device {
        self.at("ok")
    }

    fn gone(&self) -> Device {
        self.at("gone")
    }

    fn refusing(&self) -> Device {
        self.at("refusing")
    }

    fn busy(&self) -> Device {
        self.at("busy")
    }

    async fn received(&self) -> Vec<Vec<u8>> {
        self.received.lock().unwrap().clone()
    }
}

/// The adapter with a new key, allowed to reach the local service.
fn pusher(service: &Service) -> WebPush {
    let pusher = WebPush::new(&WebPush::generate_key().unwrap(), "mailto:push@kampus.test")
        .unwrap()
        .allowing_private_addresses();
    *service.vapid.lock().unwrap() = Some(pusher.public_key());
    pusher
}

#[tokio::test]
async fn a_push_service_that_checks_and_decrypts_conforms() {
    let service = Service::start().await;
    conformance::run(&pusher(&service), &service).await;
}

#[tokio::test]
async fn private_addresses_and_plain_http_are_never_reached() {
    let service = Service::start().await;
    let pusher = WebPush::new(&WebPush::generate_key().unwrap(), "mailto:push@kampus.test").unwrap();
    let local = pusher.push(&service.device(), b"hi").await.unwrap_err();
    assert_eq!(local.code(), "push.invalid", "plain http");
    let private = Device {
        endpoint: service.device().endpoint.replace("http://", "https://"),
        ..service.device()
    };
    assert_eq!(
        pusher.push(&private, b"hi").await.unwrap_err().code(),
        "push.invalid"
    );
    assert_eq!(service.received().await.len(), 0, "nothing reached the service");
}

#[tokio::test]
async fn the_settings_need_a_key_and_a_subject() {
    let open = |options: &[(&str, &str)]| {
        kampus_push::open(
            &[kampus_push_webpush::ADAPTER],
            AdapterSettings {
                url: "webpush://".to_owned(),
                options: options
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect(),
                ..AdapterSettings::default()
            },
        )
    };
    let key = WebPush::generate_key().unwrap();
    let opened = open(&[
        ("vapid_private_key", &key),
        ("subject", "mailto:push@kampus.test"),
    ])
    .await
    .unwrap();
    assert_eq!(opened.public_key().len(), 87);
    for options in [
        vec![("subject", "mailto:push@kampus.test")],
        vec![("vapid_private_key", key.as_str())],
        vec![
            ("vapid_private_key", "short"),
            ("subject", "mailto:push@kampus.test"),
        ],
        vec![
            ("vapid_private_key", key.as_str()),
            ("subject", "push@kampus.test"),
        ],
    ] {
        assert_eq!(
            open(&options).await.unwrap_err().code(),
            "push.settings_invalid",
            "{options:?}"
        );
    }
}
