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

//! The push port: a short message for one device, encrypted for it and handed to its push service.

use std::fmt;

use async_trait::async_trait;
use kampus_common::{Error, FieldError, Result};

/// The largest message, in bytes, that every push service takes once encrypted.
pub const MAX_PAYLOAD: usize = 3000;

/// A browser or phone that allowed push, as its push service described it when it subscribed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// Where its push service takes messages for it, an `https` address.
    pub endpoint: String,
    /// Its public key, P-256 in base64url.
    pub p256dh: String,
    /// Its authentication secret, 16 bytes in base64url.
    pub auth: String,
}

/// What became of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The push service took it.
    Delivered,
    /// The device is gone, and its subscription should be removed.
    Gone,
}

/// Checks the size of a message before it is encrypted.
///
/// # Errors
///
/// `validation` (`push.too_large`) above [`MAX_PAYLOAD`] bytes.
pub fn check_payload(payload: &[u8]) -> Result<()> {
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::validation(
            "push.too_large",
            vec![FieldError::new("payload", "max_length")],
        ));
    }
    Ok(())
}

/// The push port.
#[async_trait]
pub trait Pusher: Send + Sync + fmt::Debug {
    /// Sends one message to one device.
    ///
    /// # Errors
    ///
    /// `validation` (`push.invalid`) for a device whose keys or address cannot be used, or a message the
    /// push service refuses for good, and (`push.too_large`) above [`MAX_PAYLOAD`]; `unavailable`
    /// (`push.failed`) when it cannot be sent now.
    async fn push(&self, device: &Device, payload: &[u8]) -> Result<Outcome>;

    /// The public key browsers subscribe with (`applicationServerKey`), P-256 in base64url.
    fn public_key(&self) -> String;
}
