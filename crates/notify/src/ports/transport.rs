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
// @package   kampus-notify
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The transport of a channel: it carries one rendered notification to the addresses of its recipient, an
//! email address or the push endpoints of their devices. Delivery jobs call it after the change that caused
//! the notification is committed, and retry it when it fails.

use std::sync::Mutex;

use async_trait::async_trait;
use kampus_common::Result;

/// One notification, rendered for a channel, on its way to its recipient.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// The channel, such as `email` or `push`.
    pub channel: String,
    /// The account it is for.
    pub account: uuid::Uuid,
    /// Where it goes: an email address, or the endpoints of the push subscriptions of the account.
    pub to: Vec<String>,
    /// Its subject or title.
    pub subject: String,
    /// Its text.
    pub body: String,
}

/// Carries envelopes of one channel.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Sends one envelope; an error leaves the delivery to be tried again.
    async fn send(&self, envelope: &Envelope) -> Result<()>;
}

/// Keeps envelopes in memory, for tests and for local development.
#[derive(Debug, Default)]
pub struct MemoryTransport {
    /// The envelopes sent, oldest first.
    sent: Mutex<Vec<Envelope>>,
}

impl MemoryTransport {
    /// The envelopes sent so far.
    #[must_use]
    pub fn sent(&self) -> Vec<Envelope> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl Transport for MemoryTransport {
    async fn send(&self, envelope: &Envelope) -> Result<()> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(envelope.clone());
        Ok(())
    }
}

/// Writes envelopes to the log instead of sending them: what a server without a transport configured for a
/// channel does.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogTransport;

#[async_trait]
impl Transport for LogTransport {
    async fn send(&self, envelope: &Envelope) -> Result<()> {
        tracing::info!(
            channel = %envelope.channel,
            account = %envelope.account,
            recipients = envelope.to.len(),
            subject = %envelope.subject,
            "notification not sent: no transport configured for its channel"
        );
        Ok(())
    }
}
