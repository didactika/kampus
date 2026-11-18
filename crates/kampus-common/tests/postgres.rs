// This file is part of Kampus - Knowledge Academic Modular Platform for Universal Study.
//
// Copyright (c) 2026 Didactika. All rights reserved.
// Proprietary and confidential: see the LICENSE file at the root of the repository.
// SPDX-License-Identifier: LicenseRef-Didactika-Proprietary
//
// @package   kampus-common
// @author    hector-ae21
// @since     0.1.0

//! Proves that the test environment reaches PostgreSQL 17, as the tests of every module will.

use sqlx::Connection;

#[tokio::test]
async fn postgres_17_is_reachable() {
    let url = std::env::var("KAMPUS_TEST_DATABASE_URL").expect("KAMPUS_TEST_DATABASE_URL is set");
    let mut connection = sqlx::PgConnection::connect(&url)
        .await
        .expect("the test database accepts connections");
    let (version,): (String,) = sqlx::query_as("select current_setting('server_version')")
        .fetch_one(&mut connection)
        .await
        .expect("the server answers");
    assert!(
        version.starts_with("17"),
        "PostgreSQL 17 expected, found {version}"
    );
}
