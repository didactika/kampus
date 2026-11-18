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

//! What identity tells account owners, and the port that delivers it. The notification module delivers by
//! email; a message carries the secret of a link, so a courier never writes it to a log.

use std::sync::Mutex;

use async_trait::async_trait;
use kampus_access::ScopeId;
use kampus_common::Result;

use crate::AccountId;

/// What an account owner is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    /// The link that confirms a new account (UC-006).
    ConfirmEmail {
        /// The one-time token of the link.
        token: String,
    },
    /// Someone tried to sign up with an email that already has an account (UC-006 A1).
    AlreadyRegistered,
    /// The link of an invitation, which sets the password (UC-009).
    Invitation {
        /// The one-time token of the link.
        token: String,
    },
    /// The password was changed; sessions and tokens were revoked.
    PasswordChanged,
    /// The account was suspended, or its deletion scheduled.
    Suspended,
    /// The link that starts recovering access (RN-030).
    Recovery {
        /// The one-time token of the link.
        token: String,
    },
    /// A code to sign in with, as a second factor.
    EmailCode {
        /// The code.
        code: String,
    },
    /// Access was recovered and the password changed; sessions and tokens were revoked.
    RecoveryUsed,
    /// A second factor was added.
    SecondFactorAdded,
    /// A second factor was removed.
    SecondFactorRemoved,
}

impl Notice {
    /// A name of the notice without its secret, for logs.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::ConfirmEmail { .. } => "confirm_email",
            Self::AlreadyRegistered => "already_registered",
            Self::Invitation { .. } => "invitation",
            Self::PasswordChanged => "password_changed",
            Self::Suspended => "suspended",
            Self::Recovery { .. } => "recovery",
            Self::EmailCode { .. } => "email_code",
            Self::RecoveryUsed => "recovery_used",
            Self::SecondFactorAdded => "second_factor_added",
            Self::SecondFactorRemoved => "second_factor_removed",
        }
    }
}

/// A notice for one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// The campus of the account.
    pub campus: ScopeId,
    /// The account.
    pub account: AccountId,
    /// Where it goes.
    pub email: String,
    /// What it says.
    pub notice: Notice,
}

/// Delivers messages to account owners.
#[async_trait]
pub trait Courier: Send + Sync {
    /// Delivers one message, after the change it reports is committed.
    async fn send(&self, message: Message) -> Result<()>;
}

/// Keeps messages in memory, for tests and for local development.
#[derive(Debug, Default)]
pub struct MemoryCourier {
    /// The messages sent, oldest first.
    sent: Mutex<Vec<Message>>,
}

impl MemoryCourier {
    /// The messages sent so far.
    #[must_use]
    pub fn sent(&self) -> Vec<Message> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// The last message sent to an email.
    #[must_use]
    pub fn last_to(&self, email: &str) -> Option<Message> {
        self.sent().into_iter().rev().find(|m| m.email == email)
    }
}

#[async_trait]
impl Courier for MemoryCourier {
    async fn send(&self, message: Message) -> Result<()> {
        tracing::info!(account = %message.account, notice = message.notice.name(), "identity notice kept");
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(message);
        Ok(())
    }
}
