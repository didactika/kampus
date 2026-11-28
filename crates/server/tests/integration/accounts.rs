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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The accounts of a campus as its administration lists them: a page at a time, found by words, kept by
//! status, and changed in bulk with what happened to each.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::site::{Site, site};

/// Invites a person and answers the id of the account.
async fn invite(site: &Site, admin: &str, email: &str) -> String {
    let invited = site
        .call(
            Method::POST,
            &format!("/api/v1/identity/campuses/{}/accounts", site.campus),
            Some(admin),
            &[],
            Some(json!({ "email": email })),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
    invited.body["id"].as_str().unwrap().to_owned()
}

/// The ids of the accounts of a page.
fn ids(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn the_accounts_of_a_campus_are_listed_found_and_changed_in_bulk() {
    let site = site().await;
    let admin = site.session("admin").await;
    let list = format!("/api/v1/identity/campuses/{}/accounts", site.campus);
    let mut invited = Vec::new();
    for email in ["ada@example.org", "grace@example.org", "alan@example.org"] {
        invited.push(invite(&site, &admin, email).await);
    }

    let first = site
        .call(Method::GET, &format!("{list}?limit=2"), Some(&admin), &[], None)
        .await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.body);
    assert_eq!(
        ids(&first.body),
        vec![invited[2].clone(), invited[1].clone()],
        "newest first"
    );
    let after = first.body["next"].as_str().unwrap();
    let second = site
        .call(
            Method::GET,
            &format!("{list}?limit=2&after={after}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(ids(&second.body)[0], invited[0]);

    let found = site
        .call(Method::GET, &format!("{list}?q=GRACE"), Some(&admin), &[], None)
        .await;
    assert_eq!(ids(&found.body), vec![invited[1].clone()]);
    let literal = site
        .call(Method::GET, &format!("{list}?q=%25"), Some(&admin), &[], None)
        .await;
    assert!(
        ids(&literal.body).is_empty(),
        "a percent sign is found as written"
    );

    let unknown = "0190a000-0000-7000-8000-000000000000";
    let bulk = site
        .call(
            Method::POST,
            &format!("{list}/transitions"),
            Some(&admin),
            &[],
            Some(json!({ "accounts": [invited[0], invited[1], unknown], "to": "suspended", "reason": "review" })),
        )
        .await;
    assert_eq!(bulk.status, StatusCode::OK, "{}", bulk.body);
    let problems: Vec<Value> = bulk
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["problem"].clone())
        .collect();
    assert_eq!(
        problems,
        vec![Value::Null, Value::Null, json!("identity.account_not_found")]
    );
    let suspended = site
        .call(
            Method::GET,
            &format!("{list}?status=suspended"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(ids(&suspended.body).len(), 2, "{}", suspended.body);

    let empty = site
        .call(
            Method::POST,
            &format!("{list}/transitions"),
            Some(&admin),
            &[],
            Some(json!({ "accounts": [], "to": "suspended" })),
        )
        .await;
    assert_eq!(empty.code(), "identity.bulk_invalid");
    let visitor = site.call(Method::GET, &list, None, &[], None).await;
    assert_eq!(visitor.status, StatusCode::FORBIDDEN);
    site.drop().await;
}
