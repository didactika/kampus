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
// @package   kampus-cache-redis
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The cache conformance suite against Redis or Valkey (`KAMPUS_TEST_CACHE_URL`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a failing setup must stop the test"
)]

use kampus_cache::conformance;
use kampus_cache_redis::RedisCache;

#[tokio::test]
async fn a_shared_cache_conforms() {
    let url = std::env::var("KAMPUS_TEST_CACHE_URL").expect("KAMPUS_TEST_CACHE_URL is set");
    let prefix = format!("t{}_", &uuid::Uuid::now_v7().simple().to_string()[26..]);
    let cache = RedisCache::connect(&url, &prefix).await.unwrap();
    conformance::run(&cache, "conformance").await;
}
