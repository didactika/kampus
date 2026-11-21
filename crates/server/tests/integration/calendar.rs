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

//! Calendar through the server, as a client sees it: a teacher adds an entry to a course, a learner sees it in
//! a window, and a person keeps personal entries nobody else sees.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_teacher_adds_an_entry_to_a_course_and_a_learner_sees_it_with_the_personal_ones() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let (_, lea) = site.learner(&admin, "lea@example.org").await;
    let (_, max) = site.learner(&admin, "max@example.org").await;
    let instance = site
        .call(
            Method::POST,
            &format!("/api/v1/enrol/courses/{course}/enrolment-instances"),
            Some(&admin),
            &[],
            Some(json!({ "method": "self", "name": "Self", "role": site.roles["learner"] })),
        )
        .await;
    let instance = instance.body["id"].as_str().unwrap().to_owned();
    let joined = site
        .call(
            Method::POST,
            &format!("/api/v1/enrol/enrolment-instances/{instance}/self"),
            Some(&lea),
            &[],
            Some(json!({})),
        )
        .await;
    assert_eq!(joined.status, StatusCode::CREATED, "{}", joined.body);
    let detail = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let scope = detail.body["scope"].as_str().unwrap().to_owned();

    let created = site
        .call(
            Method::POST,
            "/api/v1/calendar/entries",
            Some(&admin),
            &[],
            Some(json!({
                "scope": scope,
                "title": "Midterm",
                "starts_at": "2030-03-01T09:00:00Z",
                "ends_at": "2030-03-01T11:00:00Z",
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let window = "from=2030-03-01T00:00:00Z&to=2030-03-31T00:00:00Z".to_owned();
    let seen = site
        .call(
            Method::GET,
            &format!("/api/v1/calendar/courses/{course}/calendar?{window}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(seen.status, StatusCode::OK, "{}", seen.body);
    assert_eq!(seen.body[0]["title"], "Midterm");
    assert_eq!(seen.body[0]["kind"], "entry");

    let denied = site
        .call(
            Method::POST,
            "/api/v1/calendar/entries",
            Some(&lea),
            &[],
            Some(json!({
                "scope": scope,
                "title": "Mine",
                "starts_at": "2030-03-02T09:00:00Z",
                "ends_at": "2030-03-02T10:00:00Z",
            })),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    let personal = site
        .call(
            Method::POST,
            "/api/v1/calendar/entries",
            Some(&lea),
            &[],
            Some(json!({
                "title": "Study",
                "starts_at": "2030-03-05T18:00:00Z",
                "ends_at": "2030-03-05T20:00:00Z",
            })),
        )
        .await;
    assert_eq!(personal.status, StatusCode::CREATED, "{}", personal.body);
    let mine = site
        .call(
            Method::GET,
            &format!("/api/v1/calendar/entries?{window}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(mine.body.as_array().unwrap().len(), 1, "{}", mine.body);
    let others = site
        .call(
            Method::GET,
            &format!("/api/v1/calendar/entries?{window}"),
            Some(&max),
            &[],
            None,
        )
        .await;
    assert_eq!(others.body.as_array().unwrap().len(), 0);
    let not_member = site
        .call(
            Method::GET,
            &format!("/api/v1/calendar/courses/{course}/calendar?{window}"),
            Some(&max),
            &[],
            None,
        )
        .await;
    assert_eq!(
        not_member.status,
        StatusCode::FORBIDDEN,
        "someone who does not take part does not see it"
    );
    site.drop().await;
}
