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

//! The preferences a module declares: per-person settings with a type and a default, kept by users.

/// The type of a preference, with its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceKind {
    /// On or off.
    Boolean {
        /// The value when the person did not choose one.
        default: bool,
    },
    /// One of a list of options.
    Choice {
        /// The options.
        options: &'static [&'static str],
        /// The value when the person did not choose one.
        default: &'static str,
    },
    /// A whole number in a range.
    Integer {
        /// The smallest value.
        min: i64,
        /// The largest value.
        max: i64,
        /// The value when the person did not choose one.
        default: i64,
    },
    /// Text of at most some characters.
    Text {
        /// The longest text.
        max: usize,
        /// The value when the person did not choose one.
        default: &'static str,
    },
}

/// A preference a module declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreferenceDeclaration {
    /// The key, `<module>.<name>` (`notify.digest`).
    pub key: &'static str,
    /// What it sets.
    pub summary: &'static str,
    /// Its type and default.
    pub kind: PreferenceKind,
}
