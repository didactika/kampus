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
// @package   kampus-users
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The fields of a profile.

use kampus_users::ProfileFields;
use kampus_users::service::profile::validate;

#[test]
fn fields_are_trimmed_and_empty_ones_are_absent() {
    let fields = validate(ProfileFields {
        given_name: Some("  Ana ".to_owned()),
        family_name: Some(String::new()),
        language_code: Some("es-ES".to_owned()),
        time_zone: Some("Europe/Madrid".to_owned()),
        ..ProfileFields::default()
    })
    .unwrap();
    assert_eq!(fields.given_name.as_deref(), Some("Ana"));
    assert_eq!(fields.family_name, None);
    assert_eq!(fields.language_code.as_deref(), Some("es-ES"));
}

#[test]
fn every_invalid_field_is_named() {
    let error = validate(ProfileFields {
        given_name: Some("x".repeat(101)),
        language_code: Some("Spanish".to_owned()),
        time_zone: Some("Mars/Olympus".to_owned()),
        ..ProfileFields::default()
    })
    .unwrap_err();
    let fields: Vec<_> = error
        .fields()
        .iter()
        .map(|f| (f.field.as_str(), f.rule.as_str()))
        .collect();
    assert_eq!(
        fields,
        [
            ("given_name", "length"),
            ("language_code", "bcp47"),
            ("time_zone", "iana")
        ]
    );
}
