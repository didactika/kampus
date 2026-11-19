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

//! The storage conformance suite against the local disk and against an S3-compatible store
//! (`KAMPUS_TEST_S3_ENDPOINT`, bucket `kampus-test`).

use kampus_storage::conformance;
use kampus_storage_opendal::{OpendalStorage, S3Config};

/// A namespace of its own for each run.
fn namespace() -> String {
    format!("run-{}", uuid::Uuid::now_v7().simple())
}

#[tokio::test]
async fn the_local_disk_conforms() {
    let dir = tempfile::tempdir().unwrap();
    let storage = OpendalStorage::local(dir.path().to_str().unwrap()).unwrap();
    conformance::run(&storage, &namespace()).await;
}

#[tokio::test]
async fn an_s3_compatible_store_conforms() {
    let endpoint = std::env::var("KAMPUS_TEST_S3_ENDPOINT").expect("KAMPUS_TEST_S3_ENDPOINT is set");
    let storage = OpendalStorage::s3(&S3Config {
        endpoint: Some(endpoint),
        region: "us-east-1".into(),
        bucket: "kampus-test".into(),
        root: "/conformance".into(),
        access_key_id: "kampus".into(),
        secret_access_key: "kampus-secret".into(),
    })
    .unwrap();
    conformance::run(&storage, &namespace()).await;
}
