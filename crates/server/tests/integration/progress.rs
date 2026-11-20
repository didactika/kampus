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

//! Progress through the server, as a client sees it: a teacher sets the rules of a page, a learner opens it and
//! sees their state, and a teacher decides a state with a reason.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_learner_opens_a_page_and_the_teacher_follows_their_progress() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let (lea_id, lea) = site.learner(&admin, "lea@example.org").await;
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

    let sections = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/courses/{course}/sections"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let section = sections.body[0]["id"].as_str().unwrap().to_owned();
    let page = site
        .call(
            Method::POST,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(&admin),
            &[],
            Some(
                json!({ "type": "activities.page", "title": "Welcome", "settings": { "content": "Hello" } }),
            ),
        )
        .await;
    let page = page.body["id"].as_str().unwrap().to_owned();
    let module = format!("/api/v1/progress/course-modules/{page}");

    let rules = site
        .call(
            Method::PUT,
            &format!("{module}/rules"),
            Some(&admin),
            &[],
            Some(json!({ "kinds": ["view"] })),
        )
        .await;
    assert_eq!(rules.status, StatusCode::OK, "{}", rules.body);
    assert_eq!(rules.body, json!(["view"]));
    let denied = site
        .call(
            Method::PUT,
            &format!("{module}/rules"),
            Some(&lea),
            &[],
            Some(json!({ "kinds": ["manual"] })),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    let before = site
        .call(
            Method::GET,
            &format!("{module}/states/{lea_id}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(before.body["state"], "not_started", "{}", before.body);
    let opened = site
        .call(Method::POST, &format!("{module}/views"), Some(&lea), &[], None)
        .await;
    assert_eq!(opened.status, StatusCode::OK, "{}", opened.body);
    assert_eq!(opened.body["state"], "done");
    let mine = site
        .call(
            Method::GET,
            &format!("/api/v1/progress/courses/{course}/my-progress"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(mine.body.as_array().unwrap().len(), 1, "{}", mine.body);

    let decided = site
        .call(
            Method::PUT,
            &format!("{module}/exceptions/{lea_id}"),
            Some(&admin),
            &[],
            Some(json!({ "state": "in_progress", "reason": "Reopened for a retake" })),
        )
        .await;
    assert_eq!(decided.status, StatusCode::OK, "{}", decided.body);
    assert_eq!(decided.body["state"], "in_progress");
    assert_eq!(decided.body["source"], "exception");
    let by_learner = site
        .call(
            Method::PUT,
            &format!("{module}/exceptions/{lea_id}"),
            Some(&lea),
            &[],
            Some(json!({ "state": "done", "reason": "I did it" })),
        )
        .await;
    assert_eq!(by_learner.status, StatusCode::FORBIDDEN);
    let removed = site
        .call(
            Method::DELETE,
            &format!("{module}/exceptions/{lea_id}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let after = site
        .call(
            Method::GET,
            &format!("{module}/states/{lea_id}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(after.body["state"], "done");
}
