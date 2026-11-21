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
// @package   kampus-privacy
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The events privacy publishes: requests made, decided and completed, accounts erased, policies published and
//! accepted, and legal holds placed and released.

use kampus_module::EventDeclaration;

/// A request for personal data was made.
pub const REQUEST_CREATED: &str = "privacy.request_created";

/// Every part of a request was answered.
pub const REQUEST_COMPLETED: &str = "privacy.request_completed";

/// A new version of a policy was published.
pub const POLICY_PUBLISHED: &str = "privacy.policy_published";

/// A person accepted a version of a policy.
pub const POLICY_ACCEPTED: &str = "privacy.policy_accepted";

/// A deletion was approved or rejected.
pub const REQUEST_DECIDED: &str = "privacy.request_decided";

/// The personal data of an account was erased and the account left as a tombstone.
pub const ACCOUNT_ERASED: &str = "privacy.account_erased";

/// A legal hold was placed.
pub const HOLD_PLACED: &str = "privacy.hold_placed";

/// A legal hold was released.
pub const HOLD_RELEASED: &str = "privacy.hold_released";

/// The data of a deleted account was purged when its retention period ended.
pub const DATA_PURGED: &str = "privacy.data_purged";

/// The events of this module.
pub const EVENTS: &[EventDeclaration] = &[
    EventDeclaration {
        code: REQUEST_CREATED,
        version: 1,
        summary: "A request for personal data was made",
        fields: &["request", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: REQUEST_COMPLETED,
        version: 1,
        summary: "Every part of a request for personal data was answered",
        fields: &["request", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: POLICY_PUBLISHED,
        version: 1,
        summary: "A new version of a policy was published",
        fields: &["policy", "code", "version"],
        audited: true,
        personal_fields: &[],
    },
    EventDeclaration {
        code: POLICY_ACCEPTED,
        version: 1,
        summary: "A person accepted a version of a policy",
        fields: &["policy", "version", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: REQUEST_DECIDED,
        version: 1,
        summary: "A deletion of personal data was approved or rejected",
        fields: &["request", "account", "approved"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: ACCOUNT_ERASED,
        version: 1,
        summary: "The personal data of an account was erased",
        fields: &["request", "account"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: HOLD_PLACED,
        version: 1,
        summary: "A legal hold was placed",
        fields: &["hold", "account", "campus"],
        audited: true,
        personal_fields: &["account"],
    },
    EventDeclaration {
        code: HOLD_RELEASED,
        version: 1,
        summary: "A legal hold was released",
        fields: &["hold"],
        audited: true,
        personal_fields: &[],
    },
    EventDeclaration {
        code: DATA_PURGED,
        version: 1,
        summary: "The data of a deleted account was purged when its retention period ended",
        fields: &["account", "category"],
        audited: true,
        personal_fields: &["account"],
    },
];
