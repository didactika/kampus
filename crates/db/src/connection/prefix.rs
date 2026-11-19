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
// @package   kampus-db
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The installation prefix: every physical namespace the platform creates starts with it, so
//! several installations can share one database and table names may be reserved words.

use std::fmt;
use std::str::FromStr;

use kampus_common::{Error, FieldError};

/// The installation prefix, `kmp_` by default: 2 to 8 lower-case letters, digits and a final `_`,
/// starting with a letter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix(String);

impl Prefix {
    /// The prefix of an installation that does not set one.
    pub const DEFAULT: &'static str = "kmp_";

    /// The quoted PostgreSQL schema of a module: `courses` becomes `"kmp_courses"`.
    #[must_use]
    pub fn schema(&self, module: &str) -> String {
        format!("\"{}{}\"", self.0, module)
    }

    /// The unquoted schema name of a module, for catalogue queries.
    #[must_use]
    pub fn schema_name(&self, module: &str) -> String {
        format!("{}{}", self.0, module)
    }

    /// The prefix as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Prefix {
    fn default() -> Self {
        Self(Self::DEFAULT.to_owned())
    }
}

impl FromStr for Prefix {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bytes = text.as_bytes();
        let valid = (2..=8).contains(&bytes.len())
            && bytes[0].is_ascii_lowercase()
            && bytes[bytes.len() - 1] == b'_'
            && bytes[..bytes.len() - 1]
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(Error::validation(
                "db.prefix_invalid",
                vec![FieldError::new("KAMPUS_DATABASE_PREFIX", "format")],
            )
            .with_param("prefix", text))
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
