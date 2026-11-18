// ------------------------------------------------------------------------------
//
//    *   *    *    *   *  ****   *   *   ****
//    *  *    * *   ** **  *   *  *   *  *
//    ***    *****  * * *  ****   *   *   ***
//    *  *   *   *  *   *  *      *   *      *
//    *   *  *   *  *   *  *       ***   ****
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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Request identifiers: every request and job carries one, in every log line and in every error a caller
//! sees (DEC-018, DEC-019).

use std::fmt;

use serde::Serialize;

/// The longest request identifier accepted from a caller.
const MAX_LENGTH: usize = 128;

/// Identifies one request or job across logs, traces and errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct RequestId(String);

impl RequestId {
    /// Generates a new request identifier.
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }

    /// Accepts the identifier a caller or a proxy sent, when it is short visible ASCII.
    #[must_use]
    pub fn from_header(value: &str) -> Option<Self> {
        let valid =
            !value.is_empty() && value.len() <= MAX_LENGTH && value.bytes().all(|b| b.is_ascii_graphic());
        valid.then(|| Self(value.to_owned()))
    }

    /// The identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::RequestId;

    #[test]
    fn header_values_are_accepted_only_when_safe() {
        assert_eq!(RequestId::from_header("abc-123").unwrap().as_str(), "abc-123");
        assert!(RequestId::from_header("").is_none());
        assert!(RequestId::from_header("with space").is_none());
        assert!(RequestId::from_header(&"x".repeat(129)).is_none());
    }

    #[test]
    fn generated_identifiers_are_unique() {
        assert_ne!(RequestId::generate(), RequestId::generate());
    }
}
