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

//! External references: what a system code and an external id accept.

use kampus_common::ErrorKind;
use kampus_db::references::ExternalReference;

#[test]
fn a_reference_is_trimmed_and_keeps_to_its_rules() {
    let reference = ExternalReference::new(" sis ", "  A-2026/001 ").unwrap();
    assert_eq!(
        (reference.system.as_str(), reference.external_id.as_str()),
        ("sis", "A-2026/001")
    );
    ExternalReference::new("moodle.campus-1_old", "42").unwrap();
    for (system, external_id) in [
        ("SIS", "1"),
        ("1sis", "1"),
        ("sis!", "1"),
        ("", "1"),
        ("sis", " "),
        ("sis", "a\nb"),
    ] {
        let error = ExternalReference::new(system, external_id).unwrap_err();
        assert_eq!(
            (error.kind(), error.code()),
            (ErrorKind::Validation, "references.invalid"),
            "{system:?} {external_id:?}"
        );
    }
    assert!(ExternalReference::new("sis", &"x".repeat(256)).is_err());
    assert!(ExternalReference::new(&"s".repeat(64), "1").is_err());
}
