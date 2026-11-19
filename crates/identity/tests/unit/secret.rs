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

//! Tests of `kampus-identity`: support secret.

use kampus_identity::support::secret::{new_token, same, token_hash};
use kampus_identity::{PasswordCost, PasswordHasher, SecretKey};

/// A cheap cost for tests.
const CHEAP: PasswordCost = PasswordCost {
    memory_kib: 64,
    iterations: 1,
    parallelism: 1,
};

#[tokio::test]
async fn a_password_verifies_against_its_own_hash_only() {
    let hasher = PasswordHasher::new(CHEAP);
    let stored = hasher.hash("correct horse battery").await.unwrap();
    assert!(stored.starts_with("$argon2id$v=19$m=64,t=1,p=1$"));
    assert!(
        hasher
            .verify("correct horse battery", Some(&stored))
            .await
            .unwrap()
    );
    assert!(!hasher.verify("wrong horse battery", Some(&stored)).await.unwrap());
    assert!(!hasher.verify("correct horse battery", None).await.unwrap());
    assert_ne!(stored, hasher.hash("correct horse battery").await.unwrap());
}

#[test]
fn sealed_secrets_open_only_with_their_key() {
    let key = SecretKey::generate().unwrap();
    let sealed = key.seal(b"totp secret").unwrap();
    assert_eq!(key.open(&sealed).unwrap(), b"totp secret");
    assert_ne!(key.seal(b"totp secret").unwrap(), sealed);
    assert!(SecretKey::generate().unwrap().open(&sealed).is_err());
    let mut changed = sealed.clone();
    if let Some(last) = changed.last_mut() {
        *last ^= 1;
    }
    assert!(key.open(&changed).is_err());
    let again = SecretKey::from_base64(&key.to_base64()).unwrap();
    assert_eq!(again.open(&sealed).unwrap(), b"totp secret");
    assert!(SecretKey::from_base64("short").is_err());
}

#[test]
fn tokens_are_random_and_hashed() {
    let (a, b) = (new_token().unwrap(), new_token().unwrap());
    assert_ne!(a, b);
    assert_eq!(a.len(), 43);
    assert_eq!(token_hash(&a).len(), 32);
    assert!(same(&token_hash(&a), &token_hash(&a)));
    assert!(!same(&token_hash(&a), &token_hash(&b)));
}
