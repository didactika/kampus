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
// @package   kampus-mail-smtp
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! A client of an SMTP server: each message is built as plain text in UTF-8, dated and with an id, and
//! handed to the server through a pool of connections.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use kampus_common::{AdapterSettings, Error, FieldError, Result};
use kampus_mail::{Mail, Mailer};
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// How long talking to the server may take.
const TIMEOUT: Duration = Duration::from_secs(30);

/// An SMTP server and the sender of its messages.
pub struct Smtp {
    /// The pool of connections to the server.
    transport: AsyncSmtpTransport<Tokio1Executor>,
    /// Who the messages come from.
    from: Mailbox,
}

impl fmt::Debug for Smtp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Smtp")
            .field("from", &self.from.to_string())
            .finish_non_exhaustive()
    }
}

/// The error of a server that cannot be reached or does not take a message now.
fn failed(what: &str) -> Error {
    Error::unavailable("mail.failed").with_param("cause", what.to_owned())
}

/// The error of settings that cannot open the adapter.
fn invalid(field: &str, rule: &str) -> Error {
    Error::validation("mail.settings_invalid", vec![FieldError::new(field, rule)])
}

impl Smtp {
    /// Opens the server of the settings, without connecting yet.
    ///
    /// # Errors
    ///
    /// `validation` (`mail.settings_invalid`) for a URL that is not an SMTP one, without a `from` option, or
    /// with a `from` that is not an address.
    pub fn open(settings: &AdapterSettings) -> Result<Self> {
        let from: Mailbox = settings
            .option("from")
            .ok_or_else(|| invalid("from", "required"))?
            .parse()
            .map_err(|_| invalid("from", "email"))?;
        let builder = AsyncSmtpTransport::<Tokio1Executor>::from_url(&settings.url)
            .map_err(|_| invalid("url", "smtp"))?;
        let builder = match (settings.option("username"), settings.option("password")) {
            (Some(user), Some(password)) => {
                builder.credentials(Credentials::new(user.to_owned(), password.to_owned()))
            }
            (None, None) => builder,
            _ => return Err(invalid("password", "with_username")),
        };
        Ok(Self {
            transport: builder.timeout(Some(TIMEOUT)).build(),
            from,
        })
    }

    /// The message of a mail, from the sender of this server.
    fn message(&self, mail: &Mail) -> Result<Message> {
        let mut builder = Message::builder()
            .from(self.from.clone())
            .subject(mail.subject())
            .date_now()
            .header(ContentType::TEXT_PLAIN);
        for to in mail.to() {
            builder = builder.to(to
                .parse()
                .map_err(|_| Error::validation("mail.invalid", vec![FieldError::new("to", "email")]))?);
        }
        builder
            .body(mail.text().to_owned())
            .map_err(|_| Error::validation("mail.invalid", vec![FieldError::new("to", "email")]))
    }
}

#[async_trait]
impl Mailer for Smtp {
    async fn send(&self, mail: &Mail) -> Result<()> {
        let message = self.message(mail)?;
        match self.transport.send(message).await {
            Ok(_) => Ok(()),
            Err(e) if e.is_permanent() => Err(Error::validation(
                "mail.rejected",
                vec![FieldError::new("to", "rejected")],
            )),
            Err(e) if e.is_timeout() => Err(failed("timeout")),
            Err(e) if e.is_transient() => Err(failed("refused_for_now")),
            Err(_) => Err(failed("connect")),
        }
    }

    async fn check(&self) -> Result<()> {
        match self.transport.test_connection().await {
            Ok(true) => Ok(()),
            _ => Err(failed("connect")),
        }
    }
}
