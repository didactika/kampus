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
// @package   kampus-storage-opendal
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Links the store signs: only at the public address of the store, carrying in their signature the headers
//! the response is to have in place of those kept with the blob. The development store (SeaweedFS) applies
//! only the type of them; AWS S3 applies all of them.

use std::time::Duration;

use bytes::Bytes;
use kampus_storage::{ReadHeaders, Storage, StorageKey, sha256};
use kampus_storage_opendal::{OpendalStorage, S3Config};

/// The test bucket, at a public address or none.
fn bucket(public_url: Option<String>) -> OpendalStorage {
    let endpoint = std::env::var("KAMPUS_TEST_S3_ENDPOINT").expect("KAMPUS_TEST_S3_ENDPOINT is set");
    OpendalStorage::s3(&S3Config {
        endpoint: Some(endpoint),
        region: "us-east-1".into(),
        bucket: "kampus-test".into(),
        root: "/signed-links".into(),
        access_key_id: "kampus".into(),
        secret_access_key: "kampus-secret".into(),
        public_url,
    })
    .unwrap()
}

/// A key of its own for each run.
fn key() -> StorageKey {
    StorageKey::new(format!("run-{}/blob", uuid::Uuid::now_v7().simple())).unwrap()
}

#[tokio::test]
async fn a_signed_link_points_to_the_public_address_and_carries_the_headers_asked_for() {
    let endpoint = std::env::var("KAMPUS_TEST_S3_ENDPOINT").expect("KAMPUS_TEST_S3_ENDPOINT is set");
    let storage = bucket(Some(endpoint.clone()));
    assert!(storage.capabilities().presign_read);
    let key = key();
    let content = Bytes::from_static(b"\x89PNG\r\n\x1a\na picture");
    storage
        .put(&key, content.clone(), sha256(&content))
        .await
        .unwrap();
    let headers = ReadHeaders {
        content_type: Some("image/png".into()),
        content_disposition: Some("inline; filename=\"the picture.png\"".into()),
        cache_control: Some("private, max-age=300".into()),
    };
    let url = storage
        .presign_read(&key, Duration::from_secs(300), &headers)
        .await
        .unwrap()
        .expect("a store with a public address signs links");
    assert!(url.starts_with(&endpoint), "{url} is at the public address");
    for asked in [
        "response-content-type=image/png",
        "response-content-disposition=inline%3B%20filename%3D%22the%20picture.png%22",
        "response-cache-control=private%2C%20max-age%3D300",
    ] {
        assert!(url.contains(asked), "{url} carries {asked}");
    }
    for part in ["X-Amz-Expires=300", "X-Amz-Signature="] {
        assert!(url.contains(part), "{url} is signed for its lifetime");
    }
    let response = reqwest::get(&url).await.unwrap();
    assert_eq!(response.status(), 200);
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    assert_eq!(
        header("content-type").as_deref(),
        Some("image/png"),
        "the store answers with the type asked for, not the one it keeps"
    );
    assert_eq!(
        header("content-disposition").as_deref(),
        Some("inline; filename=\"the picture.png\""),
        "and with the disposition asked for"
    );
    assert_eq!(header("cache-control").as_deref(), Some("private, max-age=300"));
    assert_eq!(response.bytes().await.unwrap(), content);
    let tampered = url.replace("max-age%3D300", "max-age%3D99999");
    assert_eq!(
        reqwest::get(&tampered).await.unwrap().status(),
        403,
        "a link whose headers were changed is refused"
    );
    let unsigned = url.split('?').next().unwrap().to_owned();
    assert_eq!(
        reqwest::get(&unsigned).await.unwrap().status(),
        403,
        "the store gives nothing without a signature"
    );
    storage.delete(&key).await.unwrap();
}

#[tokio::test]
async fn a_store_without_a_public_address_signs_no_links() {
    let storage = bucket(None);
    assert!(!storage.capabilities().presign_read);
    let signed = storage
        .presign_read(&key(), Duration::from_secs(60), &ReadHeaders::default())
        .await
        .unwrap();
    assert_eq!(signed, None);
    let dir = tempfile::tempdir().unwrap();
    let local = OpendalStorage::local(dir.path().to_str().unwrap()).unwrap();
    assert!(
        !local.capabilities().presign_read,
        "the local disk signs no links"
    );
}
