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

//! Customisation through the server: a campus renames the words of courses, adds a custom field that
//! courses fill in, and turns groups off in one course.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::site::site;

#[tokio::test]
async fn a_campus_renames_terms_adds_course_fields_and_turns_groups_off() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let campus = site.campus.to_string();
    let renamed = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{campus}/terms/courses.course"),
            Some(&admin),
            &[],
            Some(json!({ "language": "es", "singular": "asignatura", "plural": "asignaturas" })),
        )
        .await;
    assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.body);
    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let scope = read.body["scope"].as_str().unwrap().to_owned();
    let term = site
        .call(
            Method::GET,
            &format!("/api/v1/scopes/{scope}/terms/courses.course?language=es"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(term.body["plural"], "asignaturas", "{}", term.body);

    let field = site
        .call(
            Method::POST,
            &format!("/api/v1/scopes/{campus}/fields"),
            Some(&admin),
            &[],
            Some(
                json!({ "kind": "courses.course", "code": "credits", "name": "Credits", "type": "integer" }),
            ),
        )
        .await;
    assert_eq!(field.status, StatusCode::CREATED, "{}", field.body);
    let set = site
        .call(
            Method::PUT,
            &format!("/api/v1/courses/{course}/fields/credits"),
            Some(&admin),
            &[],
            Some(json!({ "value": 6 })),
        )
        .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.body);
    let fields = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}/fields"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(fields.body[0]["value"], 6, "{}", fields.body);

    let off = site
        .call(
            Method::PUT,
            &format!("/api/v1/scopes/{scope}/flags/courses.groups"),
            Some(&admin),
            &[],
            Some(json!({ "enabled": false })),
        )
        .await;
    assert_eq!(off.status, StatusCode::OK, "{}", off.body);
    let group = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/{course}/groups"),
            Some(&admin),
            &[],
            Some(json!({ "name": "Red" })),
        )
        .await;
    assert_eq!(group.status, StatusCode::CONFLICT, "{}", group.body);
    assert_eq!(group.code(), "customise.feature_off");
}
