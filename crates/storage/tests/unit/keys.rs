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
// @package   kampus-storage
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Tests of `kampus-storage`: port.

use kampus_storage::{StorageKey, sha256, verify};

#[test]
fn keys_are_validated() {
    for key in ["ab/cd/abcd", "files/report-2026_01.pdf", "a"] {
        assert!(StorageKey::new(key).is_ok(), "{key}");
    }
    for key in ["", "/a", "a//b", "A/b", "a/../b", "a b", "ñ"] {
        assert_eq!(
            StorageKey::new(key).unwrap_err().code(),
            "storage.key_invalid",
            "{key:?}"
        );
    }
}

#[test]
fn hash_keys_are_sharded() {
    let key = StorageKey::for_hash(&sha256(b"hello"));
    assert!(key.as_str().starts_with("2c/f2/2cf24dba5fb0a30e"));
    assert!(StorageKey::new(key.as_str()).is_ok());
}

#[test]
fn content_must_match_its_hash() {
    let key = StorageKey::new("a").unwrap();
    assert!(verify(&key, b"hello", &sha256(b"hello")).is_ok());
    assert_eq!(
        verify(&key, b"hello", &sha256(b"other")).unwrap_err().code(),
        "storage.hash_mismatch"
    );
}
