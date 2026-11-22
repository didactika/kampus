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

//! The entities of privacy: a request for the personal data of an account, and the part of it each module
//! answers.

use kampus_access::PrincipalId;
use kampus_common::Timestamp;
use serde::Serialize;

kampus_common::define_id! {
    /// Identifies a request.
    pub struct RequestId;
}

kampus_common::define_id! {
    /// Identifies a policy.
    pub struct PolicyId;
}

/// A policy of a campus, as of its latest version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Policy {
    /// The policy.
    pub id: PolicyId,
    /// What people refer to it by, unique in the campus.
    pub code: String,
    /// Its title.
    pub title: String,
    /// Its latest version.
    pub version: i32,
    /// What it says in that version.
    pub body: String,
    /// When that version was published.
    pub published_at: Timestamp,
    /// When the person accepted that version, when they did.
    pub accepted_at: Option<Timestamp>,
}

/// One part of a request: a data category and where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Item {
    /// The data category.
    pub category: String,
    /// `pending`, `completed` or `failed`.
    pub state: String,
    /// Why it failed.
    pub error: Option<String>,
}

/// A request to export or to delete the personal data of an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Request {
    /// The request.
    pub id: RequestId,
    /// Whose data.
    pub account: PrincipalId,
    /// What it asks for.
    pub kind: String,
    /// `pending`, `awaiting_approval`, `held`, `completed`, `failed`, `expired` or `rejected`.
    pub state: String,
    /// When it was made.
    pub created_at: Timestamp,
    /// When it must be answered by.
    pub due_at: Timestamp,
    /// When it was answered.
    pub completed_at: Option<Timestamp>,
    /// When the data stops being available.
    pub expires_at: Option<Timestamp>,
    /// Why a deletion was asked for.
    pub reason: Option<String>,
    /// Who approved a deletion.
    pub approved_by: Option<PrincipalId>,
    /// Its parts.
    pub items: Vec<Item>,
}

kampus_common::define_id! {
    /// Identifies a legal hold.
    pub struct HoldId;
}

/// A suspension of deletion for an account, or for a whole campus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Hold {
    /// The hold.
    pub id: HoldId,
    /// The account it protects; none for a hold on the whole campus.
    pub account: Option<PrincipalId>,
    /// Why deletion waits.
    pub reason: String,
    /// When it ends by itself, when it does.
    pub until: Option<Timestamp>,
    /// When it was placed.
    pub created_at: Timestamp,
    /// When it was released by hand.
    pub released_at: Option<Timestamp>,
}

/// A kind of personal data and how long the campus keeps it after the account is deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Retention {
    /// The data category, `<module>.<what>`.
    pub category: String,
    /// Why the data is kept.
    pub purpose: String,
    /// Days it is kept after the account is deleted; none keeps it for ever.
    pub keep_days: Option<i32>,
}
