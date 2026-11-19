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
// @package   kampus-common
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Tests of `kampus-common`: typed identifiers.

kampus_common::define_id! {
    /// An identifier used only by these tests.
    struct SampleId;
}

#[test]
fn generated_identifiers_are_version_7_and_ordered() {
    let first = SampleId::generate();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let second = SampleId::generate();
    assert_eq!(first.as_uuid().get_version_num(), 7);
    assert!(first < second);
}

#[test]
fn identifiers_round_trip_as_text_and_json() {
    let id = SampleId::generate();
    assert_eq!(id.to_string().parse::<SampleId>().unwrap(), id);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, format!("\"{id}\""));
    assert_eq!(serde_json::from_str::<SampleId>(&json).unwrap(), id);
}

#[test]
fn invalid_text_is_a_validation_error() {
    let error = "42".parse::<SampleId>().unwrap_err();
    assert_eq!(error.code(), "common.id_invalid");
    assert_eq!(error.fields()[0].field, "SampleId");
}
