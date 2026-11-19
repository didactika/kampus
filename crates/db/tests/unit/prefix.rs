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

//! Tests of `kampus-db`: connection prefix.

use kampus_db::Prefix;

#[test]
fn valid_prefixes_are_accepted() {
    for text in ["kmp_", "a_", "test01_", "k2b3c4d_"] {
        assert_eq!(text.parse::<Prefix>().unwrap().as_str(), text);
    }
}

#[test]
fn invalid_prefixes_are_refused() {
    for text in ["", "_", "kmp", "Kmp_", "1kmp_", "kmp-_", "toolongpx_", "k m_"] {
        assert_eq!(
            text.parse::<Prefix>().unwrap_err().code(),
            "db.prefix_invalid",
            "{text:?}"
        );
    }
}

#[test]
fn schemas_are_prefixed_and_quoted() {
    let prefix = Prefix::default();
    assert_eq!(prefix.schema("courses"), "\"kmp_courses\"");
    assert_eq!(prefix.schema_name("db"), "kmp_db");
}
