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
// @package   kampus-scan
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The conformance suite of the scanning port: an adapter is supported only when it passes every check
//! here, run against a real scanner.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "a conformance failure must stop the test"
)]

use bytes::Bytes;

use crate::{EICAR, Scanner, Verdict};

/// Runs every check.
pub async fn run(scanner: &dyn Scanner) {
    scanner.check().await.expect("the scanner answers");
    assert_eq!(
        scanner.scan(Bytes::from_static(b"plain words")).await.unwrap(),
        Verdict::Clean,
        "harmless content is clean"
    );
    assert!(
        matches!(
            scanner.scan(Bytes::from_static(EICAR)).await.unwrap(),
            Verdict::Infected(_)
        ),
        "the test file is found"
    );
    assert_eq!(
        scanner.scan(Bytes::new()).await.unwrap(),
        Verdict::Clean,
        "empty content is clean"
    );
    let large = Bytes::from(vec![b'a'; 3 * 1024 * 1024]);
    assert_eq!(
        scanner.scan(large).await.unwrap(),
        Verdict::Clean,
        "large content is scanned in chunks"
    );
}
