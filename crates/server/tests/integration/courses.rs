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

//! Courses through the server, as a client sees them: the category tree, a course with its owners and
//! sections, its lifecycle with `If-Match`, hidden content and groups.

use axum::http::{HeaderMap, Method, StatusCode, header};
use serde_json::{Value, json};

use crate::site::{PASSWORD, Site, site};

impl Site {
    /// Invites a person and lets them choose [`PASSWORD`]; returns the account and a session.
    pub(crate) async fn learner(&self, admin: &str, email: &str) -> (String, String) {
        let invited = self
            .call(
                Method::POST,
                &format!("/api/v1/campuses/{}/accounts", self.campus),
                Some(admin),
                &[],
                Some(json!({ "email": email })),
            )
            .await;
        assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
        self.call(
            Method::POST,
            "/api/v1/auth/invitations/accept",
            None,
            &[],
            Some(json!({ "token": self.token_sent_to(email), "password": PASSWORD })),
        )
        .await;
        let account = invited.body["id"].as_str().unwrap().to_owned();
        (account, self.session(email).await)
    }

    /// Creates a category and a course in it; returns both ids.
    pub(crate) async fn course(&self, admin: &str) -> (String, String) {
        let science = self
            .call(
                Method::POST,
                &format!("/api/v1/campuses/{}/categories", self.campus),
                Some(admin),
                &[],
                Some(json!({ "name": "Science", "code": "SCI" })),
            )
            .await;
        assert_eq!(science.status, StatusCode::CREATED, "{}", science.body);
        let category = science.body["id"].as_str().unwrap().to_owned();
        let created = self
            .call(
                Method::POST,
                &format!("/api/v1/categories/{category}/courses"),
                Some(admin),
                &[],
                Some(json!({ "short_name": "PHY101", "full_name": "Physics I" })),
            )
            .await;
        assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
        assert_eq!(created.body["state"], "draft");
        (category, created.body["id"].as_str().unwrap().to_owned())
    }
}

/// The value of an `ETag`.
fn etag(headers: &HeaderMap) -> String {
    headers[header::ETAG].to_str().unwrap().to_owned()
}

#[tokio::test]
async fn a_course_is_built_with_sections_and_published_with_if_match() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let read = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(
        read.body["owners"].as_array().unwrap().len(),
        1,
        "its creator owns it"
    );

    let added = site
        .call(
            Method::POST,
            &format!("/api/v1/courses/{course}/sections"),
            Some(&admin),
            &[],
            Some(json!({ "title": "Unit 1", "place": { "at": "first" } })),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.body);
    let sections = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}/sections"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let titles: Vec<Value> = sections
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["title"].clone())
        .collect();
    assert_eq!(
        titles,
        [json!("Unit 1"), Value::Null],
        "placed before the first section"
    );

    let state = format!("/api/v1/courses/{course}/state");
    let unconditional = site
        .call(
            Method::PUT,
            &state,
            Some(&admin),
            &[],
            Some(json!({ "to": "published" })),
        )
        .await;
    assert_eq!(unconditional.status, StatusCode::PRECONDITION_FAILED);
    let published = site
        .call(
            Method::PUT,
            &state,
            Some(&admin),
            &[("if-match", &etag(&read.headers))],
            Some(json!({ "to": "published" })),
        )
        .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.body);
    assert_eq!(published.body["state"], "published");
    let unknown = site
        .call(
            Method::PUT,
            &state,
            Some(&admin),
            &[("if-match", &etag(&published.headers))],
            Some(json!({ "to": "flying" })),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn learners_see_only_what_is_shown_and_do_not_edit() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (category, course) = site.course(&admin).await;
    let (_, lea) = site.learner(&admin, "lea@example.org").await;
    let seen = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(seen.status, StatusCode::OK);

    let category_read = site
        .call(
            Method::GET,
            &format!("/api/v1/categories/{category}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    let hidden = site
        .call(
            Method::PUT,
            &format!("/api/v1/categories/{category}"),
            Some(&admin),
            &[("if-match", &etag(&category_read.headers))],
            Some(json!({ "name": "Science", "code": "SCI", "visibility": { "mode": "hidden" } })),
        )
        .await;
    assert_eq!(hidden.status, StatusCode::OK, "{}", hidden.body);
    let gone = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(
        gone.status,
        StatusCode::NOT_FOUND,
        "a course in a hidden category is not shown"
    );
    let listed = site
        .call(
            Method::GET,
            &format!("/api/v1/campuses/{}/categories", site.campus),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(listed.body, json!([]));
    let still = site
        .call(
            Method::GET,
            &format!("/api/v1/courses/{course}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(still.status, StatusCode::OK, "who may see hidden content sees it");

    let edit = site
        .call(
            Method::PUT,
            &format!("/api/v1/courses/{course}"),
            Some(&lea),
            &[("if-match", &etag(&still.headers))],
            Some(json!({ "short_name": "PHY101", "full_name": "Mine now" })),
        )
        .await;
    assert_eq!(
        edit.status,
        StatusCode::FORBIDDEN,
        "a learner does not edit the course"
    );
}

#[tokio::test]
async fn groups_split_the_people_of_a_course() {
    let site = site().await;
    let admin = site.session("admin").await;
    let (_, course) = site.course(&admin).await;
    let groups = format!("/api/v1/courses/{course}/groups");
    let group = site
        .call(
            Method::POST,
            &groups,
            Some(&admin),
            &[],
            Some(json!({ "name": "Red", "visibility": "members" })),
        )
        .await;
    assert_eq!(group.status, StatusCode::CREATED, "{}", group.body);
    let group = group.body["id"].as_str().unwrap().to_owned();
    let taken = site
        .call(
            Method::POST,
            &groups,
            Some(&admin),
            &[],
            Some(json!({ "name": "RED" })),
        )
        .await;
    assert_eq!(taken.status, StatusCode::CONFLICT);
    assert_eq!(taken.code(), "courses.group_name_taken");

    let (lea_id, lea) = site.learner(&admin, "lea@example.org").await;
    let outsider = site
        .call(
            Method::POST,
            &format!("/api/v1/groups/{group}/members"),
            Some(&admin),
            &[],
            Some(json!({ "account": lea_id })),
        )
        .await;
    assert_eq!(
        outsider.code(),
        "courses.member_not_participant",
        "only participants join groups"
    );
    site.enrol(&admin, &course, &lea_id, "learner").await;
    let before = site.call(Method::GET, &groups, Some(&lea), &[], None).await;
    assert_eq!(
        before.body,
        json!([]),
        "a group shown to its members is not shown to others"
    );
    let added = site
        .call(
            Method::POST,
            &format!("/api/v1/groups/{group}/members"),
            Some(&admin),
            &[],
            Some(json!({ "account": lea_id })),
        )
        .await;
    assert_eq!(added.status, StatusCode::NO_CONTENT, "{}", added.body);
    let after = site.call(Method::GET, &groups, Some(&lea), &[], None).await;
    assert_eq!(after.body.as_array().unwrap().len(), 1);
    let members = site
        .call(
            Method::GET,
            &format!("/api/v1/groups/{group}/members"),
            Some(&lea),
            &[],
            None,
        )
        .await;
    assert_eq!(members.body[0]["origin"]["kind"], "manual");

    let sets = format!("/api/v1/courses/{course}/group-sets");
    let set = site
        .call(
            Method::POST,
            &sets,
            Some(&admin),
            &[],
            Some(json!({ "name": "Labs" })),
        )
        .await;
    assert_eq!(set.status, StatusCode::CREATED, "{}", set.body);
    let set = set.body["id"].as_str().unwrap().to_owned();
    let put = site
        .call(
            Method::PUT,
            &format!("/api/v1/group-sets/{set}/groups/{group}"),
            Some(&admin),
            &[],
            None,
        )
        .await;
    assert_eq!(put.status, StatusCode::NO_CONTENT);
    let listed = site.call(Method::GET, &sets, Some(&admin), &[], None).await;
    assert_eq!(listed.body[0]["groups"], json!([group]));
    let forbidden = site.call(Method::GET, &sets, Some(&lea), &[], None).await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
}
