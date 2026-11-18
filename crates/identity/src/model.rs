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

//! The entities of identity: accounts, their status and kind, sessions and API tokens.

use std::fmt;
use std::str::FromStr;

use kampus_access::{PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Timestamp};
use serde::Serialize;

/// An account is the principal of access with the same id (DEC-079).
pub type AccountId = PrincipalId;

kampus_common::define_id! {
    /// Identifies a session.
    pub struct SessionId;
}

kampus_common::define_id! {
    /// Identifies an API token.
    pub struct TokenId;
}

kampus_common::define_id! {
    /// Identifies a second factor.
    pub struct FactorId;
}

kampus_common::define_id! {
    /// Identifies an identity provider.
    pub struct ProviderId;
}

/// How a provider identity that is not linked yet finds its account (RN-028).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkingPolicy {
    /// Only identities linked before sign in; nothing is linked at sign-in.
    SubjectOnly,
    /// An identity whose email the provider verified links to the active account with that email.
    VerifiedEmail,
    /// The owner of the account confirms the link by signing in with their own credentials.
    Confirm,
}

impl LinkingPolicy {
    /// The policy as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubjectOnly => "subject_only",
            Self::VerifiedEmail => "verified_email",
            Self::Confirm => "confirm",
        }
    }
}

impl FromStr for LinkingPolicy {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "subject_only" => Ok(Self::SubjectOnly),
            "verified_email" => Ok(Self::VerifiedEmail),
            "confirm" => Ok(Self::Confirm),
            _ => Err(Error::validation(
                "identity.linking_policy_invalid",
                vec![FieldError::new("linking_policy", "one_of")],
            )),
        }
    }
}

/// The kind of a second factor (RN-029).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactorKind {
    /// A code from an authenticator app.
    Totp,
    /// A passkey (WebAuthn).
    Passkey,
    /// A code sent by email.
    Email,
}

impl FactorKind {
    /// The kind as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Totp => "totp",
            Self::Passkey => "passkey",
            Self::Email => "email",
        }
    }
}

impl FromStr for FactorKind {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "totp" => Ok(Self::Totp),
            "passkey" => Ok(Self::Passkey),
            "email" => Ok(Self::Email),
            _ => Err(Error::validation(
                "identity.factor_kind_invalid",
                vec![FieldError::new("kind", "one_of")],
            )),
        }
    }
}

/// A second factor as listed to its owner; its secret never leaves identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecondFactor {
    /// The factor.
    pub id: FactorId,
    /// Its kind.
    pub kind: FactorKind,
    /// The name its owner gave it.
    pub label: String,
    /// When it was added.
    pub created_at: Timestamp,
    /// When it was last used.
    pub last_used_at: Option<Timestamp>,
}

/// The status of an account, a state machine (RN-023).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Invited by a manager; the invitation link sets the password.
    Invited,
    /// Signed up; waits for the confirmation link.
    PendingConfirmation,
    /// May sign in.
    Active,
    /// May not sign in; every session and token is revoked.
    Suspended,
    /// A tombstone: personal data erased, the id kept (RN-034).
    Deleted,
}

impl AccountStatus {
    /// The status as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invited => "invited",
            Self::PendingConfirmation => "pending_confirmation",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Deleted => "deleted",
        }
    }

    /// Whether the lifecycle allows going from this status to `to` (RN-023). An invitation link proves the
    /// email as a confirmation link does, so an invited account becomes active directly.
    #[must_use]
    pub const fn may_become(self, to: Self) -> bool {
        matches!(
            (self, to),
            (
                Self::Invited | Self::PendingConfirmation | Self::Suspended,
                Self::Active
            ) | (
                Self::Invited | Self::PendingConfirmation | Self::Active,
                Self::Suspended
            ) | (Self::Suspended, Self::Deleted)
        )
    }
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AccountStatus {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        [
            Self::Invited,
            Self::PendingConfirmation,
            Self::Active,
            Self::Suspended,
            Self::Deleted,
        ]
        .into_iter()
        .find(|status| status.as_str() == text)
        .ok_or_else(|| {
            Error::validation(
                "identity.status_invalid",
                vec![FieldError::new("status", "one_of")],
            )
        })
    }
}

/// Who an account is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    /// A person.
    Person,
    /// An integration: no password and no second factor, API tokens only (RN-039).
    Service,
}

impl AccountKind {
    /// The kind as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Service => "service",
        }
    }
}

impl FromStr for AccountKind {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "person" => Ok(Self::Person),
            "service" => Ok(Self::Service),
            _ => Err(Error::validation(
                "identity.kind_invalid",
                vec![FieldError::new("kind", "one_of")],
            )),
        }
    }
}

/// An account as read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Account {
    /// The account, which is also its principal.
    pub id: AccountId,
    /// The campus it belongs to (RN-022).
    pub campus: ScopeId,
    /// The user scope of the account.
    pub scope: ScopeId,
    /// Person or service.
    pub kind: AccountKind,
    /// The username, lower-case; absent on a tombstone.
    pub username: Option<String>,
    /// The email, lower-case; absent on a tombstone and optional for services.
    pub email: Option<String>,
    /// When the email was proven.
    pub email_verified_at: Option<Timestamp>,
    /// The status.
    pub status: AccountStatus,
    /// Why the status was last changed.
    pub status_reason: Option<String>,
    /// When the account last signed in.
    pub last_sign_in_at: Option<Timestamp>,
    /// When a scheduled deletion runs (RN-035).
    pub deletion_due_at: Option<Timestamp>,
    /// The pseudonym of a tombstone.
    pub pseudonym: Option<String>,
    /// The version, for `If-Match`.
    pub version: i32,
}

/// How strongly a session was authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaLevel {
    /// One factor.
    None,
    /// A second factor was verified (RN-029).
    SecondFactor,
}

impl MfaLevel {
    /// The level as stored.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SecondFactor => "second_factor",
        }
    }

    /// Parses the stored level; anything unknown reads as the weakest.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        if text == "second_factor" {
            Self::SecondFactor
        } else {
            Self::None
        }
    }
}

/// A session that authenticated a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Session {
    /// The session.
    pub id: SessionId,
    /// Its account.
    pub account: AccountId,
    /// The campus of the account.
    pub campus: ScopeId,
    /// How it was authenticated.
    pub mfa_level: MfaLevel,
    /// The last strong authentication, for step-up (RN-032).
    pub strong_auth_at: Timestamp,
    /// When it ends at the latest.
    pub expires_at: Timestamp,
}

/// A session as listed to its owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionSummary {
    /// The session.
    pub id: SessionId,
    /// When it started.
    pub created_at: Timestamp,
    /// When it was last used.
    pub last_seen_at: Timestamp,
    /// When it ends at the latest.
    pub expires_at: Timestamp,
    /// The client that opened it.
    pub client: Option<String>,
    /// The address it was opened from.
    pub address: Option<String>,
}
