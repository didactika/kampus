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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a module offers to privacy: the personal data it keeps, and how to export it. Privacy asks each
//! module for its own data, so it never depends on the modules it serves.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use kampus_common::Result;
use serde_json::Value;

use crate::ServiceContext;

/// What an export handler returns: the personal data of one account, as JSON.
pub type PrivacyFuture<'p> = Pin<Box<dyn Future<Output = Result<Value>> + Send + 'p>>;

/// One request to a handler.
#[derive(Debug, Clone, Copy)]
pub struct PrivacyRun<'p> {
    /// The account whose data is exported.
    pub account: uuid::Uuid,
    /// The services of the process.
    pub services: &'p ServiceContext,
    /// Set when the retention period of the data ended: even what an erasure keeps for the academic record
    /// is deleted.
    pub purge: bool,
}

/// A kind of personal data a module keeps and the handler that exports it.
#[derive(Clone, Copy)]
pub struct PrivacyHandlerDeclaration {
    /// The data category, `<module>.<what>`.
    pub code: &'static str,
    /// Why the data is kept, for the people who ask for it.
    pub purpose: &'static str,
    /// Exports the data of one account.
    pub export: for<'p> fn(PrivacyRun<'p>) -> PrivacyFuture<'p>,
    /// Erases the data of one account, or says what it keeps and why (grades are kept, pointing to the
    /// tombstone of the account); returns a report of what it did.
    pub erase: for<'p> fn(PrivacyRun<'p>) -> PrivacyFuture<'p>,
}

impl fmt::Debug for PrivacyHandlerDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivacyHandlerDeclaration")
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}
