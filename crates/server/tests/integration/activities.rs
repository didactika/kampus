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

//! Activities through the server, as a client sees them: a page and a file material added to a section,
//! read by a learner, moved, deleted to the bin and restored, and found by an identifier of another system.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
#[allow(clippy::too_many_lines, reason = "one session read from top to bottom")]
async fn a_teacher_builds_a_section_with_a_page_and_a_file_and_a_learner_reads_them() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
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
    let types = site
        .call(Method::GET, "/api/v1/activities/types", Some(&admin), &[], None)
        .await;
    assert_eq!(types.body.as_array().unwrap().len(), 2, "{}", types.body);

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
    assert_eq!(page.status, StatusCode::CREATED, "{}", page.body);
    let page_id = page.body["id"].as_str().unwrap().to_owned();
    let material = site
        .call(
            Method::POST,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(&admin),
            &[],
            Some(json!({ "type": "activities.file", "title": "Syllabus" })),
        )
        .await;
    let material_id = material.body["id"].as_str().unwrap().to_owned();
    let upload = site
        .call(
            Method::POST,
            "/api/v1/files/uploads",
            Some(&admin),
            &[],
            Some(json!({ "area": "activities.material", "scope": site.campus.to_string(), "name": "syllabus.txt", "size": 8 })),
        )
        .await;
    assert_eq!(upload.status, StatusCode::CREATED, "{}", upload.body);
    let upload = upload.body["id"].as_str().unwrap().to_owned();
    site.send_bytes(
        Method::PUT,
        &format!("/api/v1/files/uploads/{upload}"),
        &admin,
        b"syllabus",
    )
    .await;
    let file = site
        .call(
            Method::POST,
            &format!("/api/v1/files/uploads/{upload}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let file = file.body["id"].as_str().unwrap().to_owned();
    let attached = site
        .call(
            Method::PUT,
            &format!("/api/v1/activities/course-modules/{material_id}/file"),
            Some(&admin),
            &[],
            Some(json!({ "file": file })),
        )
        .await;
    assert_eq!(attached.status, StatusCode::OK, "{}", attached.body);

    let (_, learner) = site.learner(&admin, "lea@example.org").await;
    let listed = site
        .call(
            Method::GET,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(&learner),
            &[],
            None,
        )
        .await;
    assert_eq!(listed.body.as_array().unwrap().len(), 2, "{}", listed.body);
    let link = site
        .call(
            Method::POST,
            &format!("/api/v1/files/files/{file}/url"),
            Some(&learner),
            &[],
            None,
        )
        .await;
    assert_eq!(link.status, StatusCode::OK, "{}", link.body);
    let denied = site
        .call(
            Method::POST,
            &format!("/api/v1/activities/sections/{section}/course-modules"),
            Some(&learner),
            &[],
            Some(json!({ "type": "activities.page", "title": "Mine" })),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    let removed = site
        .call(
            Method::DELETE,
            &format!("/api/v1/activities/course-modules/{page_id}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let bin = site
        .call(
            Method::GET,
            &format!("/api/v1/activities/courses/{course}/bin"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(bin.body[0]["title"], "Welcome", "{}", bin.body);
    let restored = site
        .call(
            Method::POST,
            &format!("/api/v1/activities/course-modules/{page_id}/restore"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(restored.status, StatusCode::OK, "{}", restored.body);

    let by_reference = format!("/api/v1/activities/sections/{section}/course-modules/by-reference/sis/M-1");
    let fields = json!({ "type": "activities.page", "title": "Synced" });
    let first = site
        .call(
            Method::PUT,
            &by_reference,
            Some(&admin),
            &[],
            Some(fields.clone()),
        )
        .await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.body);
    let again = site
        .call(Method::PUT, &by_reference, Some(&admin), &[], Some(fields))
        .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.body);
    site.drop().await;
}
