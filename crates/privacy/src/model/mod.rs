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

/// One part of a request: a data category and where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    /// The data category.
    pub category: String,
    /// `pending`, `completed` or `failed`.
    pub state: String,
    /// Why it failed.
    pub error: Option<String>,
}

/// A request for the personal data of an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Request {
    /// The request.
    pub id: RequestId,
    /// Whose data.
    pub account: PrincipalId,
    /// What it asks for.
    pub kind: String,
    /// `pending`, `completed`, `failed` or `expired`.
    pub state: String,
    /// When it was made.
    pub created_at: Timestamp,
    /// When it must be answered by.
    pub due_at: Timestamp,
    /// When it was answered.
    pub completed_at: Option<Timestamp>,
    /// When the data stops being available.
    pub expires_at: Option<Timestamp>,
    /// Its parts.
    pub items: Vec<Item>,
}
