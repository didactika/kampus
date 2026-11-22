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
// @package   kampus-push
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The conformance suite of the push port: an adapter is supported only when it passes every check here,
//! run against a push service that decrypts what it receives as the device would.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "a conformance failure must stop the test"
)]

use async_trait::async_trait;

use crate::{Device, MAX_PAYLOAD, Outcome, Pusher};

/// A push service for the suite, with devices in each state it answers for.
#[async_trait]
pub trait PushService: Send + Sync {
    /// A device the service delivers to; it holds the private key of the device.
    fn device(&self) -> Device;
    /// A device the service says is gone.
    fn gone(&self) -> Device;
    /// A device whose messages the service refuses for good.
    fn refusing(&self) -> Device;
    /// A device whose messages the service cannot take now.
    fn busy(&self) -> Device;
    /// The messages delivered so far, decrypted, oldest first.
    async fn received(&self) -> Vec<Vec<u8>>;
}

/// Runs every check.
pub async fn run(pusher: &dyn Pusher, service: &dyn PushService) {
    let key = pusher.public_key();
    assert_eq!(
        key.len(),
        87,
        "an uncompressed P-256 key in base64url without padding"
    );
    assert!(
        key.starts_with('B')
            && key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "{key}"
    );

    let before = service.received().await.len();
    let message = br#"{"title":"New grade","body":"Quiz"}"#;
    assert_eq!(
        pusher.push(&service.device(), message).await.unwrap(),
        Outcome::Delivered
    );
    let largest = vec![b'a'; MAX_PAYLOAD];
    assert_eq!(
        pusher.push(&service.device(), &largest).await.unwrap(),
        Outcome::Delivered,
        "the largest message is delivered"
    );
    let received = service.received().await;
    assert_eq!(received.len() - before, 2, "each message arrives once");
    assert_eq!(received[before], message, "the device reads what was sent");
    assert_eq!(received[before + 1], largest);

    assert_eq!(
        pusher.push(&service.gone(), message).await.unwrap(),
        Outcome::Gone
    );
    let too_large = vec![b'a'; MAX_PAYLOAD + 1];
    assert_eq!(
        pusher
            .push(&service.device(), &too_large)
            .await
            .unwrap_err()
            .code(),
        "push.too_large"
    );
    assert_eq!(
        pusher
            .push(&service.refusing(), message)
            .await
            .unwrap_err()
            .code(),
        "push.invalid"
    );
    assert_eq!(
        pusher.push(&service.busy(), message).await.unwrap_err().code(),
        "push.failed"
    );
    let broken = Device {
        p256dh: "not a key".to_owned(),
        ..service.device()
    };
    assert_eq!(
        pusher.push(&broken, message).await.unwrap_err().code(),
        "push.invalid"
    );
    assert_eq!(service.received().await.len() - before, 2, "nothing else arrives");
}
