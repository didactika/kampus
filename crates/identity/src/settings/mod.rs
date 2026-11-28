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

//! The settings identity declares: the identity policy of each campus, set for the platform or a campus,
//! and how a policy is read from them.

use async_trait::async_trait;
use kampus_access::ScopeId;
use kampus_common::Result;
use kampus_module::{ScopeKind, SettingDeclaration, ValueKind};
use kampus_settings::Settings;
use time::Duration;

use crate::permissions::POLICY_MANAGE;
use crate::{CampusPolicy, Policies};

/// The levels where the policy is set.
const LEVELS: &[ScopeKind] = &[ScopeKind::Platform, ScopeKind::Campus];

/// A setting of the policy.
const fn setting(
    key: &'static str,
    group: &'static str,
    summary: &'static str,
    kind: ValueKind,
) -> SettingDeclaration {
    SettingDeclaration {
        key,
        summary,
        group,
        kind,
        levels: LEVELS,
        permission: POLICY_MANAGE,
        restart_required: false,
    }
}

/// A whole number of the policy.
const fn number(
    key: &'static str,
    group: &'static str,
    summary: &'static str,
    min: i64,
    max: i64,
    default: i64,
) -> SettingDeclaration {
    setting(key, group, summary, ValueKind::Integer { min, max, default })
}

/// The settings of this module.
pub const SETTINGS: &[SettingDeclaration] = &[
    setting(
        "identity.self_sign_up",
        "sign_up",
        "Whether visitors may sign up by themselves",
        ValueKind::Boolean { default: false },
    ),
    setting(
        "identity.allowed_domains",
        "sign_up",
        "The email domains allowed to sign up, separated by commas; empty allows any",
        ValueKind::Text {
            max: 2000,
            default: "",
        },
    ),
    number(
        "identity.password_min_length",
        "passwords",
        "The shortest password",
        8,
        128,
        12,
    ),
    setting(
        "identity.second_factor_required",
        "sign_in",
        "Whether every account needs a second factor",
        ValueKind::Boolean { default: false },
    ),
    number(
        "identity.confirmation_hours",
        "sign_up",
        "How long a confirmation link lasts, in hours",
        1,
        168,
        24,
    ),
    number(
        "identity.invitation_days",
        "sign_up",
        "How long an invitation link lasts, in days",
        1,
        90,
        7,
    ),
    number(
        "identity.recovery_minutes",
        "passwords",
        "How long a recovery link lasts, in minutes",
        5,
        1440,
        30,
    ),
    number(
        "identity.session_idle_hours",
        "sessions",
        "How long a session lasts without being used, in hours",
        1,
        720,
        8,
    ),
    number(
        "identity.session_lifetime_days",
        "sessions",
        "How long a session lasts at most, in days",
        1,
        365,
        7,
    ),
    number(
        "identity.step_up_minutes",
        "sign_in",
        "How old the last sign-in may be for a sensitive operation, in minutes",
        1,
        1440,
        15,
    ),
    number(
        "identity.token_default_days",
        "tokens",
        "The lifetime of an API token when none is asked, in days",
        1,
        3650,
        90,
    ),
    number(
        "identity.token_max_days",
        "tokens",
        "The longest lifetime of an API token, in days",
        1,
        3650,
        365,
    ),
    number(
        "identity.deletion_grace_days",
        "accounts",
        "The grace period before a deletion runs, in days",
        0,
        365,
        14,
    ),
    number(
        "identity.attempt_retention_days",
        "sign_in",
        "How long sign-in attempts are kept, in days",
        1,
        3650,
        30,
    ),
    number(
        "identity.throttle_slow_after",
        "sign_in",
        "Failed sign-ins after which each new try waits longer",
        1,
        1000,
        3,
    ),
    number(
        "identity.throttle_lock_after",
        "sign_in",
        "Failed sign-ins after which sign-in locks",
        1,
        1000,
        10,
    ),
    number(
        "identity.throttle_lock_minutes",
        "sign_in",
        "How long a sign-in lock lasts, in minutes",
        1,
        1440,
        15,
    ),
];

/// The policy of each campus, read from its settings.
#[derive(Debug, Clone)]
pub struct SettingsPolicies {
    /// The settings of the installation.
    settings: Settings,
}

impl SettingsPolicies {
    /// Reads policies from the settings of an installation.
    #[must_use]
    pub const fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

#[async_trait]
impl Policies for SettingsPolicies {
    async fn policy(&self, campus: ScopeId) -> Result<CampusPolicy> {
        Ok(policy_of(
            &self.settings.resolve_prefix("identity.", campus).await?,
        ))
    }

    async fn policy_in(&self, conn: &mut sqlx::PgConnection, campus: ScopeId) -> Result<CampusPolicy> {
        Ok(policy_of(
            &self.settings.resolve_prefix_in(conn, "identity.", campus).await?,
        ))
    }
}

/// The policy the resolved settings of a campus give.
fn policy_of(values: &std::collections::BTreeMap<String, kampus_settings::Resolved>) -> CampusPolicy {
    {
        let defaults = CampusPolicy::default();
        let number = |key: &str| values.get(key).and_then(|r| r.value.as_i64());
        let flag =
            |key: &str, default: bool| values.get(key).and_then(|r| r.value.as_bool()).unwrap_or(default);
        let minutes = |key: &str, scale: i64, default: Duration| {
            number(key).map_or(default, |n| Duration::minutes(n.saturating_mul(scale)))
        };
        let count =
            |key: &str, default: u32| number(key).and_then(|n| u32::try_from(n).ok()).unwrap_or(default);
        CampusPolicy {
            self_sign_up: flag("identity.self_sign_up", defaults.self_sign_up),
            allowed_domains: values
                .get("identity.allowed_domains")
                .and_then(|r| r.value.as_str())
                .map(|text| {
                    text.split(',')
                        .map(|d| d.trim().to_lowercase())
                        .filter(|d| !d.is_empty())
                        .collect()
                })
                .unwrap_or(defaults.allowed_domains),
            password_min_length: number("identity.password_min_length")
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(defaults.password_min_length),
            second_factor_required: flag("identity.second_factor_required", defaults.second_factor_required),
            confirmation_lifetime: minutes("identity.confirmation_hours", 60, defaults.confirmation_lifetime),
            invitation_lifetime: minutes("identity.invitation_days", 24 * 60, defaults.invitation_lifetime),
            recovery_lifetime: minutes("identity.recovery_minutes", 1, defaults.recovery_lifetime),
            session_idle: minutes("identity.session_idle_hours", 60, defaults.session_idle),
            session_lifetime: minutes(
                "identity.session_lifetime_days",
                24 * 60,
                defaults.session_lifetime,
            ),
            step_up_window: minutes("identity.step_up_minutes", 1, defaults.step_up_window),
            token_default_lifetime: minutes(
                "identity.token_default_days",
                24 * 60,
                defaults.token_default_lifetime,
            ),
            token_max_lifetime: minutes("identity.token_max_days", 24 * 60, defaults.token_max_lifetime),
            deletion_grace: minutes("identity.deletion_grace_days", 24 * 60, defaults.deletion_grace),
            attempt_retention: minutes(
                "identity.attempt_retention_days",
                24 * 60,
                defaults.attempt_retention,
            ),
            throttle: crate::Throttle {
                slow_after: count("identity.throttle_slow_after", defaults.throttle.slow_after),
                lock_after: count("identity.throttle_lock_after", defaults.throttle.lock_after),
                lock_for: minutes("identity.throttle_lock_minutes", 1, defaults.throttle.lock_for),
                ..defaults.throttle
            },
        }
    }
}
