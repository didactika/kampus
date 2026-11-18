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
// @package   kampus-cache
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The conformance suite of the cache port: an adapter is supported only when it passes every
//! check here, run against a real backend.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "a conformance failure must stop the test"
)]

use std::time::Duration;

use bytes::Bytes;

use crate::{Cache, LockToken};

/// Runs every check, using `namespace` so runs never collide.
pub async fn run(cache: &dyn Cache, namespace: &str) {
    cache.check().await.expect("the cache answers");
    let ns = namespace;

    assert_eq!(
        cache.get(ns, "missing").await.unwrap(),
        None,
        "a missing entry reads as none"
    );
    cache
        .set(ns, "a", Bytes::from_static(b"one"), Duration::from_secs(60))
        .await
        .unwrap();
    assert_eq!(
        cache.get(ns, "a").await.unwrap(),
        Some(Bytes::from_static(b"one")),
        "an entry reads back"
    );
    cache
        .set(ns, "a", Bytes::from_static(b"two"), Duration::from_secs(60))
        .await
        .unwrap();
    assert_eq!(
        cache.get(ns, "a").await.unwrap(),
        Some(Bytes::from_static(b"two")),
        "writing again replaces it"
    );
    assert_eq!(
        cache.get(&format!("{ns}-other"), "a").await.unwrap(),
        None,
        "namespaces are separate"
    );
    cache.delete(ns, "a").await.unwrap();
    assert_eq!(cache.get(ns, "a").await.unwrap(), None, "a deleted entry is gone");
    cache.delete(ns, "a").await.unwrap();

    cache
        .set(ns, "short", Bytes::from_static(b"x"), Duration::from_millis(300))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(
        cache.get(ns, "short").await.unwrap(),
        None,
        "an entry expires after its time to live"
    );

    let window = Duration::from_millis(800);
    assert_eq!(
        cache.increment(ns, "hits", window).await.unwrap(),
        1,
        "a counter starts at one"
    );
    assert_eq!(
        cache.increment(ns, "hits", window).await.unwrap(),
        2,
        "a counter counts"
    );
    tokio::time::sleep(Duration::from_millis(1600)).await;
    assert_eq!(
        cache.increment(ns, "hits", window).await.unwrap(),
        1,
        "a counter starts again after its window"
    );

    let lease = Duration::from_secs(30);
    let token = cache
        .try_lock(ns, "job", lease)
        .await
        .unwrap()
        .expect("a free lock is taken");
    assert!(
        cache.try_lock(ns, "job", lease).await.unwrap().is_none(),
        "a held lock cannot be taken again"
    );
    assert!(
        !cache.unlock(ns, "job", &LockToken::generate()).await.unwrap(),
        "only the holder releases a lock"
    );
    assert!(
        cache.unlock(ns, "job", &token).await.unwrap(),
        "the holder releases its lock"
    );
    assert!(
        cache.try_lock(ns, "job", lease).await.unwrap().is_some(),
        "a released lock can be taken"
    );

    let short = cache
        .try_lock(ns, "lease", Duration::from_millis(300))
        .await
        .unwrap();
    assert!(short.is_some());
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert!(
        cache.try_lock(ns, "lease", lease).await.unwrap().is_some(),
        "a lock is free again after its lease"
    );
}
