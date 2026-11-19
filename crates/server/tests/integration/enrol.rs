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

//! Enrolment through the server, as a client sees it: self-enrolment with a key, approvals, and the
//! participants of a course.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn people_enrol_themselves_with_a_key() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let instances = format!("/api/v1/enrol/courses/{course}/enrolment-instances");
    let open = site
        .call(
            Method::POST,
            &instances,
            Some(&admin),
            &[],
            Some(json!({
                "method": "self",
                "name": "Self",
                "role": site.roles["learner"],
                "key": "physics-2026",
            })),
        )
        .await;
    assert_eq!(open.status, StatusCode::CREATED, "{}", open.body);
    assert_eq!(open.body["has_key"], true);
    let open = open.body["id"].as_str().unwrap().to_owned();

    let (_, lea) = site.learner(&admin, "lea@example.org").await;
    let join = format!("/api/v1/enrol/enrolment-instances/{open}/self");
    let wrong = site
        .call(
            Method::POST,
            &join,
            Some(&lea),
            &[],
            Some(json!({ "key": "guess" })),
        )
        .await;
    assert_eq!(wrong.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(wrong.code(), "enrol.key_wrong");
    let joined = site
        .call(
            Method::POST,
            &join,
            Some(&lea),
            &[],
            Some(json!({ "key": "physics-2026" })),
        )
        .await;
    assert_eq!(joined.status, StatusCode::CREATED, "{}", joined.body);
    assert_eq!(joined.body["outcome"], "enrolled");

    let participants = format!("/api/v1/enrol/courses/{course}/participants?active=true");
    let listed = site
        .call(Method::GET, &participants, Some(&admin), &[], None)
        .await;
    assert_eq!(listed.body.as_array().unwrap().len(), 1);
    let hidden = site.call(Method::GET, &participants, Some(&lea), &[], None).await;
    assert_eq!(
        hidden.status,
        StatusCode::FORBIDDEN,
        "learners do not list participants"
    );
    site.drop().await;
}

#[tokio::test]
async fn a_request_to_join_waits_for_its_approval() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let instances = format!("/api/v1/enrol/courses/{course}/enrolment-instances");
    let participants = format!("/api/v1/enrol/courses/{course}/participants?active=true");
    let asked = site
        .call(
            Method::POST,
            &instances,
            Some(&admin),
            &[],
            Some(json!({
                "method": "self",
                "name": "On request",
                "role": site.roles["learner"],
                "requires_approval": true,
            })),
        )
        .await;
    let asked = asked.body["id"].as_str().unwrap().to_owned();
    let (_, bea) = site.learner(&admin, "bea@example.org").await;
    let requested = site
        .call(
            Method::POST,
            &format!("/api/v1/enrol/enrolment-instances/{asked}/self"),
            Some(&bea),
            &[],
            Some(json!({ "message": "May I join?" })),
        )
        .await;
    assert_eq!(requested.status, StatusCode::ACCEPTED, "{}", requested.body);
    let pending = site
        .call(
            Method::GET,
            &format!("/api/v1/enrol/courses/{course}/enrolment-requests"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(pending.body[0]["message"], "May I join?");
    let request = pending.body[0]["id"].as_str().unwrap().to_owned();
    let decided = site
        .call(
            Method::PUT,
            &format!("/api/v1/enrol/enrolment-requests/{request}/decision"),
            Some(&admin),
            &[],
            Some(json!({ "approve": true })),
        )
        .await;
    assert_eq!(decided.status, StatusCode::OK, "{}", decided.body);
    assert_eq!(decided.body["state"], "approved");
    let listed = site
        .call(Method::GET, &participants, Some(&admin), &[], None)
        .await;
    assert_eq!(listed.body.as_array().unwrap().len(), 1, "approving enrolled her");
    site.drop().await;
}
