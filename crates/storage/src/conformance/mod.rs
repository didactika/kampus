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

//! The conformance suite of the file storage port: an adapter is supported only when it passes
//! every check here, run against a real backend.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "a conformance failure must stop the test"
)]

use std::time::Duration;

use bytes::Bytes;
use kampus_common::ErrorKind;

use crate::{Storage, StorageKey, sha256};

/// Runs every check against an empty store, using keys under `namespace` so runs never collide.
pub async fn run(storage: &dyn Storage, namespace: &str) {
    let key = |name: &str| StorageKey::new(format!("{namespace}/{name}")).unwrap();
    storage.check().await.expect("the store answers");

    let a = key("a");
    let content = Bytes::from_static(b"Kampus stores every blob once.");
    assert_eq!(
        storage.head(&a).await.unwrap(),
        None,
        "a missing blob has no info"
    );
    assert_eq!(
        storage.get(&a).await.unwrap_err().kind(),
        ErrorKind::NotFound,
        "reading a missing blob is not_found"
    );

    let wrong = storage
        .put(&a, content.clone(), sha256(b"something else"))
        .await
        .unwrap_err();
    assert_eq!(
        wrong.code(),
        "storage.hash_mismatch",
        "content that does not match its hash is refused"
    );
    assert_eq!(
        storage.head(&a).await.unwrap(),
        None,
        "a refused blob is not stored"
    );

    storage.put(&a, content.clone(), sha256(&content)).await.unwrap();
    assert_eq!(
        storage.get(&a).await.unwrap(),
        content,
        "a blob reads back as written"
    );
    assert_eq!(
        storage.head(&a).await.unwrap().map(|i| i.size),
        Some(content.len() as u64),
        "head gives the size"
    );
    assert_eq!(
        storage.get_range(&a, 7..13).await.unwrap(),
        Bytes::from_static(b"stores"),
        "ranges read part of a blob"
    );

    storage.put(&a, content.clone(), sha256(&content)).await.unwrap();
    assert_eq!(
        storage.get(&a).await.unwrap(),
        content,
        "writing the same content again is harmless"
    );

    let b = key("nested/b");
    storage.copy(&a, &b).await.unwrap();
    assert_eq!(
        storage.get(&b).await.unwrap(),
        content,
        "a copy has the same content"
    );
    let missing = key("missing");
    assert_eq!(
        storage.copy(&missing, &key("c")).await.unwrap_err().kind(),
        ErrorKind::NotFound,
        "copying a missing blob is not_found"
    );

    let signed = storage.presign_read(&a, Duration::from_secs(60)).await.unwrap();
    assert_eq!(
        signed.is_some(),
        storage.capabilities().presign_read,
        "presigned URLs follow the capability"
    );

    let empty = key("empty");
    storage.put(&empty, Bytes::new(), sha256(b"")).await.unwrap();
    assert_eq!(
        storage.head(&empty).await.unwrap().map(|i| i.size),
        Some(0),
        "empty blobs are allowed"
    );

    for k in [&a, &b, &empty] {
        storage.delete(k).await.unwrap();
        assert_eq!(storage.head(k).await.unwrap(), None, "a deleted blob is gone");
    }
    storage.delete(&a).await.unwrap();
}
