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
//! email address or the push subscriptions of their devices. Delivery jobs call it after the change that caused
//! the notification is committed, and retry it when it fails.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kampus_common::Result;
use kampus_mail::{Mail, Mailer};
use kampus_push::{Device, Outcome, Pusher};
use serde_json::json;

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
    /// The devices of a push envelope, with their keys; none for email.
    pub devices: Vec<Device>,
}

/// What a transport reports of an envelope it sent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sent {
    /// The addresses that are gone for good, whose subscriptions are removed.
    pub gone: Vec<String>,
}

/// Carries envelopes of one channel.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Sends one envelope; an error leaves the delivery to be tried again, unless it is a `validation` one.
    async fn send(&self, envelope: &Envelope) -> Result<Sent>;

    /// The public key browsers subscribe with, for a push transport that has one.
    fn public_key(&self) -> Option<String> {
        None
    }
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
    async fn send(&self, envelope: &Envelope) -> Result<Sent> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(envelope.clone());
        Ok(Sent::default())
    }
}

/// Writes envelopes to the log instead of sending them: what a server without a transport configured for a
/// channel does.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogTransport;

#[async_trait]
impl Transport for LogTransport {
    async fn send(&self, envelope: &Envelope) -> Result<Sent> {
        tracing::info!(
            channel = %envelope.channel,
            account = %envelope.account,
            recipients = envelope.to.len(),
            subject = %envelope.subject,
            "notification not sent: no transport configured for its channel"
        );
        Ok(Sent::default())
    }
}

/// Sends the email channel through the mail port: one message to the address of the account.
#[derive(Debug, Clone)]
pub struct MailTransport {
    /// What sends the messages.
    mailer: Arc<dyn Mailer>,
}

impl MailTransport {
    /// The email channel over a mailer.
    #[must_use]
    pub fn new(mailer: Arc<dyn Mailer>) -> Self {
        Self { mailer }
    }
}

#[async_trait]
impl Transport for MailTransport {
    async fn send(&self, envelope: &Envelope) -> Result<Sent> {
        let mail = Mail::new(envelope.to.clone(), &envelope.subject, &envelope.body)?;
        self.mailer.send(&mail).await?;
        Ok(Sent::default())
    }
}

/// Sends the push channel through the push port: the title and the text, in JSON, to each device of the
/// account. It is sent when one device took it; devices that are gone are reported to be removed.
#[derive(Debug, Clone)]
pub struct PushTransport {
    /// What sends the messages.
    pusher: Arc<dyn Pusher>,
}

impl PushTransport {
    /// The push channel over a pusher.
    #[must_use]
    pub fn new(pusher: Arc<dyn Pusher>) -> Self {
        Self { pusher }
    }
}

/// The message of an envelope, its text cut so it fits what push services take.
fn push_payload(envelope: &Envelope) -> Vec<u8> {
    let mut body: Vec<char> = envelope.body.chars().collect();
    loop {
        let text: String = body.iter().collect();
        let payload = json!({ "title": envelope.subject, "body": text })
            .to_string()
            .into_bytes();
        if payload.len() <= kampus_push::MAX_PAYLOAD || body.is_empty() {
            return payload;
        }
        let excess = payload.len() - kampus_push::MAX_PAYLOAD;
        body.truncate(body.len().saturating_sub(excess.max(1)));
    }
}

#[async_trait]
impl Transport for PushTransport {
    async fn send(&self, envelope: &Envelope) -> Result<Sent> {
        let payload = push_payload(envelope);
        let mut sent = Sent::default();
        let mut delivered = false;
        let mut failure = None;
        for device in &envelope.devices {
            match self.pusher.push(device, &payload).await {
                Ok(Outcome::Delivered) => delivered = true,
                Ok(Outcome::Gone) => sent.gone.push(device.endpoint.clone()),
                Err(error) => {
                    if failure.as_ref().is_none_or(|f: &kampus_common::Error| {
                        f.kind() == kampus_common::ErrorKind::Validation
                    }) {
                        failure = Some(error);
                    }
                }
            }
        }
        match failure {
            Some(error) if !delivered => Err(error),
            _ => Ok(sent),
        }
    }

    fn public_key(&self) -> Option<String> {
        Some(self.pusher.public_key())
    }
}
