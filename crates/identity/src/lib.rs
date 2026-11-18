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

//! Identity of Kampus (DEC-030 to DEC-033): accounts of a campus and their lifecycle, passwords, one-time
//! tokens, the sign-in pipeline with throttling, OpenID Connect providers, second factors and recovery,
//! server-side sessions with step-up, and API tokens.
//!
//! Every account is a principal of access with the same id; identity keeps the principal active only
//! while the account is (RN-010), so access never reads identity.

pub mod account;
pub mod api_token;
pub mod courier;
pub mod declarations;
pub mod jobs;
pub mod lifecycle;
pub mod model;
pub mod oidc;
pub mod one_time;
pub mod password;
pub mod policy;
pub mod recovery;
pub mod schema;
pub mod second_factor;
pub mod secret;
pub mod session;
pub mod sign_in;
pub mod totp;

use std::sync::Arc;

use kampus_access::Access;
use kampus_common::{Clock, Timestamp};
use kampus_db::Database;

pub use courier::{Courier, MemoryCourier, Message, Notice};
pub use model::{
    Account, AccountId, AccountKind, AccountStatus, FactorId, FactorKind, LinkingPolicy, MfaLevel,
    ProviderId, SecondFactor, Session, SessionId, SessionSummary, TokenId,
};
pub use policy::{CampusPolicy, FixedPolicies, Policies, Throttle};
pub use secret::{PasswordCost, PasswordHasher, SecretKey};

/// The module name, as owner of the grants it gives (RN-016).
pub const MODULE: &str = "identity";

/// The identity service.
#[derive(Clone)]
pub struct Identity {
    /// The database of the installation.
    db: Database,
    /// Access, for principals, grants and checks.
    access: Access,
    /// Where "now" comes from (DEC-022).
    clock: Arc<dyn Clock>,
    /// The policy of each campus.
    policies: Arc<dyn Policies>,
    /// Delivers notices to account owners.
    courier: Arc<dyn Courier>,
    /// Hashes passwords.
    hasher: PasswordHasher,
    /// Seals the secrets identity reads back.
    key: SecretKey,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl Identity {
    /// Creates the service on an installation's database, hashing passwords at the recommended cost.
    #[must_use]
    pub fn new(
        db: Database,
        access: Access,
        clock: Arc<dyn Clock>,
        policies: Arc<dyn Policies>,
        courier: Arc<dyn Courier>,
        key: SecretKey,
    ) -> Self {
        Self {
            db,
            access,
            clock,
            policies,
            courier,
            hasher: PasswordHasher::new(PasswordCost::default()),
            key,
        }
    }

    /// Uses another cost for new password hashes; tests use a cheap one.
    #[must_use]
    pub fn with_password_cost(mut self, cost: PasswordCost) -> Self {
        self.hasher = PasswordHasher::new(cost);
        self
    }

    /// The quoted name of a table of this module.
    fn table(&self, name: &str) -> String {
        format!("{}.\"{name}\"", self.db.prefix().schema(schema::MODULE))
    }

    /// The current instant.
    fn now(&self) -> Timestamp {
        self.clock.now()
    }

    /// Sends a message after its change is committed; a failure is logged, never shown, so the answer of
    /// the request does not depend on delivery (RN-027).
    async fn deliver(&self, message: Message) {
        let (account, notice) = (message.account, message.notice.name());
        if let Err(error) = self.courier.send(message).await {
            tracing::error!(%account, notice, code = error.code(), "identity notice not delivered");
        }
    }
}

/// Wraps SQL built by this module as audited: only quoted, prefixed table names are interpolated, and
/// every value is bound.
fn sql(text: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(text)
}

/// Adds a duration to an instant.
fn later(instant: Timestamp, by: time::Duration) -> Timestamp {
    Timestamp::from_offset_date_time(instant.as_offset_date_time() + by)
}
