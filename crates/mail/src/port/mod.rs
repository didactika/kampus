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
// @package   kampus-mail
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The mail port: one plain text message, checked before it leaves, and what sends it.

use std::fmt;

use async_trait::async_trait;
use kampus_common::{Error, FieldError, Result};

/// The longest subject kept; a longer one is cut.
pub const MAX_SUBJECT: usize = 200;

/// The most recipients of one message.
pub const MAX_RECIPIENTS: usize = 50;

/// A plain text message to one or more addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    /// Where it goes.
    to: Vec<String>,
    /// Its subject, on one line.
    subject: String,
    /// Its text.
    text: String,
}

/// Whether an address can be written in a header as it is: one `@` between a local part and a domain, and
/// nothing that ends a header or opens another address.
fn plain_address(address: &str) -> bool {
    let Some((local, domain)) = address.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && address.len() <= 254
        && !domain.contains('@')
        && address
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '<' | '>' | ',' | ';' | '"' | '(' | ')' | '\\'))
}

impl Mail {
    /// A message, with a subject folded onto one line and cut to [`MAX_SUBJECT`] characters.
    ///
    /// # Errors
    ///
    /// `validation` (`mail.invalid`) without recipients, with more than [`MAX_RECIPIENTS`], or with an
    /// address that is not a plain one.
    pub fn new(to: Vec<String>, subject: &str, text: &str) -> Result<Self> {
        if to.is_empty() || to.len() > MAX_RECIPIENTS {
            return Err(Error::validation(
                "mail.invalid",
                vec![FieldError::new("to", "count")],
            ));
        }
        if !to.iter().all(|a| plain_address(a)) {
            return Err(Error::validation(
                "mail.invalid",
                vec![FieldError::new("to", "email")],
            ));
        }
        let subject: String = subject
            .split(|c: char| c.is_control())
            .filter(|part| !part.trim().is_empty())
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(MAX_SUBJECT)
            .collect();
        Ok(Self {
            to,
            subject,
            text: text.replace("\r\n", "\n"),
        })
    }

    /// Where it goes.
    #[must_use]
    pub fn to(&self) -> &[String] {
        &self.to
    }

    /// Its subject, on one line.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Its text, with `\n` line ends.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The mail port.
#[async_trait]
pub trait Mailer: Send + Sync + fmt::Debug {
    /// Sends one message.
    ///
    /// # Errors
    ///
    /// `validation` (`mail.rejected`) when the server refuses it for good, so it is not tried again, and
    /// `unavailable` (`mail.failed`) when it cannot be sent now.
    async fn send(&self, mail: &Mail) -> Result<()>;

    /// Checks that the server answers, for the readiness of the process.
    ///
    /// # Errors
    ///
    /// `unavailable` when it does not.
    async fn check(&self) -> Result<()>;
}
