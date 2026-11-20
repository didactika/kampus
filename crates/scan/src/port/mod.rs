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
// @package   kampus-scan
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The scanning port: what a scanner says of some content.

use std::fmt;

use async_trait::async_trait;
use bytes::Bytes;
use kampus_common::Result;

/// The standard test file every antivirus reports, harmless by design.
pub const EICAR: &[u8] = br"X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

/// What a scanner says of some content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Nothing found.
    Clean,
    /// Malware found, with the name the scanner gives it.
    Infected(String),
}

/// The malware scanning port.
#[async_trait]
pub trait Scanner: Send + Sync + fmt::Debug {
    /// Scans content.
    ///
    /// # Errors
    ///
    /// `unavailable` when the scanner cannot be reached or fails; content is then not kept.
    async fn scan(&self, content: Bytes) -> Result<Verdict>;

    /// Checks that the scanner answers, for the readiness of the process.
    ///
    /// # Errors
    ///
    /// `unavailable` when it does not.
    async fn check(&self) -> Result<()>;
}
