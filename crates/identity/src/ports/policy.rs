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

//! The identity policy of a campus: sign-up, passwords, lifetimes, throttling and deletion.
//! The settings module provides it per campus; until a campus changes it, the defaults apply.

use async_trait::async_trait;
use kampus_access::ScopeId;
use kampus_common::Result;
use time::Duration;

/// The throttling of sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Throttle {
    /// The period failures are counted over.
    pub window: Duration,
    /// Failures after which each new try waits, doubling each time.
    pub slow_after: u32,
    /// Failures after which sign-in locks.
    pub lock_after: u32,
    /// How long a lock lasts after the last failure.
    pub lock_for: Duration,
    /// Failures from one address, for any name, after which the address locks.
    pub address_lock_after: u32,
}

/// The identity policy of a campus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampusPolicy {
    /// Whether visitors may sign up by themselves.
    pub self_sign_up: bool,
    /// The email domains allowed to sign up; empty allows any.
    pub allowed_domains: Vec<String>,
    /// The shortest password.
    pub password_min_length: usize,
    /// Whether every account needs a second factor.
    pub second_factor_required: bool,
    /// How long a confirmation link lasts.
    pub confirmation_lifetime: Duration,
    /// How long an invitation link lasts.
    pub invitation_lifetime: Duration,
    /// How long a recovery link lasts.
    pub recovery_lifetime: Duration,
    /// How long a session lasts without being used.
    pub session_idle: Duration,
    /// How long a session lasts at most.
    pub session_lifetime: Duration,
    /// How old the last strong authentication may be for a sensitive operation.
    pub step_up_window: Duration,
    /// The lifetime of an API token when none is asked.
    pub token_default_lifetime: Duration,
    /// The longest lifetime of an API token.
    pub token_max_lifetime: Duration,
    /// The grace period before a deletion runs.
    pub deletion_grace: Duration,
    /// How long sign-in attempts are kept.
    pub attempt_retention: Duration,
    /// Sign-in throttling.
    pub throttle: Throttle,
}

impl Default for CampusPolicy {
    fn default() -> Self {
        Self {
            self_sign_up: false,
            allowed_domains: Vec::new(),
            password_min_length: 12,
            second_factor_required: false,
            confirmation_lifetime: Duration::hours(24),
            invitation_lifetime: Duration::days(7),
            recovery_lifetime: Duration::minutes(30),
            session_idle: Duration::hours(8),
            session_lifetime: Duration::days(7),
            step_up_window: Duration::minutes(15),
            token_default_lifetime: Duration::days(90),
            token_max_lifetime: Duration::days(365),
            deletion_grace: Duration::days(14),
            attempt_retention: Duration::days(30),
            throttle: Throttle {
                window: Duration::minutes(15),
                slow_after: 3,
                lock_after: 10,
                lock_for: Duration::minutes(15),
                address_lock_after: 50,
            },
        }
    }
}

/// Where the policy of each campus comes from.
#[async_trait]
pub trait Policies: Send + Sync {
    /// The policy of a campus.
    async fn policy(&self, campus: ScopeId) -> Result<CampusPolicy>;

    /// The policy of a campus, read on a connection the caller holds, such as the transaction of a change, so a
    /// change never waits for a second connection; policies that read no database need not override it.
    async fn policy_in(&self, _conn: &mut sqlx::PgConnection, campus: ScopeId) -> Result<CampusPolicy> {
        self.policy(campus).await
    }
}

/// The same policy for every campus: the defaults, or one given.
#[derive(Debug, Clone, Default)]
pub struct FixedPolicies(pub CampusPolicy);

#[async_trait]
impl Policies for FixedPolicies {
    async fn policy(&self, _campus: ScopeId) -> Result<CampusPolicy> {
        Ok(self.0.clone())
    }
}
