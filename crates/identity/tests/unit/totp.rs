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
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Tests of `kampus-identity`: support totp.

use kampus_identity::support::totp::{base32_decode, base32_encode, code, step_of, uri, verify};

/// The SHA-1 secret of the test vectors of RFC 6238.
const SECRET: &[u8] = b"12345678901234567890";

#[test]
fn codes_match_the_test_vectors_of_rfc_6238() {
    for (unix, expected) in [
        (59, "94287082"),
        (1_111_111_109, "07081804"),
        (1_111_111_111, "14050471"),
        (1_234_567_890, "89005924"),
        (2_000_000_000, "69279037"),
        (20_000_000_000, "65353130"),
    ] {
        assert_eq!(code(SECRET, step_of(unix), 8), expected, "{unix}");
    }
}

#[test]
fn a_code_is_accepted_once_and_near_its_step_only() {
    let now = 1_234_567_890;
    let current = code(SECRET, step_of(now), 6);
    assert_eq!(verify(SECRET, &current, now, None), Some(step_of(now)));
    assert_eq!(verify(SECRET, &current, now + 30, None), Some(step_of(now)));
    assert_eq!(verify(SECRET, &current, now + 90, None), None);
    assert_eq!(verify(SECRET, &current, now, Some(step_of(now))), None);
    assert_eq!(verify(SECRET, "12345", now, None), None);
}

#[test]
fn base32_round_trips() {
    assert_eq!(base32_encode(b"foobar"), "MZXW6YTBOI");
    assert_eq!(base32_decode("mzxw6ytboi").unwrap(), b"foobar");
    assert_eq!(base32_decode("MZXW 6YTB OI==").unwrap(), b"foobar");
    assert!(base32_decode("not base32!").is_none());
    let secret = [7_u8; 20];
    assert_eq!(base32_decode(&base32_encode(&secret)).unwrap(), secret);
}

#[test]
fn the_link_names_the_issuer_and_the_account() {
    assert_eq!(
        uri("Kampus Campus", "ana@example.org", "MZXW6YTBOI"),
        "otpauth://totp/Kampus%20Campus:ana%40example.org?secret=MZXW6YTBOI&issuer=Kampus%20Campus&algorithm=SHA1&digits=6&period=30"
    );
}
