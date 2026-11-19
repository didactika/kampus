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

//! Tests of `kampus-common`: adapter.

use kampus_common::adapter::{AdapterSettings, choose};

#[test]
fn the_scheme_chooses_the_adapter() {
    let adapters = [("memory", &["memory"][..]), ("redis", &["redis", "valkey"][..])];
    let settings = AdapterSettings {
        url: "Valkey://cache:6379".into(),
        ..AdapterSettings::default()
    };
    assert_eq!(settings.location(), "cache:6379");
    assert_eq!(choose(&adapters, |a| a.1, &settings, "cache").unwrap().0, "redis");
    let unknown = AdapterSettings {
        url: "memcached://x".into(),
        ..AdapterSettings::default()
    };
    let error = choose(&adapters, |a| a.1, &unknown, "cache").unwrap_err();
    assert_eq!(error.code(), "cache.adapter_unknown");
    assert!(error.to_string().contains("known=memory, redis, valkey"));
    assert!(AdapterSettings::default().scheme().is_err());
}
