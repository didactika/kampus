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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The address of a person behind proxies: only trusted proxies may tell it, read from the right.

use std::net::IpAddr;

use kampus_server::http::client::TrustedProxies;

fn ip(text: &str) -> IpAddr {
    text.parse().unwrap()
}

#[test]
fn without_trusted_proxies_the_peer_is_the_person_whatever_it_forwards() {
    let none = TrustedProxies::default();
    assert_eq!(
        none.client(ip("172.18.0.5"), Some("203.0.113.9")),
        ip("172.18.0.5")
    );
}

#[test]
fn a_trusted_proxy_tells_the_address_read_from_the_right() {
    let trusted = TrustedProxies::parse("172.16.0.0/12, 10.0.0.1").unwrap();
    assert_eq!(
        trusted.client(ip("172.18.0.5"), Some("203.0.113.9")),
        ip("203.0.113.9")
    );
    assert_eq!(
        trusted.client(ip("172.18.0.5"), Some("198.51.100.7, 203.0.113.9, 10.0.0.1")),
        ip("203.0.113.9"),
        "an address the person wrote themselves on the left is never believed"
    );
    assert_eq!(trusted.client(ip("172.18.0.5"), None), ip("172.18.0.5"));
    assert_eq!(
        trusted.client(ip("172.18.0.5"), Some("not an address")),
        ip("172.18.0.5")
    );
}

#[test]
fn an_untrusted_peer_cannot_tell_another_address() {
    let trusted = TrustedProxies::parse("10.0.0.0/8").unwrap();
    assert_eq!(
        trusted.client(ip("203.0.113.9"), Some("10.0.0.2")),
        ip("203.0.113.9")
    );
}

#[test]
fn networks_are_read_in_both_versions_and_mistakes_are_refused() {
    let trusted = TrustedProxies::parse("fd00::/8").unwrap();
    assert_eq!(
        trusted.client(ip("fd12::1"), Some("2001:db8::7")),
        ip("2001:db8::7")
    );
    assert!(TrustedProxies::parse("10.0.0.0/33").is_none());
    assert!(TrustedProxies::parse("a network").is_none());
    assert_eq!(TrustedProxies::parse("").unwrap(), TrustedProxies::default());
}
