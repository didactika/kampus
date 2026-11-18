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

//! Time-based one-time passwords (RFC 6238 over RFC 4226, HMAC-SHA1, six digits, 30-second steps), the
//! base32 form of their secrets (RFC 4648), and the `otpauth://` link authenticator apps read.

use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

/// The length of a step, in seconds.
pub const STEP: i64 = 30;

/// The digits of a code.
pub const DIGITS: u32 = 6;

/// The steps before and after the current one that are accepted, for clocks that drift.
const DRIFT: i64 = 1;

/// The alphabet of base32.
const BASE32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// The step of an instant, in seconds since the Unix epoch.
#[must_use]
pub const fn step_of(unix: i64) -> i64 {
    unix.div_euclid(STEP)
}

/// The code of a secret for one step, with `digits` digits.
#[must_use]
pub fn code(secret: &[u8], step: i64, digits: u32) -> String {
    let Ok(mut mac) = Hmac::<Sha1>::new_from_slice(secret) else {
        return String::new();
    };
    mac.update(&step.to_be_bytes());
    let hash = mac.finalize().into_bytes();
    let offset = usize::from(hash[hash.len() - 1] & 0x0f);
    let binary = u32::from_be_bytes([
        hash[offset] & 0x7f,
        hash[offset + 1],
        hash[offset + 2],
        hash[offset + 3],
    ]);
    let width = usize::try_from(digits).unwrap_or(6);
    format!("{:0width$}", binary % 10_u32.pow(digits))
}

/// The step a code matches at an instant, among the current one and its neighbours, refusing steps at or
/// before `after` so a code is never used twice.
#[must_use]
pub fn verify(secret: &[u8], given: &str, unix: i64, after: Option<i64>) -> Option<i64> {
    let given: String = given.chars().filter(|c| !c.is_whitespace()).collect();
    if given.len() != DIGITS as usize {
        return None;
    }
    let now = step_of(unix);
    (now - DRIFT..=now + DRIFT)
        .filter(|step| after.is_none_or(|after| *step > after))
        .find(|step| crate::secret::same(code(secret, *step, DIGITS).as_bytes(), given.as_bytes()))
}

/// Encodes bytes in base32 without padding.
#[must_use]
pub fn base32_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(5) * 8);
    let (mut buffer, mut bits) = (0_u32, 0_u32);
    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(char::from(BASE32[((buffer >> bits) & 31) as usize]));
        }
    }
    if bits > 0 {
        out.push(char::from(BASE32[((buffer << (5 - bits)) & 31) as usize]));
    }
    out
}

/// Decodes base32, ignoring case, spaces and padding.
#[must_use]
pub fn base32_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 5 / 8);
    let (mut buffer, mut bits) = (0_u32, 0_u32);
    for c in text.chars().filter(|c| !c.is_whitespace() && *c != '=') {
        let upper = u8::try_from(c.to_ascii_uppercase()).ok()?;
        let value = u32::try_from(BASE32.iter().position(|&b| b == upper)?).ok()?;
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((buffer >> bits) & 0xff).ok()?);
        }
    }
    Some(out)
}

/// The link an authenticator app reads, usually from a QR code.
#[must_use]
pub fn uri(issuer: &str, account: &str, secret: &str) -> String {
    let issuer = percent(issuer);
    format!(
        "otpauth://totp/{issuer}:{}?secret={secret}&issuer={issuer}&algorithm=SHA1&digits={DIGITS}&period={STEP}",
        percent(account)
    )
}

/// Percent-encodes everything but the unreserved characters of RFC 3986.
fn percent(text: &str) -> String {
    /// The hexadecimal digits.
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{base32_decode, base32_encode, code, step_of, uri, verify};

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
}
